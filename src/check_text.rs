#![allow(unstable)]
extern crate getopts;
extern crate curl;

use getopts::{ optflag, reqopt, getopts, short_usage, usage, OptGroup };
use std::os;
use std::option::{ Option };
use curl::http;

fn print_usage (program: &str, opts: &[OptGroup]) {
	println! ("{}", short_usage (program, opts));
}

fn print_help (program: &str, opts: &[OptGroup]) {
	println! ("{}", usage (program, opts));
}

struct Options {
	hostname: String,
	uri: String,
	text: String,
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
			"t",
			"text",
			"the text that is going to be searched",
			"<text-to-check>"),

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
			return None;
		}
	};

	if matches.opt_present ("help") {
		print_help (program.as_slice (), opts);
		return None;
	}

	if ! matches.free.is_empty () {
		print_usage (program.as_slice (), opts);
		return None;
	}

	
	let hostname = matches.opt_str ("host-name").unwrap ();
	let uri = matches.opt_str ("uri").unwrap ();
	let text = matches.opt_str ("text").unwrap ();

	return Some (Options {
		hostname: hostname,
		uri: uri,
		text: text,
	});

}

fn check_text (host: &str, uri: &str, text: &str) -> String {
    
	let url = host.to_string() + uri;
	
   	let resp = http::handle()
		    .get(url)
		    .exec().unwrap();

	let url_code = String::from_utf8_lossy(resp.get_body());

 	if url_code.contains(text) { return "OK".to_string(); } 
	else { return "CRITICAL".to_string(); } 	
}

fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => { return }
	};

	let text = opts.text.as_slice();
	let result = check_text (opts.hostname.as_slice(), opts.uri.as_slice(), text);

	if result.contains("TEXT ERROR") {
		println!("UNKNOWN: Could not execute text check: {}.", result); 
		os::set_exit_status(3);	
	}
	else if result == "OK" {
		println!("OK: The text \"{}\" is on the specified site.", text);
		os::set_exit_status(0);	
	}
	else if result == "CRITICAL" {
		println!("CRITICAL: Could not find the text \"{}\" on the specified site.", text);
		os::set_exit_status(2);	
	}
	else {
		println!("UNKNOWN: Could not execute text check. Unknown error."); 
		os::set_exit_status(3);	
	}

	return;
}



