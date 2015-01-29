#![allow(unstable)]
extern crate getopts;

use getopts::{ optflag, reqopt, getopts, short_usage, usage, OptGroup };
use std::os;
use std::option::{ Option };
use std::old_io::{ Command };
use std::f64;

fn print_usage (program: &str, opts: &[OptGroup]) {
	println! ("{}", short_usage (program, opts));
}

fn print_help (program: &str, opts: &[OptGroup]) {
	println! ("{}", usage (program, opts));
}

struct Options {
	local: String,
	remote: String,
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
			"l",
			"local",
			"folder in which the local repository is placed",
			"<local-repository>"),

		reqopt (
			"r",
			"remote",
			"remote repository ssh",
			"<remote-repository-ssh>"),

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

	
	let container = matches.opt_str ("container").unwrap ();
	let warning = matches.opt_str ("warning").unwrap ();

	return Some (Options {
		local: local,
		remote: remote,
	});

}

fn check_git_changes(local: &str) -> String {

	let changes_stamp;

	let changes_output =
		match Command::new ("git")
			.arg ("-C".to_string ())
			.arg (local.to_string ())
			.arg ("status".to_string ())
			.output () {
		Ok (output) => { output }
		Err (_) => { return "GIT ERROR".to_string(); }
	};
	changes = String::from_utf8_lossy(changes_output.output.as_slice()).to_string();
	
	return "CHANGES".to_string();
}

fn check_git_sync(local: &str, remote: &str) -> String {

	llet changes_stamp;

	let changes_output =
		match Command::new ("git")
			.arg ("-C".to_string ())
			.arg (local.to_string ())
			.arg ("fetch".to_string ())
			.arg (remote.to_string ())
			.arg ("--dry-run".to_string ())
			.output () {
		Ok (output) => { output }
		Err (_) => { return "GIT ERROR".to_string(); }
	};
	changes = String::from_utf8_lossy(changes_output.output.as_slice()).to_string();
	
	return "CHANGES".to_string();
}

fn check_checkout(local: &str, compareTo: &str) -> String {

	let changes_stamp;

	let changes_output =
		match Command::new ("git")
			.arg ("diff".to_string ())
			.arg (local.to_string ())
			.arg (compareTo.to_string ())
			.output () {
		Ok (output) => { output }
		Err (_) => { return "GIT ERROR".to_string(); }
	};
	changes = String::from_utf8_lossy(changes_output.output.as_slice()).to_string();
	
	return "CHANGES".to_string();

}

fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => { return }
	};
	
	
	
	return;
}

