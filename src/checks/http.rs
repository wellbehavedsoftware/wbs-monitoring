#![ allow (unused_parens) ]

extern crate getopts;

use std::collections::HashMap;
use std::error;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use std::time::Instant;

use chrono::Duration as ChronoDuration;
use chrono::UTC;

use itertools::Itertools;

use resolv;

use logic::*;
use lowlevel::http;

check! {

	new = new,
	name = "check-http",
	prefix = "HTTP",

	provider = CheckHttpProvider,

	instance = CheckHttpInstance {

		address: String,
		port: u64,
		secure: bool,

		method: http::HttpMethod,
		path: String,

		send_headers: Vec <(String, String)>,

		expect_status_code: Vec <u64>,
		expect_headers: Vec <(String, String)>,
		expect_body_text: Option <String>,

		response_time_warning: Option <Duration>,
		response_time_critical: Option <Duration>,

		timeout: Duration,

	},

	options_spec = |options_spec| {

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

	},

	options_parse = |options_matches| {

		// determine secure parameter beforehand

		let secure =
			arg_helper::check_if_present (
				options_matches,
				"secure",
			) ?;

		// return

		CheckHttpInstance {

			// connection

			address:
				arg_helper::parse_string_required (
					options_matches,
					"address",
				) ?,

			port:
				arg_helper::parse_positive_integer_or_default (
					options_matches,
					"port",
					if secure { 443 } else { 80 },
				) ?,

			secure:
				secure,

			// request

			method:
				http::HttpMethod::Get,

			path:
				arg_helper::parse_string_or_default (
					options_matches,
					"path",
					"/",
				) ?,

			send_headers:
				parse_headers (
					arg_helper::parse_string_multiple (
						options_matches,
						"send-header",
					) ?,
				) ?,

			// response

			expect_status_code:
				arg_helper::parse_positive_integer_multiple_or_default (
					options_matches,
					"expect-status-code",
					& vec! [ 200 ],
				) ?,

			expect_headers:
				parse_headers (
					arg_helper::parse_string_multiple (
						options_matches,
						"expect-header",
					) ?,
				) ?,

			expect_body_text:
				arg_helper::parse_string (
					options_matches,
					"expect-body-text",
				) ?,

			// timings

			response_time_warning:
				arg_helper::parse_duration (
					options_matches,
					"response-time-warning",
				) ?,

			response_time_critical:
				arg_helper::parse_duration (
					options_matches,
					"response-time-critical",
				) ?,

			timeout:
				arg_helper::parse_duration_or_default (
					options_matches,
					"timeout",
					& Duration::new (60, 0),
				) ?,

		}

	},

	perform = |self, plugin_provider, check_result_builder| {

		// perform http request

		self.check_http (
			& mut check_result_builder,
		) ?;

	},

}

impl CheckHttpInstance {

	fn check_http (
		& self,
		check_result_builder: & mut CheckResultBuilder,
	) -> Result <(), Box <error::Error>> {

		let (lookup_duration, addresses) =
			self.perform_hostname_lookup ();

		let num_addresses =
			addresses.len ();

		if num_addresses == 0 {

			check_result_builder.critical (
				format! (
					"Failed to resolve hostname: {}",
					& self.address));

			return Ok (());

		}

		check_result_builder.extra_information (
			format! (
				"Resolved {} to {} hosts in {}",
				& self.address,
				num_addresses,
				check_helper::display_duration_long (
					& lookup_duration)));

		let request_results =
			self.perform_requests (
				addresses,
			) ?;

		let mut num_successes: u64 = 0;
		let mut num_warnings: u64 = 0;
		let mut num_criticals: u64 = 0;
		let mut num_connection_errors: u64 = 0;
		let mut num_timeouts: u64 = 0;
		let mut num_other_errors: u64 = 0;

		let mut durations: Vec <Duration> =
			Vec::new ();

		for (address, result) in request_results {

			match result {

				RequestResult::Success (result) => {

					check_result_builder.extra_information (
						format! (
							"{}: {}",
							address,
							result.messages.iter ().join (
								& ", ".to_string (),
							)));

					durations.push (
						result.duration);

					match result.check_status {

						CheckStatus::Ok =>
							num_successes += 1,

						CheckStatus::Warning =>
							num_warnings += 1,

						CheckStatus::Critical =>
							num_criticals += 1,

						_ =>
							panic! (),

					};

					check_result_builder.update_status (
						result.check_status);

				},

				RequestResult::ConnectionError (error) => {

					check_result_builder.extra_information (
						format! (
							"{}: {}",
							address,
							error));

					num_connection_errors += 1;

					check_result_builder.update_status (
						CheckStatus::Critical);

				},

				RequestResult::Timeout => {

					check_result_builder.extra_information (
						format! (
							"{}: timeout",
							address));

					num_timeouts += 1;

					check_result_builder.update_status (
						CheckStatus::Critical);

				},

				RequestResult::OtherError (error) => {

					check_result_builder.extra_information (
						format! (
							"{}: {}",
							address,
							error));

					num_other_errors += 1;

					check_result_builder.update_status (
						CheckStatus::Critical);

				},

			}

		}

		for (count, label) in vec! [
			(num_other_errors, "reported unknown errors"),
			(num_timeouts, "timed out"),
			(num_connection_errors, "failed to connect"),
			(num_criticals, "critical"),
			(num_warnings, "warning"),
			(num_successes, "ok"),
		] {

			if count > 0 {

				check_result_builder.ok (
					format! (
						"{} {} {}",
						count,
						if count == 1 { "host" } else { "hosts" },
						label));

			}

		}

		self.check_response_timing (
			check_result_builder,
			& durations
		);

		Ok (())

	}

	fn perform_hostname_lookup (
		& self,
	) -> (Duration, Vec <String>) {

		let start_time =
			Instant::now ();

		let mut qualified_address =
			self.address.to_string ();

		let last_character =
			qualified_address.chars ().rev ().next ().unwrap ();

		if last_character != '.' {
			qualified_address.push ('.');
		}

		let mut resolver =
			resolv::Resolver::new ().unwrap ();

		let mut addresses: Vec <String> =
			Vec::new ();

		if let Ok (mut response) =
			resolver.query (
				qualified_address.as_bytes (),
				resolv::Class::IN,
				resolv::RecordType::A,
			) {

			for index in 0 .. response.get_section_count (
				resolv::Section::Answer) {

				if let Ok (record) =
					response.get_record::<resolv::record::A> (
						resolv::Section::Answer,
						index,
					) {

					addresses.push (
						format! (
							"{}",
							record.data.address));

				}

			}

		}

		addresses.sort ();

		let end_time =
			Instant::now ();

		let duration =
			end_time.duration_since (
				start_time);

		(
			duration,
			addresses,
		)

	}

	fn perform_requests (
		& self,
		addresses: Vec <String>,
	) -> Result <Vec <(String, RequestResult)>, String> {

		let request_futures: Vec <(String, JoinHandle <RequestResult>)> =
			addresses.into_iter ().map (
				|address| {

			let self_copy =
				self.clone ();

			(
				address.clone (),
				thread::spawn (
					move ||

					self_copy.perform_request (
						& address)

				),
			)

		}).collect ();

		Ok (request_futures.into_iter ().map (
			|(address, request_future)|

			match request_future.join () {

				Ok (result) =>
					(address, result),

				Err (error) => (

					address,

					RequestResult::OtherError (
						error.downcast::<String> (
						).map (
							|boxed_error| * boxed_error
						).unwrap_or (
							"unknown internal error".to_string ()
						)
					)

				),

			}

		).collect ())

	}

	fn perform_request (
		& self,
		address: & str,
	) -> RequestResult {

		let mut send_headers =
			self.send_headers.clone ();

		send_headers.push ((
			"Host".to_string (),
			self.address.clone (),
		));

		let http_request =
			http::HttpRequest {

			address: address,
			hostname: & self.address,
			port: self.port,
			secure: self.secure,

			method: self.method,
			path: & self.path,
			headers: & send_headers,

			timeout: self.timeout,

		};

		match (
			http::perform_request (
				& http_request)
		) {

			http::PerformRequestResult::Success (http_response) =>
				self.process_response (
					& http_response,
				).unwrap_or_else (
					|error|

					RequestResult::OtherError (
						error.description ().to_string ())

				),

			http::PerformRequestResult::Failure (reason) =>
				RequestResult::ConnectionError (
					format! (
						"failed to connect: {}",
						reason)),

			http::PerformRequestResult::Timeout (_duration) =>
				RequestResult::Timeout,

		}

	}

	fn process_response (
		& self,
		http_response: & http::HttpResponse,
	) -> Result <RequestResult, Box <error::Error>> {

		let mut result =
			RequestResultSuccess {
				check_status: CheckStatus::Ok,
				duration: http_response.duration,
				messages: Vec::new (),
			};

		self.check_response_status_code (
			& mut result,
			http_response,
		) ?;

		self.check_response_headers (
			& mut result,
			http_response,
		) ?;

		self.check_response_body (
			& mut result,
			http_response,
		) ?;

		self.check_certificate_expiry (
			& mut result,
			http_response,
		) ?;

		Ok (RequestResult::Success (result))

	}

	fn check_response_status_code (
		& self,
		result: & mut RequestResultSuccess,
		http_response: & http::HttpResponse,
	) -> Result <(), Box <error::Error>> {

		if self.expect_status_code.contains (
			& http_response.status_code,
		) {

			result.messages.push (
				format! (
					"status {}",
					http_response.status_code));

		} else {

			result.messages.push (
				format! (
					"status {} (critical)",
					http_response.status_code));

			result.check_status.update (
				CheckStatus::Critical);

		}

		Ok (())

	}

	fn check_response_headers (
		& self,
		result: & mut RequestResultSuccess,
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

				result.messages.push (
					format! (
						"matched {} headers",
						matched_headers.len ()));

			}

			if ! missing_headers.is_empty () {

				result.messages.push (
					format! (
						"missing {} headers (warning)",
						missing_headers.len ()));

				result.check_status.update (
					CheckStatus::Warning);

			}

			if ! mismatched_headers.is_empty () {

				result.messages.push (
					format! (
						"failed to match {} headers (critical)",
						mismatched_headers.len ()));

				result.check_status =
					CheckStatus::Critical;

			}

		}

		Ok (())

	}

	fn check_response_body (
		& self,
		result: & mut RequestResultSuccess,
		http_response: & http::HttpResponse,
	) -> Result <(), Box <error::Error>> {

		if self.expect_body_text.is_some () {

			let expect_body_text =
				self.expect_body_text.as_ref ().unwrap ();

			if http_response.body.contains (
				expect_body_text) {

				result.messages.push (
					"body text matched".to_string ());

			} else {

				result.messages.push (
					"body text not matched (critical)".to_string ());

				result.check_status =
					CheckStatus::Critical;

			}

		}

		Ok (())

	}

	fn check_certificate_expiry (
		& self,
		result: & mut RequestResultSuccess,
		http_response: & http::HttpResponse,
	) -> Result <(), Box <error::Error>> {

		if let Some (certificate_expiry) =
			http_response.certificate_expiry {

			let now =
				UTC::now ().naive_utc ();

			let remaining_time =
				certificate_expiry.signed_duration_since (
					now);

			if remaining_time < ChronoDuration::days (5) {

				result.check_status.update (
					CheckStatus::Critical);

				result.messages.push (
					format! (
						"certificate expires in {} hours",
						remaining_time.num_hours ()));

			} else if remaining_time < ChronoDuration::weeks (1) {

				result.check_status.update (
					CheckStatus::Warning);

				result.messages.push (
					format! (
						"certificate expires in {} days (warning)",
						remaining_time.num_days ()));

			} else {

				result.messages.push (
					format! (
						"certificate expires in {} weeks",
						remaining_time.num_weeks ()));

			}

		}

		Ok (())

	}

	fn check_response_timing (
		& self,
		check_result_builder: & mut CheckResultBuilder,
		durations: & [Duration],
	) {

		if let Some (max_duration) =
			durations.iter ().max () {

			check_helper::check_duration_less_than (
				check_result_builder,
				& self.response_time_warning,
				& self.response_time_critical,
				& format! (
					"request took {}",
					check_helper::display_duration_long (
						& max_duration)),
				& max_duration);

		}

	}

}

enum RequestResult {
	Success (RequestResultSuccess),
	ConnectionError (String),
	Timeout,
	OtherError (String),
}

struct RequestResultSuccess {
	check_status: CheckStatus,
	duration: Duration,
	messages: Vec <String>,
}

fn parse_headers (
	header_strings: Vec <String>,
) -> Result <Vec <(String, String)>, Box <error::Error>> {

	let mut header_tuples: Vec <(String, String)> =
		vec! [];

	for header_string in header_strings.iter () {

		header_tuples.push (
			parse_header (
				header_string,
			) ?,
		);

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
