//Rust file

extern crate getopts;
extern crate curl;
extern crate time;

use getopts::Options;

use std::env;
use std::error;
use std::process;

use curl::easy;
use time::PreciseTime;

fn print_usage (program: &str, opts: Options) {
	let brief = format!("Usage: {} [options]", program);
	println!("{}", opts.usage(&brief));
}

fn print_help (program: &str, opts: Options) {
	let brief = format!("Help: {} [options]", program);
	println!("{}", opts.usage(&brief));
}

struct Opts {
	hostname: String,
	uri: String,
	text: String,
	secure: bool,
	warning: String,
	critical: String,
	timeout: String,
	headers: Vec<String>,
}

fn parse_options () -> Option<Opts> {

	let args = env::args ();

	let mut opts = Options::new();

	opts.optflag (
			"",
			"help",
			"print this help menu");

	opts.reqopt (
			"",
			"hostname",
			"Hostname of the site in which the check is performed",
			"<hostname>");

	opts.reqopt (
			"",
			"uri",
			"URI of the site in which the check is performed",
			"<url>");

	opts.reqopt (
			"",
			"text",
			"Text that will be searched in the site",
			"<text>");

	opts.reqopt (
			"",
			"ssl",
			"use https instead of http",
			"<http-enabled>");

	opts.reqopt (
			"",
			"warning-time",
			"warning is returned when the response time exceeds this threshold",
			"<warning>");

	opts.reqopt (
			"",
			"critical-time",
			"critical is returned when the response time exceeds this threshold",
			"<critical>");

	opts.reqopt (
			"",
			"timeout",
			"timeout in which the check stops waiting aborts the process",
			"<http-enabled>");

	opts.optmulti (
			"",
			"header",
			"Request header to send, such as 'Name: value'",
			"<HEADER>");

	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("check_site", opts);
			process::exit(3);
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_site", opts);
		process::exit(3);
	}

	let hostname = matches.opt_str ("hostname").unwrap ();
	let uri = matches.opt_str ("uri").unwrap ();
	let text = matches.opt_str ("text").unwrap ();
	let secure_str = matches.opt_str ("ssl").unwrap ();
	let warning = matches.opt_str ("warning-time").unwrap ();
	let critical = matches.opt_str ("critical-time").unwrap ();
	let timeout = matches.opt_str ("timeout").unwrap ();
	let headers = matches.opt_strs ("header");

	let mut secure: bool = false;
	if &secure_str == "true" {
		secure = true;
	}

	return Some (Opts {
		hostname: hostname,
		uri: uri,
		text: text,
		secure: secure,
		warning: warning,
		critical: critical,
		timeout: timeout,
		headers: headers,
	});

}

fn check_site (
	host: & str,
	uri: & str,
	text: & str,
	secure: bool,
	headers: & Vec <String>,
	warning: f64,
	critical: f64,
	timeout: f64,
) -> Result <String, Box <error::Error>> {

	let prefix =
		if secure {
			"https://".to_string ()
		} else {
			"http://".to_string ()
		};

	let url = prefix + host + uri;

	let start =
		PreciseTime::now ();

   	let mut curl_easy =
   		easy::Easy::new ();

	try! (
		curl_easy.get (
			true));

	try! (
		curl_easy.url (
			url.as_str ()));

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

	let response_body =
		try! (
			String::from_utf8 (
				response_buffer));

	// Wait for the call to finish

	let end =
		PreciseTime::now ();

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

	// Code check and text

	let result_message =
		if response_code < 200 || response_code >= 300 {

		if response_body.contains (text) {

			format! (
				"SITE-CRITICAL: {}. Text \"{}\" found.",
				response_code,
				text)

		} else {

			format! (
				"SITE-CRITICAL: {}. Text \"{}\" not found.",
				response_code,
				text)

		}

	} else {

		if response_body.contains (text) {

			format! (
				"SITE-OK: {}. Text \"{}\" found.",
				response_code,
				text)

		} else {

			format! (
				"SITE-WARNING: {}. Text \"{}\" not found.",
				response_code,
				text)

		}

	};

	// Final message including result and timeout info

	if (millis_message.contains("CRITICAL") && (result_message.contains("WARNING") || result_message.contains("OK"))) || (millis_message.contains("WARNING") && result_message.contains("OK")) {

		Ok (
			format! (
				"{}\n{} | response_time={}ms;;;;",
				millis_message,
				result_message,
				millis))

	} else {

		Ok (
			format! (
				"{}\n{} | response_time={}ms;;;;",
				result_message,
				millis_message,
				millis))

	}

}

fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => {
			println!("UNKNOWN: Wrong arguments.");
			process::exit(3);
		}
	};


	let hostname = &opts.hostname;
	let uri = &opts.uri;
	let text = &opts.text;
	let secure = opts.secure;
	let headers = opts.headers;

	let warning : f64 = match opts.warning.parse() {
		Ok (f64) => { f64 }
		Err (_) => {
			println!("UNKNOWN: Warning level must be a value between 0.0 and 1.0.");
			process::exit(3);
		}
	};

	let critical : f64 = match opts.critical.parse() {
		Ok (f64) => { f64 }
		Err (_) => {
			println!("UNKNOWN: Critical level must be a value between 0.0 and 1.0.");
			process::exit(3);
		}
	};

	let timeout : f64 = match opts.timeout.parse() {
		Ok (f64) => { f64 }
		Err (_) => {
			println!("UNKNOWN: Timeout must be a number!");
			process::exit(3);
		}
	};

	let site_res =
		check_site (
			hostname,
			uri,
			text,
			secure,
			& headers,
			warning,
			critical,
			timeout,
		).unwrap ();

	println! (
		"{}",
		site_res);

	if site_res.contains("UNKNOWN") {
		process::exit(3);
	}
	else if site_res.contains("CRITICAL") {
		process::exit(2);
	}
	else if site_res.contains("WARNING") {
		process::exit(1);
	}
	else {
		process::exit(0);
	}

}
