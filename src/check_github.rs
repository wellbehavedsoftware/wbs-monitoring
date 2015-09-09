//Rust file

extern crate getopts;
extern crate curl;
extern crate time;
extern crate serde_json;

use getopts::Options;
use std::env;
use std::option::{ Option };
use std::process;
use std::thread;
use curl::http;
use time::PreciseTime;
use serde_json::Value;

fn print_usage (program: &str, opts: Options) {
	let brief = format!("Usage: {} [options]", program);
	println!("{}", opts.usage(&brief));
}

fn print_help (program: &str, opts: Options) {
	let brief = format!("Help: {} [options]", program);
	println!("{}", opts.usage(&brief));
}

struct Opts {
	owner: String,
	repository: String,
	version: String,
	timeout: String,
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
			"owner",
			"Owner of the repository that is going to be checked.",
			"<owner>");

	opts.reqopt (
			"",
			"repository",
			"Name of the repository that is going to be checked.",
			"<repository>");

	opts.reqopt (
			"",
			"version",
			"Installed version of the repository that is going to be checked.",
			"<version>");

	opts.reqopt (
			"",
			"timeout",
			"timeout in which the check stops waiting aborts the process",
			"<http-enabled>");

	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => { 
			print_usage ("check_github", opts);
			process::exit(3);
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_github", opts);
		process::exit(3);
	}

	let owner = matches.opt_str ("owner").unwrap ();
	let repository = matches.opt_str ("repository").unwrap ();
	let version = matches.opt_str ("version").unwrap ();
	let timeout = matches.opt_str ("timeout").unwrap ();

	return Some (Opts {
		owner: owner,
		repository: repository,
		version: version,
		timeout: timeout,

	});

}

fn check_github (owner: &str, repository: &str, version: &str, timeout: f64) -> String {
    
	let prefix = "https://api.github.com/repos".to_string();

	let url = format!("{}/{}/{}/releases/latest", prefix, owner, repository);

	let start = PreciseTime::now();

	let child = thread::spawn(move || {	

	   	let mut http_handle = http::handle();

		let mut http_request = http_handle.get (url);

		http_request = http_request.header("Accept-Language", "en");
		http_request = http_request.header("User-Agent", "well-behaved-software");

		let resp = http_request.exec().unwrap();

		let url_code = String::from_utf8_lossy(resp.get_body()).to_string();

		return format!("{}", url_code);
	});

	// Wait for the call to finish
	let res = child.join();

	let end = PreciseTime::now();

	let millis = start.to(end).num_milliseconds() as f64;

	if millis > timeout {
		return format!("GITHUB-CRITICAL: The check timed out at {} milliseconds.", millis);
	}

	// Get the child's process results
	let result = match res {
		Ok (value) => { value }
		Err (_) => { 
			return format!("GITHUB-CRITICAL: The check could not be performed. No response received."); 
		}
	};

	let data: Value = serde_json::from_str(&result).unwrap();
	let obj = data.as_object().unwrap();
	let release = obj.get("tag_name").unwrap().as_string().unwrap();
	
	if release != version {
		return format!("GITHUB-WARNING: {} new release available ({}). {} currently installed.", repository, release, version); 
	}
	else {
		return format!("GITHUB-OK: {} version is up to date.", repository); 
	}


}

fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => { 
			println!("GITHUB-UNKNOWN: Wrong arguments.");
			process::exit(3);
		}
	};

	let owner = &opts.owner;
	let repository = &opts.repository;
	let version = &opts.version;

	let timeout : f64 = match opts.timeout.parse() {
		Ok (f64) => { f64 }
		Err (_) => {
			println!("GITHUB-UNKNOWN: Timeout must be a number."); 
			process::exit(3);
		}
	};

	let github_res = check_github(owner, repository, version, timeout);
	println!("{}", github_res);

	if github_res.contains("UNKNOWN") {
		process::exit(3);
	}
	else if github_res.contains("CRITICAL") {
		process::exit(2);
	} 
	else if github_res.contains("WARNING") {
		process::exit(1);
	} 
	else {
		process::exit(0);
	}

}
