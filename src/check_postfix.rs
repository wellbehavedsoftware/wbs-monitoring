//Rust file
extern crate getopts;
extern crate chrono;

use getopts::Options;
use std::env;
use std::process;
use std::option::{ Option };
use chrono::*;

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
	mails: String,
	quota: String,
	age: String,
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
			"m",
			"mails",
			"the maximum number of messages allowed",
			"<mails>");

	opts.reqopt (
			"q",
			"quota",
			"maximum mails per hour quota allowed",
			"<quota>");

	opts.reqopt (
			"a",
			"age",
			"maximum age (in days) that mails are allowed to stay in the queue",
			"<age>");

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
	let mails = matches.opt_str ("mails").unwrap ();
	let quota = matches.opt_str ("quota").unwrap ();
	let age = matches.opt_str ("age").unwrap ();

	return Some (Opts {
		rootfs: rootfs,
		mails: mails,
		quota: quota,
		age: age,
	});

}

fn check_email_queue (rootfs: &str, max_mails: i32) -> String {

	let mut queue_output: String = "".to_string();
	let mut deferred_output: String = "".to_string();
	
	if !rootfs.is_empty() {
	
		//check email queue

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
	
		queue_output = String::from_utf8_lossy(&email_output.stdout).to_string();

		//check deferred email queue

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

		deferred_output =  String::from_utf8_lossy(&email_output.stdout).to_string();
	}
	else {

		//check email queue

		let email_output =
			match process::Command::new ("sudo")
			.arg ("qshape".to_string ())		
			.output () {
		Ok (output) => { output }
		Err (err) => { return format!("Check email: {}.", err); }
		};
	
		queue_output = String::from_utf8_lossy(&email_output.stdout).to_string();

		//check deferred email queue

		let email_output =
			match process::Command::new ("sudo")
			.arg ("qshape".to_string ())
			.arg ("deferred".to_string ())	
			.output () {
		Ok (output) => { output }
		Err (err) => { return format!("Check email: {}.", err); }
		};

		deferred_output =  String::from_utf8_lossy(&email_output.stdout).to_string();
	}

	// if any of the checks failed, unknown is returned

	if queue_output.contains("failed to get the init pid") || queue_output.is_empty() || 
	   deferred_output.contains("failed to get the init pid") || deferred_output.is_empty()	
	{
		return format!("MAIL-UNKNOWN: Unable to perform the check: {}\n{}", queue_output, deferred_output);
	}

	// check if the number of mails surpasses the maximum	

	let queue_lines: Vec<&str> = queue_output.split('\n').collect();
	let total_queue_mails_line: Vec<&str> = queue_lines[1].split(' ').collect();

	let deferred_lines: Vec<&str> = deferred_output.split('\n').collect();
	let total_deferred_mails_line: Vec<&str> = deferred_lines[1].split(' ').collect();

	let mut queue_mails: i32 = 0;

	for token in total_queue_mails_line {

		if !token.is_empty() && token != "TOTAL" {

			queue_mails = match token.parse() {
				Ok (i32) => { i32 }
				Err (_) => {
					return "UNKNOWN: Error while parsing mails number.".to_string(); 
				}
			};
 
			break; 
		}
	}

	let mut deferred_mails: i32 = 0;

	for token in total_deferred_mails_line {

		if !token.is_empty() && token != "TOTAL" {

			deferred_mails = match token.parse() {
				Ok (i32) => { i32 }
				Err (_) => {
					return "UNKNOWN: Error while parsing deferred mails number.".to_string(); 
				}
			};
 
			break; 
		}
	}

	let total_mails = queue_mails + deferred_mails;

	if total_mails <= max_mails {

		return format!("POSTFIX-OK: Emails queue has {} mails. Deferred emails queue has {} mails. Max. allowed: {}.", queue_mails, deferred_mails, max_mails);

	}
	else {

		return format!("POSTFIX-WARNING: Emails queue has {} mails. Deferred emails queue has {} mails. Max allowed: {}.\n\nMails queue:\n\n{}\n\nDeferred mails queue:\n\n{}", queue_mails, deferred_mails, max_mails, queue_output, deferred_output);

	}

}


fn check_emails_age (rootfs: &str, max_age: i32) -> String {

	let now = UTC::now();
	let mut mailq_data: String = "".to_string();

	if !rootfs.is_empty() {
	
		//check mailq data

		let mailq_output =
			match process::Command::new ("sudo")
			.arg ("lxc-attach".to_string ())
			.arg ("--name".to_string ())
			.arg (rootfs.to_string ())
			.arg ("--".to_string ())
			.arg ("mailq".to_string ())		
			.output () {
		Ok (output) => { output }
		Err (err) => { return format!("Check email: {}.", err); }
		};
	
		mailq_data = String::from_utf8_lossy(&mailq_output.stdout).to_string();

	}
	else {

		//check mailq data

		let mailq_output =
			match process::Command::new ("sudo")
			.arg ("mailq".to_string ())		
			.output () {
		Ok (output) => { output }
		Err (err) => { return format!("Check email: {}.", err); }
		};
	
		mailq_data = String::from_utf8_lossy(&mailq_output.stdout).to_string();

	}

	let mailq_lines: Vec<&str> = mailq_data.split('\n').collect();
	let mut index = 1;

	let mut warnings: i32 = 0;
	let mut warning_messages: String = "".to_string();

	while ((index + 2) < mailq_lines.len()) {

		// obtaining the mail arrival datetime

		let mailq_line_tokens: Vec<&str> = mailq_lines[index].split(' ').collect();

		let mut i = 1;
		while (mailq_line_tokens[i].is_empty()) { i = i + 1; }
		i = i + 1;

		let date_string = format!("{} {} {} {}", mailq_line_tokens[i], mailq_line_tokens[i+1], mailq_line_tokens[i+2], mailq_line_tokens[i+3]);

		// add year to datetime in order to perform arithmetic operations

		let mut complete_date = format!("{} {}", date_string, now.year());

		let date_object = UTC.datetime_from_str(&complete_date, "%a %b %e %T %Y");

		let date = match date_object {

			Ok (date) => { date }
			Err (_) => {

				// fix the problem if the resultin datetime is from the future

				complete_date = format!("{} {}", date_string, now.year() - 1);
				UTC.datetime_from_str(&complete_date, "%a %b %e %T %Y").unwrap()	
			}
		};

		let date_dif = now - date;

		let diffseconds = date_dif.num_seconds() as f64;
		let diffdays = (diffseconds / (24.0 * 3600.0)) as i32;

		if diffdays > max_age {

			warning_messages = format!("{}POSTFIX-WARNING: Email from {} with date {}.\n", warning_messages, mailq_line_tokens[i+4], complete_date);
			warnings = warnings + 1;

		}

		index = index + 4;
	}

	if warnings > 0 {

		return format!("POSTFIX-WARNING: {} emails have been in the queu for more than {} days:\n", warnings, max_age);

	}
	else {
		return format!("POSTFIX-OK: {} emails have been in the queue for more than {} days.\n", warnings, max_age);
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
	let mails : i32 = match opts.mails.parse() {
		Ok (i32) => { i32 }
		Err (_) => {
			println!("UNKNOWN: The maximum mails number must be an integer!"); 
			process::exit(3);
		}
	};
	let quota : i32 = match opts.quota.parse() {
		Ok (i32) => { i32 }
		Err (_) => {
			println!("UNKNOWN: The maximum mails per hour quota must be an integer!"); 
			process::exit(3);
		}
	};

	let age : i32 = match opts.age.parse() {
		Ok (i32) => { i32 }
		Err (_) => {
			println!("UNKNOWN: The maximum days that mails are allowed to stay in the queue must be an integer!"); 
			process::exit(3);
		}
	};

	let mut result = "".to_string();

	if rootfs.contains("mail") || rootfs.contains("dovecot") {

		result = check_email_queue(rootfs, mails);

		let mailq_result = check_emails_age (rootfs, age);
		
		if result.contains("OK") && !mailq_result.contains("OK") {
			result = format!("{}\n{}", mailq_result, result);
		}
		else {
			result = format!("{}\n{}", result, mailq_result);
		}
	}
	else {
		result = check_email_queue(rootfs, 0);
	}

	println!("{}", result);

	if result.contains("UNKNOWN") {

		process::exit(3);

	}
	else if result.contains("WARNING") {

		process::exit(1);

	}
	else {

		process::exit(0);
	}
	
}
