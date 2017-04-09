//Rust file
extern crate getopts;
extern crate regex;

use getopts::Options;
use std::env;
use std::option::{ Option };
use std::process;
use regex::Regex;

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
	service: String,
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
			"Name of the container in which the check will be performed.",
			"<container-name>");

	opts.reqopt (
			"",
			"service",
			"Service that is going to be checked (smtp or imap).",
			"<service>");

	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("check_mail_ports", opts);
			process::exit(3);
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_mail_ports", opts);
		process::exit(3);
	}

	let container_name = matches.opt_str ("container-name").unwrap ();
	let service = matches.opt_str ("service").unwrap ();

	return Some (Opts {
		container_name: container_name,
		service: service,

	});

}


fn check_mail_ports(container_name: &str, service: &str) -> String {


	let netstat_output =
		match process::Command::new ("sudo")
			.arg ("lxc-attach".to_string ())
			.arg ("--name".to_string ())
			.arg (&container_name)
			.arg ("--".to_string ())
			.arg ("netstat".to_string ())
			.arg ("-tulpn".to_string ())
			.output () {
		Ok (output) => { output }
		Err (err) => { return format!("MAIL-PORTS-UNKNOWN: Could not perform the check: {}.", err); }
	};

	let netstat = String::from_utf8_lossy(&netstat_output.stdout).to_string();

	if netstat.is_empty() {
		return format!("MAIL-PORTS-UNKNOWN: The container {} is not running or does not exist.", container_name);
	}

	let mut expression: String = "".to_string();
	let mut port_list = vec![];

	if service.contains("imap") {
		expression = format!("tcp (.+)dovecot");
		port_list = vec![":143", ":993"];
	}
	else if service.contains("smtp") {
		expression = format!("tcp (.+)master");
		port_list = vec![":25", ":587", ":465"];
	}
	else {
		return format!("MAIL-PORTS-UNKNOWN: Unknown service {}. Choose \"smtp\" or \"imap\".", container_name);
	}


	let re = Regex::new(&expression).unwrap();
	let mut matches = 0;

	for cap in re.captures_iter(&netstat) {
		let capt = cap.get(1).map_or("", |m| m.as_str ()).trim();
		for port in port_list.clone() {
			if capt.contains(port) { matches = matches + 1; }
		}

	}

	if matches >= port_list.len() {
		return format!("MAIL-PORTS-OK: {} ports configuration is OK.", service);
	}
	else {
		return format!("MAIL-PORTS-CRITICAL: The {} service is not correclty configured.\n{}", service, netstat);
	}
}


fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => { return }
	};

	let mail_ports_msg = check_mail_ports(&opts.container_name, &opts.service);

	println!("{}", mail_ports_msg);

	if mail_ports_msg.contains("UNKNOWN") {
		process::exit(3);
	}
	else if mail_ports_msg.contains("OK") {
		process::exit(0);
	}
	else if mail_ports_msg.contains("WARNING") {
		process::exit(1);
	}
	else if mail_ports_msg.contains("CRITICAL") {
		process::exit(2);
	}
	else {
		println!("MAIL-PORTS-UNKNOWN: Could not execute mail ports check. Unknown error.");
		process::exit(3);
	}

}
