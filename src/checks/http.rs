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

	expect_status: u64,
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

		// url options

		options_spec.optopt (
			"",
			"method",
			"method to use: get (default) or post",
			"METHOD");

		options_spec.optopt (
			"",
			"path",
			"path to request, defaults to /",
			"HOURS");

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
				vec! [],			

			// response

			expect_status:
				200,

			expect_headers:
				vec! [],

			// timings

			response_time_warning:
				None,

			response_time_critical:
				None,

			timeout:
				time::Duration::new (60, 0),
				
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
			self.perform_request ());

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
	Timeout,
}

impl CheckHttpInstance {

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

		/*
		let mut curl_headers =
			easy::List::new ();

		try! (
			curl_headers.append (
				"Accept-Language: en"));

		for header in headers {

			try! (
				curl_headers.append (
					header.as_str ()));

		}

		try! (
			curl_easy.http_headers (
				curl_headers));
		*/

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

// ex: noet ts=4 filetype=rust
