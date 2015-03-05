//Rust file
#![feature(env)]
#![feature(core)]
#![feature(io)]

extern crate getopts;
extern crate curl;

use getopts::Options;
use std::env;
use std::option::{ Option };
use std::old_io::{ Command };
use curl::http;

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
			"service",
			"Command that will be executed in the host",
			"<command>");

	opts.reqopt (
			"S",
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
	let command = matches.opt_str ("command").unwrap ();

	return Some (Opts {
		host: host,
		command: command,
	});

}

fn main () {

	/*let opts = match parse_options () {
		Some (opts) => { opts }
		None => { 
			env::set_exit_status(3);
			println!("UNKNOWN: Wrong arguments.");
			return;
		}
	};


	let host = opts.host.as_slice();
	let command_option = opts.command.as_slice();

	let command: String = match command_option {

		//"apache-restart" => return "service apache2 restart".to_string(),

	};

	println!("{}", command);*/
	
	return;
}

