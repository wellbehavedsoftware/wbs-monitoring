extern crate getopts;
extern crate hyper;
extern crate libc;
extern crate serde_json;

use std::error;
use std::error::Error;
use std::io::Read;
use std::time;
use std::time::Instant;

use logic::*;

check! {

	new = new,
	name = "check-generic",
	prefix = "GENERIC",

	provider = CheckGenericProvider,

	instance = CheckGenericInstance {

		target: String,

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
				) ?,

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

		let mut hyper_client =
			hyper::Client::new ();

		hyper_client.set_write_timeout (
			Some (self.request_timeout));

		hyper_client.set_read_timeout (
			Some (self.request_timeout));

		let start_time =
			Instant::now ();

		let hyper_response =
			match hyper_client.get (
				& self.target,
			).send () {

			Ok (value) => value,

			Err (hyper::Error::Io (io_error)) => {

				check_result_builder.critical (
					format! (
						"Connection IO error: {}",
						io_error.description ()));

				return Ok (());

			},

			Err (error) => {

				check_result_builder.unknown (
					format! (
						"Unknown connection error: {}",
						error.description ()));

				return Ok (());

			},

		};

		let hyper_response_bytes_result: Result <Vec <u8>, _> =
			hyper_response.bytes ().collect ();

		let end_time =
			Instant::now ();

		let request_duration =
			end_time - start_time;

		check_helper::check_duration_less_than (
			check_result_builder,
			& self.request_time_warning,
			& self.request_time_critical,
			& format! (
				"Request took {}",
				check_helper::display_duration_short (
					& request_duration)),
			& request_duration);

		let hyper_response_string =
			match String::from_utf8 (
				hyper_response_bytes_result ?,
			) {

			Ok (value) => value,

			Err (error) => {

				check_result_builder.critical (
					format! (
						"Error decoding result as UTF-8 string: {}",
						error.description ()));

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
