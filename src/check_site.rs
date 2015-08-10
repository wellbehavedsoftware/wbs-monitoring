//Rust file

extern crate getopts;
extern crate curl;

use getopts::Options;
use std::env;
use std::option::{ Option };
use std::process;
use curl::http;

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

	opts.optmulti (
			"",
			"header",
			"Request header to send, such as 'Name: value'",
			"HEADER");

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
		headers: headers,
	});

}

fn get_response (host: &str, uri: &str, secure: bool, headers: &Vec<String>) -> String {

	let mut list_command =
		process::Command::new ("/usr/lib/nagios/plugins/check_http");

	list_command
		.arg ("-H".to_string ())
		.arg (host.to_string ())
		.arg ("-u".to_string ())
		.arg (uri.to_string ());

	if secure {

		list_command
			.arg ("--ssl".to_string ());

	}

	for header in headers {

		list_command
			.arg ("--header")
			.arg (header);

	}

	let list_output = match list_command.output () {
		Ok (output) => { output }
		Err (err) => { return format!("SITE-UNKNOWN: Error while checking the site: {}", err); }
	};

	let response = String::from_utf8_lossy(&list_output.stdout).to_string();

	let response_vector: Vec<&str> = response.split(' ').collect();

	let informational = 	vec![100isize, 101, 102];
	let success = 		vec![200isize, 201, 202, 203, 204, 205, 206, 208, 226];
	let redirection = 	vec![300isize, 301, 302, 303, 304, 305, 306, 308];
	let client_error = 	vec![400isize, 401, 402, 403, 404, 405, 406, 407, 408, 409, 410, 
				411, 412, 413, 414, 415, 416, 417, 418, 419, 420, 422,
				423, 424, 426, 428, 429, 431, 440, 444, 450, 451, 494, 
				495, 496, 497, 498, 499];
	let server_error =	vec![500isize, 501, 502, 503, 504, 505, 506, 507, 508, 509, 510,
				511, 520, 521, 522, 523, 524, 598, 599];

	let response_code : isize = match response_vector[3].parse() {
		Ok (isize) => { isize }
		Err (_) => { 
			return "SITE-UNKNOWN: The check could not be performed. No response received.".to_string(); 
		}
	};
	

	if in_array(response_code, informational) {
		return format!("SITE-UNKNOWN: {}", response);
	}
	else if in_array(response_code, success) {
		return format!("SITE-OK: {}", response);
	}
	else if in_array(response_code, redirection) {
		return format!("SITE-WARNING: {}", response);
	}
	else if in_array(response_code, client_error) || in_array(response_code, server_error) {
		return format!("SITE-CRITICAL: {}", response);
	}
	else {
		return "SITE-UNKNOWN: Check_response failed.".to_string();
	}
	
}

fn in_array (element: isize, haystack: Vec<isize>) -> bool {

	let mut found = false;

	for &code in haystack.iter() {

		if code == element {
			found = true;
		}

	}

	return found;
}

fn check_text (host: &str, uri: &str, text: &str, secure: bool, headers: &Vec<String>) -> String {
    
	let mut prefix: String;

	if secure { prefix = "https://".to_string(); }
	else { prefix = "http://".to_string(); }

	let url = prefix + host + uri;
	
   	let mut http_handle = http::handle();

	let mut http_request = http_handle.get (url);

	http_request = http_request.header("Accept-Language", "en");

	for header in headers {
		let header_parts: Vec<&str> = header.splitn (2, ": ").collect ();
		http_request = http_request.header (header_parts [0], header_parts [1]);
	}

	let resp = http_request.exec().unwrap();

	let url_code = String::from_utf8_lossy(resp.get_body());

	if url_code.contains(text)
	/* && url_code.contains("<body") && url_code.contains("</body>") 
		&& url_code.contains("<head") && url_code.contains("</head>") && url_code.contains("<html") 
		&& url_code.contains("</html>")*/ { 
	
		return "SITE-OK: Correct text found.".to_string();

	} else {

		return "SITE-WARNING: Text not found.".to_string(); 

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

	let response_res = get_response(hostname, uri, secure, & headers);
	let text_res = check_text(hostname, uri, text, secure, & headers);


	if response_res.contains("UNKNOWN") || text_res.contains("UNKNOWN") {

		println!("{}\n{}", response_res, text_res);
		process::exit(3);

	}
	else if response_res.contains("CRITICAL") {

		println!("{}\n{}", response_res, text_res);
		process::exit(2);

	} else if response_res.contains("WARNING") {

		println!("{}\n{}", response_res, text_res);
		process::exit(1);

	} else if text_res.contains("WARNING") {

		println!("{}\n{}", text_res, response_res);
		process::exit(1);

	} else {

		println!("{}\n{}", response_res, text_res);
		process::exit(0);

	}

}
