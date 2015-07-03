//Rust file
extern crate getopts;

use getopts::Options;
use std::env;
use std::process;
use std::option::{ Option };

fn print_usage (program: &str, opts: Options) {
	let brief = format!("Usage: {} [options]", program);
	println!("{}", opts.usage(&brief));
}

fn print_help (program: &str, opts: Options) {
	let brief = format!("Help: {} [options]", program);
	println!("{}", opts.usage(&brief));
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
				match process::Command::new ("sudo")
				.arg ("lxc-attach".to_string ())
				.arg ("--name".to_string ())
				.arg (rootfs.to_string ())
				.arg ("--".to_string ())
				.arg ("qshape".to_string ())
				.arg ("deferred".to_string ())				
				.output () {
			Ok (output) => { output }
			Err (err) => { return format!("Check email: {}.", err); }
			};
			
			return String::from_utf8_lossy(&email_output.stdout).to_string();
		}
		else {
			let email_output =
				match process::Command::new ("sudo")
				.arg ("lxc-attach".to_string ())
				.arg ("--name".to_string ())
				.arg (rootfs.to_string ())
				.arg ("--".to_string ())
				.arg ("qshape".to_string ())
				.output () {
			Ok (output) => { output }
			Err (err) => { return format!("Check email: {}.", err); }
			};

			return String::from_utf8_lossy(&email_output.stdout).to_string();
		}
	}
	else {
		if deferred {
			let email_output =
				match process::Command::new ("sudo")
				.arg ("qshape".to_string ())
				.arg ("deferred".to_string ())
				.output () {
			Ok (output) => { output }
			Err (err) => { return format!("Check email: {}.", err); }
			};

			return String::from_utf8_lossy(&email_output.stdout).to_string();
		}
		else {
			let email_output =
				match process::Command::new ("sudo")
				.arg ("qshape".to_string ())
				.output () {
			Ok (output) => { output }
			Err (err) => { return format!("Check email: {}.", err); }
			};

			return String::from_utf8_lossy(&email_output.stdout).to_string();
		}
	}
}

fn check_email_output (mail_output: String, deferred: bool) -> (String, String) {

	if mail_output.contains("failed to get the init pid") || mail_output.is_empty() {
		return (format!("MAIL-UNKNOWN: Unable to perform the check: {}", mail_output), "".to_string());
	}

	let lines: Vec<&str> = mail_output.split('\n').collect();
	let total_mails_line: Vec<&str> = lines[1].split(' ').collect();

	let mut total_mails = "";
	for token in total_mails_line {
		if !token.is_empty() && token != "TOTAL" { total_mails = token; break; }
	}

	if !deferred && lines.len() > 3 {
		return (format!("MAIL-WARNING: {} at emails queue.\n{}", total_mails, mail_output), total_mails.to_string());
	}
	else if deferred && lines.len() > 3 {
		return (format!("MAIL-WARNING: {} emails at the deferred emails queue.\n{}", total_mails, mail_output), total_mails.to_string());
	}
	else {
		return ("MAIL-OK: The emails queue is empty.\n".to_string(), total_mails.to_string());
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

	let rootfs = &opts.rootfs;
	let deferred = opts.deferred;
	let container = opts.container;

	let mut command_output = check_email (rootfs, false, container);
	let (mut result, mails) = check_email_output (command_output, false);

	let mut deferred_values: (String, String) = ("".to_string(), "".to_string());

	if deferred {
		command_output = check_email (rootfs, true, container);
		let (deferred_result, deferred_mails) = check_email_output (command_output, true);
		deferred_values = (deferred_result, deferred_mails);
	}

	let (deferred_result, deferred_mails) = deferred_values;

	if result.contains("UNKNOWN") {

		result = result + "\n -- Deferred queue -- \n\n" + &deferred_result; 

		println!("{} | mails={};;;; deferred_mails={};;;;", result, mails, deferred_mails);
		process::exit(3);

	}
	else if deferred_result.contains("UNKNOWN") {

		result = deferred_result + "\n -- Queue -- \n\n" + &result; 

		println!("{} | mails={};;;; deferred_mails={};;;;", result, mails, deferred_mails);
		process::exit(3);

	}
	else if result.contains("WARNING") {

		result = result + "\n -- Deferred queue -- \n\n" + &deferred_result; 

		println!("{} | mails={};;;; deferred_mails={};;;;", result, mails, deferred_mails);
		process::exit(1);

	}
	else if deferred_result.contains("WARNING") {

		result = deferred_result + "\n -- Queue -- \n\n" + &result; 

		println!("{} | mails={};;;; deferred_mails={};;;;", result, mails, deferred_mails);
		process::exit(1);

	} else {
		println!("{} | mails={};;;; deferred_mails={};;;;", result, mails, deferred_mails);
		process::exit(0);
	}
	
}
