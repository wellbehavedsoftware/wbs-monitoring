//Rust file
#![feature(env)]
#![feature(core)]
#![feature(io)]

extern crate getopts;

use getopts::Options;
use std::env;
use std::option::{ Option };
use std::old_io::{ Command };

fn print_usage (program: &str, opts: Options) {
	let brief = format!("Usage: {} [options]", program);
	println!("{}", opts.usage(brief.as_slice()));
}

fn print_help (program: &str, opts: Options) {
	let brief = format!("Help: {} [options]", program);
	println!("{}", opts.usage(brief.as_slice()));
}

struct Opts {
	rootfs: String,
	deferred: bool,
	container: bool,
}

fn parse_options () -> Option<Opts> {

	let args = env::args ();

	let mut opts = Options::new();

	opts.optflag (	
			"h",
			"help",
			"print this help menu");

	opts.reqopt (
			"r",
			"rootfs",
			"root of the file system in which the checks will be performed",
			"<rootfs>");

	opts.reqopt (
			"c",
			"container",
			"the specified rootfs is a container",
			"<container>");

	opts.reqopt (
			"d",
			"deferred",
			"the deferred queue is also checked",
			"<deferred>");

	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("check_email", opts);
			return None;
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_email", opts);
		return None;
	}

	let mut deferred = false;
	let mut container = false;

	if matches.opt_present ("deferred") {
		deferred = true;
	}

	if matches.opt_present ("container") {
		container = true;
	}

	let rootfs = matches.opt_str ("rootfs").unwrap ();
	let cont = matches.opt_str ("container").unwrap ();
	let mut container = false;
	if cont == "true" {
		container = true;
	}
	let def = matches.opt_str ("deferred").unwrap ();
	let mut deferred = false;
	if def == "true" {
		deferred = true;
	}

	return Some (Opts {
		rootfs: rootfs,
		deferred: deferred,
		container: container,
	});

}

fn check_email (rootfs: &str, deferred: bool, container: bool) -> String {
	
	if container {
		if deferred {
			let email_output =
				match Command::new ("sudo")
				.arg ("lxc-attach".to_string ())
				.arg ("--name".to_string ())
				.arg (rootfs.to_string ())
				.arg ("qshape".to_string ())
				.arg ("deferred".to_string ())				
				.output () {
			Ok (output) => { output }
			Err (err) => { return format!("Check email: {}.", err); }
			};

			return String::from_utf8_lossy(email_output.output.as_slice()).to_string();
		}
		else {
			let email_output =
				match Command::new ("sudo")
				.arg ("lxc-attach".to_string ())
				.arg ("--name".to_string ())
				.arg (rootfs.to_string ())
				.arg ("qshape".to_string ())
				.output () {
			Ok (output) => { output }
			Err (err) => { return format!("Check email: {}.", err); }
			};

			return String::from_utf8_lossy(email_output.output.as_slice()).to_string();
		}
	}
	else {
		if deferred {
			let email_output =
				match Command::new ("sudo")
				.arg ("qshape".to_string ())
				.arg ("deferred".to_string ())
				.output () {
			Ok (output) => { output }
			Err (err) => { return format!("Check email: {}.", err); }
			};

			return String::from_utf8_lossy(email_output.output.as_slice()).to_string();
		}
		else {
			let email_output =
				match Command::new ("sudo")
				.arg ("qshape".to_string ())
				.output () {
			Ok (output) => { output }
			Err (err) => { return format!("Check email: {}.", err); }
			};

			return String::from_utf8_lossy(email_output.output.as_slice()).to_string();
		}
	}
}

fn check_email_output (mail_output: String, deferred: bool) -> String {

	if mail_output.contains("failed to get the init pid") || mail_output.is_empty() {
		return format!("MAIL-UNKNOWN: Unable to perform the check: {}", mail_output);
	}

	let lines: Vec<&str> = mail_output.as_slice().split('\n').collect();

	if !deferred && lines.len() > 3 {
		return format!("MAIL-WARNING: The emails queue is not empty.\n{}", mail_output);
	}
	else if deferred && lines.len() > 4 {
		return format!("MAIL-WARNING: The deferred emails queue is not empty.\n{}", mail_output);
	}
	else {
		return "MAIL-OK: The emails queue is empty.\n".to_string();
	}
	
}

fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => { 
			env::set_exit_status(3);
			println!("UNKNOWN: Wrong arguments.");
			return;
		}
	};

	let rootfs = opts.rootfs.as_slice();
	let deferred = opts.deferred;
	let container = opts.container;

	let mut command_output = check_email (rootfs, false, container);
	let mut result = check_email_output (command_output, false);

	if deferred {
		command_output = check_email (rootfs, true, container);
		result = result + "\n\n -- Deferred queue -- \n\n" + check_email_output (command_output, true).as_slice();
	}

	if result.contains("UNKNOWN") {

		env::set_exit_status(3);
		println!("{}", result);

	}
	else if result.contains("WARNING") {

		env::set_exit_status(1);
		println!("{}", result);

	} else {

		env::set_exit_status(0);
		println!("{}", result);

	}
	
	return;
}
