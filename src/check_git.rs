//Rust file
#![feature(env)]
#![feature(core)]
#![feature(io)]
#![feature(collections)]
#![feature(std_misc)]

extern crate getopts;
extern crate git2;

use getopts::Options;
use std::env;
use std::option::{ Option };
use std::old_io::{ Command };
use std::f64;
use git2::Repository;

fn print_usage (program: &str, opts: Options) {
	let brief = format!("Usage: {} [options]", program);
	println!("{}", opts.usage(brief.as_slice()));
}

fn print_help (program: &str, opts: Options) {
	let brief = format!("Help: {} [options]", program);
	println!("{}", opts.usage(brief.as_slice()));
}

struct Opts {
	local: String,
	remote: String,
}

fn parse_options () -> Option<Opts> {

	let args = env::args ();

	let mut opts = Options::new();

	opts.optflag (	
			"h",
			"help",
			"print this help menu");

	opts.reqopt (
			"l",
			"local",
			"folder in which the local repository is placed",
			"<local-repository>");

	opts.reqopt (
			"r",
			"remote",
			"remote repository ssh",
			"<remote-repository-ssh>");

	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("check_git", opts);
			return None;
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_git", opts);
		return None;
	}

	let local = matches.opt_str ("local").unwrap ();
	let remote = matches.opt_str ("remote").unwrap ();

	return Some (Opts {
		local: local,
		remote: remote,
	});

}

fn check_git_changes(local: &str) -> String {

	let changes_output =
		match Command::new ("git")
			.arg ("-C".to_string ())
			.arg (local.to_string ())
			.arg ("status".to_string ())
			.output () {
		Ok (output) => { output }
		Err (_) => { return "GIT ERROR".to_string(); }
	};

	let changes = String::from_utf8_lossy(changes_output.output.as_slice()).to_string();
	
	return "CHANGES".to_string();
}

fn check_git_sync(local: &str, remote: &str) -> String {

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

	let changes = String::from_utf8_lossy(changes_output.output.as_slice()).to_string();
	
	return "CHANGES".to_string();
}

fn check_checkout(local: &str, compare_to: &str) -> String {

	let changes_output =
		match Command::new ("git")
			.arg ("diff".to_string ())
			.arg (local.to_string ())
			.arg (compare_to.to_string ())
			.output () {
		Ok (output) => { output }
		Err (_) => { return "GIT ERROR".to_string(); }
	};

	let changes = String::from_utf8_lossy(changes_output.output.as_slice()).to_string();
	
	return "CHANGES".to_string();

}

fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => { return }
	};
	
	
	
	return;
}

