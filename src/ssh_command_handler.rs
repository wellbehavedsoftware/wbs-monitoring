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
	host: String,
	state: String,
	command: String,
}

fn parse_options () -> Option<Opts> {

	let args = env::args ();

	let mut opts = Options::new();

	opts.optflag (	
			"h",
			"help",
			"print this help menu");

	opts.reqopt (
			"H",
			"host",
			"Host in which the command will be executed",
			"<host>");

	opts.reqopt (
			"s",
			"state",
			"Command that will be executed in the host",
			"<command>");

	opts.reqopt (
			"c",
			"command",
			"Command that will be executed in the host if the state is critical",
			"<command>");



	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("ssh_command_handler", opts);
			return None;
		}
	};

	if matches.opt_present ("help") {
		print_help ("ssh_command_handler", opts);
		return None;
	}

	let host = matches.opt_str ("host").unwrap ();
	let state = matches.opt_str ("state").unwrap ();
	let command = matches.opt_str ("command").unwrap ();

	return Some (Opts {
		host: host,
		state: state,
		command: command,
	});

}

fn exec_command (host: &str, command: &str) -> String {

	let host_dir = "ubuntu@".to_string() + host + ".vpn.wbsoft.co";

	let command_output =
		match Command::new ("ssh")
			.arg (host_dir)
			.arg (command.to_string ())
			.output () {
		Ok (output) => { output }
		Err (err) => { return format!("SSH COMMAND ERROR: {}.", err); }
		};

	return String::from_utf8_lossy(command_output.output.as_slice()).to_string();

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


	let host = opts.host.as_slice();
	let state = opts.state.as_slice();
	let command = opts.command.as_slice();

	let mut command_result: String = "UNKNOWN".to_string();

	if state.contains("CRITICAL") {
		command_result = exec_command (host, command);
	}

	println!("{}", command_result);

	return;
}

