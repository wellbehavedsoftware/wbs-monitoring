extern crate getopts;
extern crate libc;
extern crate serde_json;

use std::error;
use std::error::Error;
use std::time;

use hyper::Uri;

use logic::*;
use lowlevel::http;
use lowlevel::http::HttpMethod;
use lowlevel::http::HttpRequest;

check! {

	new = new,
	name = "check-generic",
	prefix = "GENERIC",

	provider = CheckGenericProvider,

	instance = CheckGenericInstance {

		target: Uri,

		request_time_warning: Option <time::Duration>,
		request_time_critical: Option <time::Duration>,
		request_timeout: time::Duration,

	},

	options_spec = |options_spec| {

		// path

		options_spec.reqopt (
			"",
			"target",
			"target URL for generic check",
			"TARGET");

		// timeouts

		options_spec.optopt (
			"",
			"request-timeout",
			"status request timeout duration",
			"DURATION");

		options_spec.optopt (
			"",
			"request-time-warning",
			"status request time warning duration",
			"DURATION");

		options_spec.optopt (
			"",
			"request-time-critical",
			"status request time critical duration",
			"DURATION");

	},

	options_parse = |options_matches| {

		CheckGenericInstance {

			target:
				arg_helper::parse_string_required (
					options_matches,
					"target",
				) ?.parse () ?,

			request_time_warning:
				arg_helper::parse_duration (
					options_matches,
					"request-time-warning",
				) ?,

			request_time_critical:
				arg_helper::parse_duration (
					options_matches,
					"request-time-critical",
				) ?,

			request_timeout:
				arg_helper::parse_duration_or_default (
					options_matches,
					"request-timeout",
					& time::Duration::new (60, 0),
				) ?,

		}

	},

	perform = |self, plugin_provider, check_result_builder| {

		self.perform_request (
			& mut check_result_builder,
		) ?;

	},

}

impl CheckGenericInstance {

	fn perform_request (
		& self,
		check_result_builder: & mut CheckResultBuilder,
	) -> Result <(), Box <Error>> {

		let http_request =
			HttpRequest {

			address: self.target.host ().unwrap (),
			hostname: self.target.host ().unwrap (),

			port: self.target.port ().unwrap_or (
				match self.target.scheme ().unwrap () {
					"http" => 80,
					"https" => 443,
					_ => panic! ("Invalid scheme"),
				}
			) as u64,

			secure: match self.target.scheme ().unwrap () {
				"http" => false,
				"https" => true,
				_ => panic! ("Invalid scheme"),
			},

			headers: & Vec::new (),

			method: HttpMethod::Get,
			path: self.target.path (),

			timeout: self.request_timeout,

		};

		let http_response = match

			http::perform_request (
				& http_request)

		{

			http::PerformRequestResult::Success (http_response) =>
				Ok (http_response),

			http::PerformRequestResult::Failure (reason) =>
				Err (
					format! (
						"failed to connect: {}",
						reason)),

			http::PerformRequestResult::Timeout (_duration) =>
				Err (
					format! (
						"Request timed out")),

		} ?;

		check_helper::check_duration_less_than (
			check_result_builder,
			& self.request_time_warning,
			& self.request_time_critical,
			& format! (
				"Request took {}",
				check_helper::display_duration_short (
					& http_response.duration ())),
			& http_response.duration ());

		let hyper_response_string =
			match http_response.body_string () {

			Ok (value) => value,

			Err (error) => {

				check_result_builder.critical (
					format! (
						"Error decoding result as UTF-8 string: {}",
						error));

				return Ok (());

			},

		};

		let check_response =
			match serde_json::from_str::<GenericCheckResponse> (
				& hyper_response_string,
			) {

			Ok (value) => value,

			Err (error) => {

				check_result_builder.critical (
					format! (
						"Error decoding JSON structure: {}",
						error.description ()));

				return Ok (());

			},

		};

		match check_response.status.as_str () {

			"ok" =>
				check_result_builder.ok (
					check_response.status_message),

			"warning" =>
				check_result_builder.warning (
					check_response.status_message),

			"critical" =>
				check_result_builder.critical (
					check_response.status_message),

			"unknown" =>
				check_result_builder.unknown (
					check_response.status_message),

			_ =>
				check_result_builder.unknown (
					format! (
						"Invalid check result status: {}",
						check_response.status)),

		}

		for additional_message in check_response.additional_messages {

			check_result_builder.extra_information (
				additional_message);

		}

		Ok (())

	}

}

#[ derive (Serialize, Deserialize) ]
struct GenericCheckResponse {

	#[ serde (rename = "status") ]
	status: String,

	#[ serde (rename = "status-message") ]
	status_message: String,

	#[ serde (rename = "additional-messages") ]
	additional_messages: Vec <String>,

}

// ex: noet ts=4 filetype=rust
