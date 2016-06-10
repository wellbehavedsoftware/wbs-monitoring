extern crate curl;
extern crate xml;

use getopts;

use std::error;
use std::time;

use logic::*;

macro_rules! xml_child_element {

	(
		$check_result_builder: expr,
		$parent_element: expr,
		$child_element_name: expr
	) => {

		match $parent_element.get_child (
			$child_element_name,
			None) {

			Some (value) =>
				value,

			None => {

				$check_result_builder.unknown (
					format! (
						"can't find <{}> element in <{}>",
						$child_element_name,
						$parent_element.name));

				return Ok (None);

			},

		}

	};

}

macro_rules! xml_child_element_u64 {

	(
		$check_result_builder: expr,
		$parent_element: expr,
		$child_element_name: expr
	) => {

		{

			let child_element =
				xml_child_element! (
					$check_result_builder,
					$parent_element,
					$child_element_name);

			let content_string =
				child_element.content_str ();

			match content_string.parse::<u64> () {

				Ok (value) =>
					value,

				Err (_) => {

					$check_result_builder.unknown (
						format! (
							"unable to interpret numerical value for <{}>",
							$child_element_name));

					return Ok (None);

				}

			}

		}

	}

}


pub fn new (
) -> Box <PluginProvider> {

	Box::new (
		CheckAuthsmtpProvider {},
	)

}

struct CheckAuthsmtpProvider {
}

struct CheckAuthsmtpInstance {

	data_usage_warning: Option <f64>,
	data_usage_critical: Option <f64>,

	message_usage_warning: Option <f64>,
	message_usage_critical: Option <f64>,

	api_username: String,
	api_password: String,

}

struct BasicUserResult {
	messages_limit: u64,
	data_limit: u64,
	messages_sent: u64,
	data_sent: u64,
}

impl PluginProvider
for CheckAuthsmtpProvider {

	fn name (
		& self,
	) -> & str {
		"check-authsmtp"
	}

	fn prefix (
		& self,
	) -> & str {
		"AUTHSMTP"
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

		options_spec.optopt (
			"",
			"data-usage-warning",
			"data usage warning level",
			"FRACTION");

		options_spec.optopt (
			"",
			"data-usage-critical",
			"data usage critical level",
			"FRACTION");

		options_spec.optopt (
			"",
			"message-usage-warning",
			"message usage warning level",
			"FRACTION");

		options_spec.optopt (
			"",
			"message-usage-critical",
			"message usage critical level",
			"FRACTION");

		options_spec.reqopt (
			"",
			"api-username",
			"AuthSMTP API username",
			"USER");

		options_spec.reqopt (
			"",
			"api-password",
			"AuthSMTP API password",
			"PASS");

		options_spec

	}

	fn new_instance (
		& self,
		_options_spec: & getopts::Options,
		options_matches: & getopts::Matches,
	) -> Result <Box <PluginInstance>, Box <error::Error>> {

		// data usage

		let data_usage_warning =
			try! (
				arghelper::parse_decimal_fraction (
					options_matches,
					"data-usage-warning"));

		let data_usage_critical =
			try! (
				arghelper::parse_decimal_fraction (
					options_matches,
					"data-usage-critical"));

		// message usage

		let message_usage_warning =
			try! (
				arghelper::parse_decimal_fraction (
					options_matches,
					"message-usage-warning"));

		let message_usage_critical =
			try! (
				arghelper::parse_decimal_fraction (
					options_matches,
					"message-usage-critical"));

		// api credentials

		let api_username =
			options_matches.opt_str (
				"api-username",
			).unwrap ();

		let api_password =
			options_matches.opt_str (
				"api-password",
			).unwrap ();

		return Ok (Box::new (

			CheckAuthsmtpInstance {

				data_usage_warning: data_usage_warning,
				data_usage_critical: data_usage_critical,

				message_usage_warning: message_usage_warning,
				message_usage_critical: message_usage_critical,

				api_username: api_username,
				api_password: api_password,

			}

		));

	}

}

impl PluginInstance
for CheckAuthsmtpInstance {

	fn perform_check (
		& self,
		plugin_provider: & PluginProvider,
	) -> Result <CheckResult, Box <error::Error>> {

		let mut check_result_builder =
			CheckResultBuilder::new ();

		let (result_string, result_element) =
			try! (
				self.make_api_call ());

		let basic_user_result_option =
			try! (
				self.interpret_result (
					& mut check_result_builder,
					& result_element));

		if basic_user_result_option.is_some () {

			let basic_user_result =
				basic_user_result_option.unwrap ();

			try! (
				self.check_messages_result (
					& mut check_result_builder,
					& basic_user_result));

			try! (
				self.check_data_result (
					& mut check_result_builder,
					& basic_user_result));

		}

		check_result_builder.extra_information (
			result_string);

		let check_result =
			check_result_builder.into_check_result (
				plugin_provider);

		Ok (check_result)

	}

}

impl CheckAuthsmtpInstance {

	fn make_api_call (
		& self,
	) -> Result <(String, xml::Element), Box <error::Error>> {

		// set up the http call

		let mut curl_easy =
			curl::easy::Easy::new ();

		try! (
			curl_easy.url (
				& format! (
					"https://secure.authsmtp.com/restful/basic_user/{}",
					self.api_username)));

		try! (
			curl_easy.connect_timeout (
				time::Duration::from_secs (3)));

		try! (
			curl_easy.follow_location (
				true));

		try! (
			curl_easy.username (
				self.api_username.as_str ()));

		try! (
			curl_easy.password (
				self.api_password.as_str ()));

		try! (
			curl_easy.get (
				true));

		// make the call

		let mut response_buffer: Vec <u8> =
			vec! [];

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

		let response_code =
			try! (
				curl_easy.response_code ());

		if response_code != 200 {

			return Err (
				Box::new (
					SimpleError::from (
						format! (
							"server returned {}",
							response_code,
						).to_string ())));

		}

		// parse the response body

		let mut xml_parser =
			xml::Parser::new ();

		let response_body =
			try! (
				String::from_utf8 (
					response_buffer));

		xml_parser.feed_str (
			response_body.as_str ());

		let mut xml_builder =
			xml::ElementBuilder::new ();

		for xml_result in xml_parser.filter_map (
			|xml_event|

			xml_builder.handle_event (
				xml_event)) {

			return Ok (
				(
					response_body,
					try! (
						xml_result),
				)
			);

		}

		Err (
			Box::new (
				SimpleError::from (
					format! (
						"don't understand server response: {}",
						response_body,
					).to_string ())))

	}

	fn interpret_result (
		& self,
		check_result_builder: & mut CheckResultBuilder,
		result_element: & xml::Element,
	) -> Result <Option <BasicUserResult>, Box <error::Error>> {

		if result_element.name != "result" {

			check_result_builder.unknown (
				format! (
					"got element <{}>, expected <result>",
					result_element.name));

			return Ok (None);

		}

		let basic_user_element =
			xml_child_element! (
				check_result_builder,
				result_element,
				"basic_user");

		Ok (Some (

			BasicUserResult {

				messages_limit:
					xml_child_element_u64! (
						check_result_builder,
						basic_user_element,
						"messages_limit"),

				data_limit:
					xml_child_element_u64! (
						check_result_builder,
						basic_user_element,
						"data_limit"),

				messages_sent:
					xml_child_element_u64! (
						check_result_builder,
						basic_user_element,
						"messages_sent"),

				data_sent:
					xml_child_element_u64! (
						check_result_builder,
						basic_user_element,
						"data_sent"),

			}

		))

	}

	fn check_messages_result (
		& self,
		check_result_builder: & mut CheckResultBuilder,
		basic_user_result: & BasicUserResult,
	) -> Result <(), Box <error::Error>> {

		let message_usage_value =
			basic_user_result.messages_sent as f64
				/ basic_user_result.messages_limit as f64;

		if

			self.message_usage_critical.is_some ()

			&& message_usage_value
				> self.message_usage_critical.unwrap ()

		{

			check_result_builder.critical (
				format! (
					"messages {} of {} or {}% (critical is {}%)",
					basic_user_result.messages_sent,
					basic_user_result.messages_limit,
					(message_usage_value * 100.0) as u64,
					(self.message_usage_critical.unwrap () * 100.0) as u64));

		} else if

			self.message_usage_warning.is_some ()

			&& message_usage_value
				> self.message_usage_warning.unwrap ()

		{

			check_result_builder.warning (
				format! (
					"messages {} of {} or {}% (warning is {}%)",
					basic_user_result.messages_sent,
					basic_user_result.messages_limit,
					(message_usage_value * 100.0) as u64,
					(self.message_usage_warning.unwrap () * 100.0) as u64));

		} else {

			check_result_builder.ok (
				format! (
					"messages {} of {} or {}%",
					basic_user_result.messages_sent,
					basic_user_result.messages_limit,
					(message_usage_value * 100.0) as u64));

		}

		Ok (())

	}

	fn check_data_result (
		& self,
		check_result_builder: & mut CheckResultBuilder,
		basic_user_result: & BasicUserResult,
	) -> Result <(), Box <error::Error>> {

		let data_usage_value =
			basic_user_result.data_sent as f64
				/ basic_user_result.data_limit as f64;

		if

			self.data_usage_critical.is_some ()

			&& data_usage_value
				> self.data_usage_critical.unwrap ()

		{

			check_result_builder.critical (
				format! (
					"data {} of {} mb or {}% (critical is {}%)",
					basic_user_result.data_sent / 1024 / 1024,
					basic_user_result.data_limit / 1024 / 1024,
					(data_usage_value * 100.0) as u64,
					(self.data_usage_critical.unwrap () * 100.0) as u64));

		} else if

			self.data_usage_warning.is_some ()

			&& data_usage_value
				> self.data_usage_warning.unwrap ()

		{

			check_result_builder.warning (
				format! (
					"data {} of {} mb or {}% (warning is {}%)",
					basic_user_result.data_sent / 1024 / 1024,
					basic_user_result.data_limit / 1024 / 1024,
					(data_usage_value * 100.0) as u64,
					(self.data_usage_warning.unwrap () * 100.0) as u64));

		} else {

			check_result_builder.ok (
				format! (
					"data {} of {} mb or {}%",
					basic_user_result.data_sent / 1024 / 1024,
					basic_user_result.data_limit / 1024 / 1024,
					(data_usage_value * 100.0) as u64));

		}

		Ok (())

	}

}

// ex: noet ts=4 filetype=rust
