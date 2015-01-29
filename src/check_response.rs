#![allow(unstable)]
extern crate getopts;

use getopts::{ optflag, reqopt, getopts, short_usage, usage, OptGroup };
use std::os;
use std::option::{ Option };
use std::io::{ Command };

fn print_usage (program: &str, opts: &[OptGroup]) {
	println! ("{}", short_usage (program, opts));
}

fn print_help (program: &str, opts: &[OptGroup]) {
	println! ("{}", usage (program, opts));
}

struct Options {
	hostname: String,
	uri: String,
	secure: bool,
}

fn parse_options () -> Option<Options> {

	let args: Vec<String> = os::args ();

	let program = args [0].clone ();

	let opts = &[

		optflag (
			"h",
			"help",
			"print this help menu"),

		reqopt (
			"s",
			"ssl",
			"use https instead of http",
			"<http-enabled>"),

		reqopt (
			"H",
			"host-name",
			"name of the host",
			"<host-name>"),

		reqopt (
			"u",
			"uri",
			"uri where the request is sent",
			"<uri>"),

	];

	let matches = match getopts (args.tail (), opts) {
		Ok (m) => { m }
		Err (_) => {
			print_usage (program.as_slice (), opts);
			os::set_exit_status(3);	
			return None;
		}
	};

	if matches.opt_present ("help") {
		print_help (program.as_slice (), opts);
		os::set_exit_status(3);	
		return None;
	}

	if ! matches.free.is_empty () {
		print_usage (program.as_slice (), opts);
		os::set_exit_status(3);	
		return None;
	}

	
	let hostname = matches.opt_str ("host-name").unwrap ();
	let uri = matches.opt_str ("uri").unwrap ();
	let secure_str = matches.opt_str ("ssl").unwrap ();

	let mut secure: bool = false;
	if secure_str.as_slice() == "true" {
		secure = true;
	}

	return Some (Options {
		hostname: hostname,
		uri: uri,
		secure: secure,
	});

}

fn get_response (opts: Options) -> String {

	if opts.secure {

		let list_output =
			match Command::new ("/usr/lib/nagios/plugins/check_http")
				.arg ("-H".to_string ())
				.arg (opts.hostname.to_string ())
				.arg ("-u".to_string ())
				.arg (opts.uri.to_string ())
				.arg ("--ssl".to_string())
				.output () {
			Ok (output) => { output }
			Err (err) => { panic! ("Error calling check_http: {}", err) }
		};

		String::from_utf8_lossy(list_output.output.as_slice()).to_string()
	}

	else {
	
		let list_output =
			match Command::new ("/usr/lib/nagios/plugins/check_http")
				.arg ("-H".to_string ())
				.arg (opts.hostname.to_string ())
				.arg ("-u".to_string ())
				.arg (opts.uri.to_string ())
				.output () {
			Ok (output) => { output }
			Err (err) => { panic! ("Error calling check_http: {}", err) }
		};

		String::from_utf8_lossy(list_output.output.as_slice()).to_string()

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
	let response_vector: Vec<&str> = response.as_slice().split(' ').collect();

	let informational = 	vec![100is, 101, 102];
	let success = 		vec![200is, 201, 202, 203, 204, 205, 206, 208, 226];
	let redirection = 	vec![300is, 301, 302, 303, 304, 305, 306, 308];
	let client_error = 	vec![400is, 401, 402, 403, 404, 405, 406, 407, 408, 409, 410, 
				411, 412, 413, 414, 415, 416, 417, 418, 419, 420, 422,
				423, 424, 426, 428, 429, 431, 440, 444, 450, 451, 494, 
				495, 496, 497, 498, 499];
	let server_error =	vec![500is, 501, 502, 503, 504, 505, 506, 507, 508, 509, 510,
				511, 520, 521, 522, 523, 524, 598, 599];

	
	let aux: Option<isize> = response_vector[3].parse();
	if aux.is_none() { panic! ("The check could not be performed. No response received."); }
	let response_code: isize = aux.unwrap();
	

	if in_array(response_code, informational) {
		os::set_exit_status(3);
		println!("{}", response);
	}
	else if in_array(response_code, success) {
		os::set_exit_status(0);
		println!("{}", response);
	}
	else if in_array(response_code, redirection) {
		os::set_exit_status(1);
		println!("{}", response);
	}
	else if in_array(response_code, client_error) || in_array(response_code, server_error) {
		os::set_exit_status(2);
		println!("{}", response);
	}
	else {
		os::set_exit_status(3);
		println!("UNKNOWN: Check_response failed.");
	}

	return;
}



