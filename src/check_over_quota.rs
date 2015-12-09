//Rust file
extern crate getopts;

use getopts::Options;
use std::env;
use std::option::{ Option };
use std::process;
use std::fs;
use std::fs::File;

fn print_usage (program: &str, opts: Options) {
	let brief = format!("Usage: {} [options]", program);
	println!("{}", opts.usage(&brief));
}

fn print_help (program: &str, opts: Options) {
	let brief = format!("Help: {} [options]", program);
	println!("{}", opts.usage(&brief));
}

struct Opts {
	container_name: String,
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
			"container-name",
			"Name of the container in which the check will be performed. Set to \"none\" for performing the check in the host.",
			"<container-name>");

	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("check_over_quota", opts);
			process::exit(3);
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_over_quota", opts);
		process::exit(3);
	}

	let container_name = matches.opt_str ("container-name").unwrap ();

	return Some (Opts {
		container_name: container_name,

	});

}


fn check_over_quota(rootfs: &str) -> String {

	let mut	test_file_route: String = "/home/ubuntu/test.txt".to_string();

	if !rootfs.contains("none") {

		test_file_route = format!("/var/lib/lxc/{}/rootfs{}", rootfs, test_file_route);

	}

	match File::create(&test_file_route) {
		Ok (f) => { f }
		Err (e) => {
			return format!("OVER-QUOTA-CRITICAL: Could not create the file {}: {}.", test_file_route, e);
		}
	};

	match fs::remove_file(&test_file_route) {

		Ok (_) => { return format!("OVER-QUOTA-OK: quota limit not exceeded."); }
		Err (e) => {
			return format!("OVER-QUOTA-UNKNOWN: Unexpected error: {}.", e);
		}

	};

}


fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => { return }
	};

	let over_quota_msg = check_over_quota(&opts.container_name);

	println!("{}", over_quota_msg);

	if over_quota_msg.contains("UNKNOWN") {
		process::exit(3);
	}
	else if over_quota_msg.contains("OK") {
		process::exit(0);
	}
	else if over_quota_msg.contains("CRITICAL") {
		process::exit(2);
	}
	else {
		println!("OVER-QUOTA-UNKNOWN: Could not execute over quota check. Unknown error.");
		process::exit(3);
	}

}
