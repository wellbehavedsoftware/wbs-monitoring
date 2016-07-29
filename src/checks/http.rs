extern crate getopts;
extern crate curl;

use std::error;
use std::time;

use logic::*;

pub fn new (
) -> Box <PluginProvider> {

	Box::new (
		CheckHttpProvider {},
	)

}

struct CheckHttpProvider {
}

enum HttpMethod {
	Get,
	Post,
}

struct CheckHttpInstance {

	address: String,
	port: u64,
	secure: bool,

	method: HttpMethod,
	path: String,

	send_headers: Vec <(String, String)>,

	expect_status_code: Vec <u64>,
	expect_headers: Vec <(String, String)>,

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
			"HOURS");

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
				HttpMethod::Get,

			path:
				"/".to_string (),

			send_headers:
				try! (
					parse_headers (
						try! (
							arghelper::parse_string_multiple (
								options_matches,
								"send-header")))),

			// response

			expect_status_code:
				vec! [ 200 ],

			expect_headers:
				vec! [],

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

struct HttpResponse {
	status_code: u64,
	//status_message: String,
	body: String,
	duration: time::Duration,
}

enum PerformRequestResult {
	Success (HttpResponse),
	Timeout (time::Duration),
}

impl CheckHttpInstance {

	fn check_http (
		& self,
		check_result_builder: & mut CheckResultBuilder,
	) -> Result <(), Box <error::Error>> {

		match try! (
			self.perform_request ()) {

			PerformRequestResult::Success (http_response) =>
				try! (
					self.process_response (
						check_result_builder,
						& http_response)),

			PerformRequestResult::Timeout (duration) =>
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
		http_response: & HttpResponse,
	) -> Result <(), Box <error::Error>> {

		try! (
			self.check_response_content (
				check_result_builder,
				http_response));

		try! (
			self.check_response_timing (
				check_result_builder,
				http_response));

		Ok (())

	}

	fn check_response_content (
		& self,
		check_result_builder: & mut CheckResultBuilder,
		http_response: & HttpResponse,
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

	fn check_response_timing (
		& self,
		check_result_builder: & mut CheckResultBuilder,
		http_response: & HttpResponse,
	) -> Result <(), Box <error::Error>> {

		try! (
			checkhelper::check_duration_less_than (
				check_result_builder,
				& self.response_time_warning,
				& self.response_time_critical,
				& format! (
					"request took {}",
					checkhelper::display_duration_long (
						& http_response.duration)),
				& http_response.duration));

		Ok (())

	}

	fn perform_request (
		& self,
	) -> Result <PerformRequestResult, Box <error::Error>> {

		// setup request

		let url =
			format! (
				"{}://{}:{}{}",
				if self.secure { "https" } else { "http" },
				self.address,
				self.port,
				self.path);

	   	let mut curl_easy =
	   		curl::easy::Easy::new ();

		try! (
			curl_easy.get (
				true));

		try! (
			curl_easy.url (
				url.as_str ()));

		try! (
			curl_easy.timeout (
				self.timeout));

		// setup request headers

		let mut curl_headers =
			curl::easy::List::new ();

		for & (ref header_name, ref header_value)
			in self.send_headers.iter () {

			try! (
				curl_headers.append (
					& format! (
						"{}: {}",
						header_name,
						header_value)));

		}

		try! (
			curl_easy.http_headers (
				curl_headers));

		// perform request

		let mut response_buffer: Vec <u8> =
			vec! [];

		let start_time =
			time::Instant::now ();

		{

			let mut curl_transfer =
				curl_easy.transfer ();

			try! (
				curl_transfer.write_function (
					|data| {

				response_buffer.extend_from_slice (
					data);

				Ok (data.len ())

			}));

			try! (
				curl_transfer.perform ());

		}

		let end_time =
			time::Instant::now ();

		let duration =
			end_time.duration_since (
				start_time);

		// process response

		let response_status_code =
			try! (
				curl_easy.response_code ()
			) as u64;

		println! (
			"response status code: {}",
			response_status_code);

		let response_body =
			try! (
				String::from_utf8 (
					response_buffer));

		println! (
			"response body: {}",
			response_body);

		println! (
			"response duration: {:?}",
			duration);

		/*
		let millis = start.to(end).num_milliseconds() as f64;
		let mut millis_message = "".to_string();

		if millis <= warning {

			millis_message =
				format! (
					"TIMEOUT-OK: The request took {} milliseconds.",
					millis);

		} else if millis > warning && millis <= critical {

			millis_message =
				format! (
					"TIMEOUT-WARNING: The request took {} milliseconds.",
					millis);

		} else if millis > critical && millis <= timeout {

			millis_message =
				format! (
					"TIMEOUT-CRITICAL: The request took {} milliseconds.",
					millis);

		} else if millis > timeout {

			return Ok (
				format! (
					"TIMEOUT-CRITICAL: The timed out at {} milliseconds.",
					millis));

		}
		*/

		Ok (
			PerformRequestResult::Success (

			HttpResponse {
				status_code: response_status_code,
				body: response_body,
				duration: duration,
			}

		))

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
