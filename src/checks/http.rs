#![ allow (unused_parens) ]

extern crate getopts;

use std::collections::HashMap;
use std::error;
use std::time;

use logic::*;
use lowlevel::http;

pub fn new (
) -> Box <PluginProvider> {

	Box::new (
		CheckHttpProvider {},
	)

}

struct CheckHttpProvider {
}

struct CheckHttpInstance {

	address: String,
	port: u64,
	secure: bool,

	method: http::HttpMethod,
	path: String,

	send_headers: Vec <(String, String)>,

	expect_status_code: Vec <u64>,
	expect_headers: Vec <(String, String)>,
	expect_body_text: Option <String>,

	response_time_warning: Option <time::Duration>,
	response_time_critical: Option <time::Duration>,

	timeout: time::Duration,

}

impl PluginProvider
for CheckHttpProvider {

	fn name (
		& self,
	) -> & str {
		"check-http"
	}

	fn prefix (
		& self,
	) -> & str {
		"HTTP"
	}

	fn build_options_spec (
		& self,
	) -> getopts::Options {

		let mut options_spec =
			getopts::Options::new ();

		options_spec.optflag (
			"",
			"help",
			"print this help menu");

		// connection options

		options_spec.reqopt (
			"",
			"address",
			"address to connect to, hostname or IP",
			"ADDRESS");

		options_spec.optopt (
			"",
			"port",
			"port to connect to, defaults to 80 or 443",
			"PORT");

		options_spec.optflag (
			"",
			"secure",
			"use a secure connection, ie SSL or TLS");

		// request options

		options_spec.optopt (
			"",
			"method",
			"method to use: get (default) or post",
			"METHOD");

		options_spec.optopt (
			"",
			"path",
			"path to request, defaults to /",
			"PATH");

		options_spec.optmulti (
			"",
			"send-header",
			"header to send, eg 'name: value'",
			"NAME:VALUE");

		// response options

		options_spec.optopt (
			"",
			"expect-status-code",
			"status code to expect, defaults to 200",
			"CODE");

		options_spec.optmulti (
			"",
			"expect-header",
			"header to expect, eg 'name: value'",
			"NAME:VALUE");

		options_spec.optopt (
			"",
			"expect-body-text",
			"text to expect in body",
			"TEXT");

		// timings

		options_spec.optopt (
			"",
			"response-time-warning",
			"total response time warning threshold",
			"DURATION");

		options_spec.optopt (
			"",
			"response-time-critical",
			"total response time critical threshold",
			"DURATION");

		options_spec.optopt (
			"",
			"timeout",
			"maximum time to wait for server response, defaults to 60 seconds",
			"DURATION");

		// return

		options_spec

	}

	fn new_instance (
		& self,
		_options_spec: & getopts::Options,
		options_matches: & getopts::Matches,
	) -> Result <Box <PluginInstance>, Box <error::Error>> {

		// determine secure parameter beforehand

		let secure =
			try! (
				arghelper::check_if_present (
					options_matches,
					"secure"));

		// return

		Ok (Box::new (
			CheckHttpInstance {

			// connection

			address:
				try! (
					arghelper::parse_string_required (
						options_matches,
						"address")),

			port:
				try! (
					arghelper::parse_positive_integer_or_default (
						options_matches,
						"port",
						if secure { 443 } else { 80 })),

			secure:
				secure,

			// request

			method:
				http::HttpMethod::Get,

			path:
				try! (
					arghelper::parse_string_or_default (
						options_matches,
						"path",
						"/")),

			send_headers:
				try! (
					parse_headers (
						try! (
							arghelper::parse_string_multiple (
								options_matches,
								"send-header")))),

			// response

			expect_status_code:
				try! (
					arghelper::parse_positive_integer_multiple_or_default (
						options_matches,
						"expect-status-code",
						& vec! [ 200 ])),

			expect_headers:
				try! (
					parse_headers (
						try! (
							arghelper::parse_string_multiple (
								options_matches,
								"expect-header")))),

			expect_body_text:
				try! (
					arghelper::parse_string (
						options_matches,
						"expect-body-text")),

			// timings

			response_time_warning:
				try! (
					arghelper::parse_duration (
						options_matches,
						"response-time-warning")),

			response_time_critical:
				try! (
					arghelper::parse_duration (
						options_matches,
						"response-time-critical")),

			timeout:
				try! (
					arghelper::parse_duration_or_default (
						options_matches,
						"timeout",
						& time::Duration::new (60, 0))),

		}))

	}

}

impl PluginInstance
for CheckHttpInstance {

	fn perform_check (
		& self,
		plugin_provider: & PluginProvider,
	) -> Result <CheckResult, Box <error::Error>> {

		let mut check_result_builder =
			CheckResultBuilder::new ();

		// perform http request

		try! (
			self.check_http (
				& mut check_result_builder));

		// return

		Ok (
			check_result_builder.into_check_result (
				plugin_provider,
			)
		)

	}

}

impl CheckHttpInstance {

	fn check_http (
		& self,
		check_result_builder: & mut CheckResultBuilder,
	) -> Result <(), Box <error::Error>> {

		let http_request =
			http::HttpRequest {

			address: & self.address,
			port: self.port,
			secure: self.secure,

			method: self.method,
			path: & self.path,
			headers: & self.send_headers,

			timeout: self.timeout,

		};

		match (
			http::perform_request (
				& http_request)
		) {

			http::PerformRequestResult::Success (http_response) =>
				try! (
					self.process_response (
						check_result_builder,
						& http_response)),

			http::PerformRequestResult::Failure (reason) =>
				check_result_builder.critical (
					format! (
						"failed to connect: {}",
						reason)),

			http::PerformRequestResult::Timeout (duration) =>
				check_result_builder.critical (
					format! (
						"request timed out after {}",
						checkhelper::display_duration_long (
							& duration))),

		}

		Ok (())

	}

	fn process_response (
		& self,
		check_result_builder: & mut CheckResultBuilder,
		http_response: & http::HttpResponse,
	) -> Result <(), Box <error::Error>> {

		try! (
			self.check_response_status_code (
				check_result_builder,
				http_response));

		try! (
			self.check_response_headers (
				check_result_builder,
				http_response));

		try! (
			self.check_response_body (
				check_result_builder,
				http_response));

		try! (
			self.check_response_timing (
				check_result_builder,
				http_response));

		Ok (())

	}

	fn check_response_status_code (
		& self,
		check_result_builder: & mut CheckResultBuilder,
		http_response: & http::HttpResponse,
	) -> Result <(), Box <error::Error>> {

		if self.expect_status_code.contains (
			& http_response.status_code,
		) {

			check_result_builder.ok (
				format! (
					"status {}",
					http_response.status_code));

		} else {

			check_result_builder.critical (
				format! (
					"status {}",
					http_response.status_code));

		}

		Ok (())

	}

	fn check_response_headers (
		& self,
		check_result_builder: & mut CheckResultBuilder,
		http_response: & http::HttpResponse,
	) -> Result <(), Box <error::Error>> {

		if ! self.expect_headers.is_empty () {

			let mut matched_headers: Vec <(String, String)> =
				vec! [];

			let mut missing_headers: Vec <(String, String)> =
				vec! [];

			let mut mismatched_headers: Vec <(String, String, String)> =
				vec! [];

			let response_headers_map: HashMap <String, String> =
				http_response.headers.iter ().map (
					|& (ref header_name, ref header_value)|
					(
						header_name.to_lowercase (),
						header_value.to_owned (),
					)
				).collect ();

			for & (ref expect_header_name, ref expect_header_value)
			in self.expect_headers.iter () {

				match response_headers_map.get (
					& expect_header_name.to_lowercase ()) {

					None => {

						missing_headers.push (
							(
								expect_header_name.to_owned (),
								expect_header_value.to_owned (),
							)
						);

					},

					Some (actual_header_value) => {

						if actual_header_value == expect_header_value {

							matched_headers.push (
								(
									expect_header_name.to_owned (),
									expect_header_value.to_owned (),
								)
							);

						} else {

							mismatched_headers.push (
								(
									expect_header_name.to_owned (),
									expect_header_value.to_owned (),
									actual_header_value.to_owned (),
								)
							);

						}

					},

				}

			}

			if ! matched_headers.is_empty () {

				check_result_builder.ok (
					format! (
						"matched {} headers",
						matched_headers.len ()));

			}

			if ! missing_headers.is_empty () {

				check_result_builder.warning (
					format! (
						"missing {} headers",
						missing_headers.len ()));

			}

			if ! mismatched_headers.is_empty () {

				check_result_builder.critical (
					format! (
						"failed to match {} headers",
						mismatched_headers.len ()));

			}

		}

		Ok (())

	}

	fn check_response_body (
		& self,
		check_result_builder: & mut CheckResultBuilder,
		http_response: & http::HttpResponse,
	) -> Result <(), Box <error::Error>> {

		if self.expect_body_text.is_some () {

			let expect_body_text =
				self.expect_body_text.as_ref ().unwrap ();

			if http_response.body.contains (
				expect_body_text) {

				check_result_builder.ok (
					"body text matched");

			} else {

				check_result_builder.critical (
					"body text not matched");

			}

		}

		Ok (())

	}

	fn check_response_timing (
		& self,
		check_result_builder: & mut CheckResultBuilder,
		http_response: & http::HttpResponse,
	) -> Result <(), Box <error::Error>> {

		checkhelper::check_duration_less_than (
			check_result_builder,
			& self.response_time_warning,
			& self.response_time_critical,
			& format! (
				"request took {}",
				checkhelper::display_duration_long (
					& http_response.duration)),
			& http_response.duration);

		Ok (())

	}

}

fn parse_headers (
	header_strings: Vec <String>,
) -> Result <Vec <(String, String)>, Box <error::Error>> {

	let mut header_tuples: Vec <(String, String)> =
		vec! [];

	for header_string in header_strings.iter () {

		header_tuples.push (
			try! (
				parse_header (
					header_string)));

	}

	Ok (header_tuples)

}

fn parse_header (
	header_string: & str,
) -> Result <(String, String), Box <error::Error>> {

	let split_position =
		match header_string.find (
			':') {

		Some (pos) => pos,

		None =>
			return Err (

			Box::new (
				SimpleError::from (
					"Header strings must be 'name:value' format"))

		),

	};

	let (name_raw, rest_raw) =
		header_string.split_at (
			split_position);

	let value_raw =
		& rest_raw [1..];

	Ok (

		(
			name_raw.trim ().to_string (),
			value_raw.trim ().to_string (),
		)

	)

}

// ex: noet ts=4 filetype=rust
