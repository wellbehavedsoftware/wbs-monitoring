extern crate getopts;
extern crate curl;
extern crate time;

use std::env;
use std::error;
use std::process;
use std::thread;

fn print_usage (program: &str, opts: getopts::Options) {
	let brief = format!("Usage: {} [options]", program);
	println!("{}", opts.usage(&brief));
}

fn print_help (program: &str, opts: getopts::Options) {
	let brief = format!("Help: {} [options]", program);
	println!("{}", opts.usage(&brief));
}

struct Opts {
	hostname: String,
	uri: String,
	secure: bool,
	warning: String,
	critical: String,
	timeout: String,
	headers: Vec<String>,
}

fn parse_options () -> Option<Opts> {

	let args = env::args ();

	let mut opts = getopts::Options::new();

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
			"HEADER");

	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("check_etcd", opts);
			process::exit(3);
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_etcd", opts);
		process::exit(3);
	}

	let hostname = matches.opt_str ("hostname").unwrap ();
	let uri = matches.opt_str ("uri").unwrap ();
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
		secure: secure,
		warning: warning,
		critical: critical,
		timeout: timeout,
		headers: headers,
	});

}

fn check_etcd (
	host: & str,
	uri: & str,
	secure: bool,
	headers: & Vec <String>,
	warning: f64,
	critical: f64,
	timeout: f64,
) -> Result <String, Box <error::Error>> {

	let url =
		"http://10.109.160.17:2380/metrics";

	let start =
		time::PreciseTime::now ();

	let mut curl_easy =
		curl::easy::Easy::new ();

	try! (
		curl_easy.get (
			true));

	try! (
		curl_easy.url (
			url));

	let mut curl_headers =
		curl::easy::List::new ();

	try! (
		curl_headers.append (
			"Accept-Language: en"));

	try! (
		curl_headers.append (
			"User-Agent: check-github (wbs-monitoring)"));

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

	let end =
		time::PreciseTime::now ();

	let millis =
		start.to (end).num_milliseconds () as f64;

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

	println! (
		"{}",
		response_body);

/*
	// Code check and text
	let informational = 	vec![100isize, 101, 102];
	let success = 		vec![200isize, 201, 202, 203, 204, 205, 206, 208, 226];
	let redirection = 	vec![300isize, 301, 302, 303, 304, 305, 306, 308];
	let client_error = 	vec![400isize, 401, 402, 403, 404, 405, 406, 407, 408, 409, 410,
				411, 412, 413, 414, 415, 416, 417, 418, 419, 420, 422,
				423, 424, 426, 428, 429, 431, 440, 444, 450, 451, 494,
				495, 496, 497, 498, 499];
	let server_error =	vec![500isize, 501, 502, 503, 504, 505, 506, 507, 508, 509, 510,
				511, 520, 521, 522, 523, 524, 598, 599];



	let response_code : isize = match code_string.to_string().parse() {
		Ok (isize) => { isize }
		Err (_) => {
			return "SITE-UNKNOWN: The check could not be performed. No response received.".to_string();
		}
	};

	let mut result_message: String = "".to_string();

	if informational.contains(&response_code) {
		if url_code.contains(text) {
			result_message = format!("SITE-UNKNOWN: {}. Text \"{}\" found.", code_string.clone(), text);
		}
		else {
			result_message = format!("SITE-UNKNOWN: {}. Text \"{}\" not found.", code_string.clone(), text);
		}
	}
	else if success.contains(&response_code) {
		if url_code.contains(text) {
			result_message = format!("SITE-OK: {}. Text \"{}\" found.", code_string.clone(), text);
		}
		else {
			result_message = format!("SITE-WARNING: {}. Text \"{}\" not found.", code_string.clone(), text);
		}
	}
	else if redirection.contains(&response_code) {
		if url_code.contains(text) {
			result_message = format!("SITE-WARNING: {}. Text \"{}\" found.", code_string.clone(), text);
		}
		else {
			result_message = format!("SITE-WARNING: {}. Text \"{}\" not found.", code_string.clone(), text);
		}
	}
	else if client_error.contains(&response_code) || server_error.contains(&response_code) {
		if url_code.contains(text) {
			result_message = format!("SITE-CRITICAL: {}. Text \"{}\" found.", code_string.clone(), text);
		}
		else {
			result_message = format!("SITE-CRITICAL: {}. Text \"{}\" not found.", code_string.clone(), text);
		}
	}
	else {
		result_message = "SITE-UNKNOWN: check_site failed.".to_string();
	}

	// Final message including result and timeout info

	if (millis_message.contains("CRITICAL") && (result_message.contains("WARNING") || result_message.contains("OK"))) || (millis_message.contains("WARNING") && result_message.contains("OK")) {
		return format!("{}\n{} | response_time={}ms;;;;", millis_message, result_message, millis);
	}
	else {
		return format!("{}\n{} | response_time={}ms;;;;", result_message, millis_message, millis);
	}*/

	Ok ("OK".to_string ())

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
			println!("UNKNOWN: Critical level must be a value between 0.0 and 1.0.");
			process::exit(3);
		}
	};

	let etcd_res =
		check_etcd (
			hostname,
			uri,
			secure,
			& headers,
			warning,
			critical,
			timeout,
		).unwrap ();

	println! ("{}", etcd_res);

	if etcd_res.contains("UNKNOWN") {

		process::exit(3);

	}
	else if etcd_res.contains("CRITICAL") {

		process::exit(2);

	}
	else if etcd_res.contains("WARNING") {

		process::exit(1);

	}
	else {

		process::exit(0);

	}

}
