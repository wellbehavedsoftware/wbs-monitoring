//Rust file
extern crate getopts;

use getopts::Options;
use std::env;
use std::option::{ Option };
use std::process;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;

fn print_usage (program: &str, opts: Options) {
	let brief = format!("Usage: {} [options]", program);
	println!("{}", opts.usage(&brief));
}

fn print_help (program: &str, opts: Options) {
	let brief = format!("Help: {} [options]", program);
	println!("{}", opts.usage(&brief));
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
			"",
			"help",
			"print this help menu");

	opts.reqopt (
			"",
			"host",
			"Host in which the command will be executed",
			"<host>");

	opts.reqopt (
			"",
			"state",
			"state of the check that triggers the event",
			"<state>");

	opts.reqopt (
			"",
			"state-type",
			"state type of the check that triggers the event",
			"<state-trype>");

	opts.reqopt (
			"",
			"command",
			"Command that will be executed in the host if the state is critical",
			"<command>");

	opts.reqopt (
			"",
			"service",
			"Service that will be executed in the host if the state is critical",
			"<service>");



	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("ssh_command_handler", opts);
			process::exit(3);
		}
	};

	if matches.opt_present ("help") {
		print_help ("ssh_command_handler", opts);
		process::exit(3);
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
		match process::Command::new (command.to_string())
			.arg (host.to_string())
			.arg (service.to_string())
			.output () {
		Ok (output) => { output }
		Err (err) => { return format!("SSH COMMAND ERROR: {}.", err); }
		};

	return String::from_utf8_lossy(&command_output.stdout).to_string();

}

fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => {
			println!("UNKNOWN: Wrong arguments.");
			process::exit(3);
		}
	};


	let host = &opts.host;
	let state = &opts.state;
	let state_type = &opts.state_type;
	let command = &opts.command;
	let service = &opts.service;

	let mut command_result: String = "UNKNOWN".to_string();

	let fname = "/home/nagios/ssh_command_output.txt";
	let p = Path::new(fname);

	let f = match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&p) {

	    Ok(f) => f,
	    Err(e) => panic!("file error: {}", e),
	};

	let mut writer = BufWriter::new(&f);

	writer.write(state_type.as_bytes());
	writer.write(state.as_bytes());

	if state_type.contains("HARD") && (state.contains("2") || state.contains("3")) {
		command_result = exec_command (host, command, service);
	}

	writer.write_all(&command_result.as_bytes());

	return;
}

