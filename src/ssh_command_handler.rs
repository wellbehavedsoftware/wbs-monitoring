//Rust file
#![feature(env)]
#![feature(core)]
#![feature(io)]

extern crate getopts;

use getopts::Options;
use std::env;
use std::option::{ Option };
use std::old_io::{ Command };
use std::old_io::{File, Open, Read, Write, ReadWrite};

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
	state_type: String,
	command: String,
	service: String,
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
			"S",
			"state",
			"state of the check that triggers the event",
			"<state>");

	opts.reqopt (
			"t",
			"state-type",
			"state type of the check that triggers the event",
			"<state-trype>");

	opts.reqopt (
			"c",
			"command",
			"Command that will be executed in the host if the state is critical",
			"<command>");

	opts.reqopt (
			"s",
			"service",
			"Service that will be executed in the host if the state is critical",
			"<service>");



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
	let state_type = matches.opt_str ("state-type").unwrap ();
	let command = matches.opt_str ("command").unwrap ();
	let service = matches.opt_str ("service").unwrap ();

	return Some (Opts {
		host: host,
		state: state,
		state_type: state_type,
		command: command,
		service: service,
	});

}

fn exec_command (host: &str, command: &str, service: &str) -> String {

	let command_output =
		match Command::new (command.to_string())
			.arg (host.to_string())
			.arg (service.to_string())
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
	let state_type = opts.state_type.as_slice();
	let command = opts.command.as_slice();
	let service = opts.service.as_slice();

	let mut command_result: String = "UNKNOWN".to_string();

	let fname = "/home/nagios/ssh_command_output.txt";
	let p = Path::new(fname);

	let mut f = match File::open_mode(&p, Open, Write) {
	    Ok(f) => f,
	    Err(e) => panic!("file error: {}", e),
	};

	f.write_line(state_type);
	f.write_line(state);

	if state_type.contains("HARD") && (state.contains("2") || state.contains("3")) {
		f.write_line("entro if");
		command_result = exec_command (host, command, service);
	}

	f.write_line(command_result.as_slice());

	return;
}

