extern crate getopts;
extern crate hyper;
extern crate libc;
extern crate serde_json;

use std::error;
use std::io::Read;
use std::time;

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
				arghelper::parse_string_required (
					options_matches,
					"target",
				) ?,

			request_time_warning:
				arghelper::parse_duration (
					options_matches,
					"request-time-warning",
				) ?,

			request_time_critical:
				arghelper::parse_duration (
					options_matches,
					"request-time-critical",
				) ?,

			request_timeout:
				arghelper::parse_duration_or_default (
					options_matches,
					"request-timeout",
					& time::Duration::new (60, 0),
				) ?,

		}

	},

	perform = |self, plugin_provider, check_result_builder| {

		let hyper_client =
			hyper::Client::new ();

		let hyper_response =
			hyper_client.get (
				& self.target,
			).send () ?;

		let hyper_response_bytes_result: Result <Vec <u8>, _> =
			hyper_response.bytes ().collect ();

		let hyper_response_string =
			String::from_utf8 (
				hyper_response_bytes_result ?,
			) ?;

		let check_response =
			serde_json::from_str::<GenericCheckResponse> (
				& hyper_response_string,
			) ?;

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

	},

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
