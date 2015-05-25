//Rust file
extern crate getopts;

use getopts::Options;
use std::env;
use std::option::{ Option };
use std::process;

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
	secure: bool,
}

fn parse_options () -> Option<Opts> {

	let args = env::args ();

	let mut opts = Options::new();

	opts.optflag (	
			"h",
			"help",
			"print this help menu");

	opts.reqopt (
			"s",
			"ssl",
			"use https instead of http",
			"<http-enabled>");

	opts.reqopt (
			"H",
			"host-name",
			"name of the host",
			"<host-name>");

	opts.reqopt (
			"u",
			"uri",
			"uri where the request is sent",
			"<uri>");

	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("check_response", opts);
			process::exit(3);
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_response", opts);
		process::exit(3);
	}

	let hostname = matches.opt_str ("host-name").unwrap ();
	let uri = matches.opt_str ("uri").unwrap ();
	let secure_str = matches.opt_str ("ssl").unwrap ();

	let mut secure: bool = false;
	if &secure_str == "true" {
		secure = true;
	}

	return Some (Opts {
		hostname: hostname,
		uri: uri,
		secure: secure,
	});

}

fn get_response (opts: Opts) -> String {

	if opts.secure {

		let list_output =
			match process::Command::new ("/usr/lib/nagios/plugins/check_http")
				.arg ("-H".to_string ())
				.arg (opts.hostname.to_string ())
				.arg ("-u".to_string ())
				.arg (opts.uri.to_string ())
				.arg ("--ssl".to_string())
				.output () {
			Ok (output) => { output }
			Err (err) => { panic! ("Error calling check_http: {}", err) }
		};

		String::from_utf8_lossy(&list_output.stdout).to_string()
	}

	else {
	
		let list_output =
			match process::Command::new ("/usr/lib/nagios/plugins/check_http")
				.arg ("-H".to_string ())
				.arg (opts.hostname.to_string ())
				.arg ("-u".to_string ())
				.arg (opts.uri.to_string ())
				.output () {
			Ok (output) => { output }
			Err (err) => { panic! ("Error calling check_http: {}", err) }
		};

		String::from_utf8_lossy(&list_output.stdout).to_string()

	}

	
}

fn in_array (element: isize, haystack: Vec<isize>) -> bool {

	let mut found = false;

	for &code in haystack.iter() {

		if code == element {
			found = true;
		}

	}

	found
}

fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => { return }
	};

	let response = get_response (opts);
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
			println!("UNKNOWN: The check could not be performed. No response received."); 
			process::exit(3);
		}
	};
	

	if in_array(response_code, informational) {
		println!("{}", response);
		process::exit(3);
	}
	else if in_array(response_code, success) {
		println!("{}", response);
		process::exit(0);
	}
	else if in_array(response_code, redirection) {
		println!("{}", response);
		process::exit(1);
	}
	else if in_array(response_code, client_error) || in_array(response_code, server_error) {
		println!("{}", response);
		process::exit(2);
	}
	else {
		println!("UNKNOWN: Check_response failed.");
		process::exit(3);
	}

}



