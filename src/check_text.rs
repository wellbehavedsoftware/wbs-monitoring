//Rust file
extern crate getopts;
extern crate curl;

use getopts::Options;
use std::env;
use std::process;
use std::option::{ Option };
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
}

fn parse_options () -> Option<Opts> {

	let args = env::args ();

	let mut opts = Options::new();

	opts.optflag (	
			"h",
			"help",
			"print this help menu");

	opts.reqopt (
			"t",
			"text",
			"the text that is going to be searched",
			"<text-to-check>");

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
			print_usage ("check_text", opts);
			process::exit(3);
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_text", opts);
		process::exit(3);
	}

	let hostname = matches.opt_str ("host-name").unwrap ();
	let uri = matches.opt_str ("uri").unwrap ();
	let text = matches.opt_str ("text").unwrap ();

	return Some (Opts {
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

	let text = &opts.text;
	let result = check_text (&opts.hostname, &opts.uri, text);

	if result.contains("TEXT ERROR") {
		println!("UNKNOWN: Could not execute text check: {}.", result); 
		process::exit(3);	
	}
	else if result == "OK" {
		println!("OK: The text \"{}\" is on the specified site.", text);
		process::exit(0);	
	}
	else if result == "CRITICAL" {
		println!("CRITICAL: Could not find the text \"{}\" on the specified site.", text);
		process::exit(2);	
	}
	else {
		println!("UNKNOWN: Could not execute text check. Unknown error."); 
		process::exit(3);	
	}

}



