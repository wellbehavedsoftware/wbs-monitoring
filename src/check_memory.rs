//Rust file
#![feature(env)]
#![feature(core)]
#![feature(io)]
#![feature(std_misc)]

extern crate getopts;

use getopts::Options;
use std::env;
use std::old_io::{ Command };
use std::f64;

fn print_usage (program: &str, opts: Options) {
	let brief = format!("Usage: {} [options]", program);
	println!("{}", opts.usage(brief.as_slice()));
}

fn print_help (program: &str, opts: Options) {
	let brief = format!("Help: {} [options]", program);
	println!("{}", opts.usage(brief.as_slice()));
}

fn parse_options () {

	let args = env::args ();

	let mut opts = Options::new();

	opts.optflag (	
			"h",
			"help",
			"print this help menu");

	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("check_memory", opts);
			panic!("");
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_memory", opts);
		env::set_exit_status(3);	
		panic!("");
	}

}

fn memory_state () -> String {

	let list_output =
		match Command::new ("free")
			.arg ("-b".to_string ())
			.output () {
		Ok (output) => { output }
		Err (_) => { return "ERROR".to_string(); }
	};

	String::from_utf8_lossy(list_output.output.as_slice()).to_string()
}


fn main () {

	parse_options ();

	let state = memory_state ();

	if state == "ERROR".to_string() {
		println!("MEM UNKNOWN: Could not execute memory check command."); 
		env::set_exit_status(3);	
		return;
	}

	let mut state_vector: Vec<&str> = state.as_slice().split('\n').collect();
	state_vector = state_vector[2].as_slice().split(' ').collect();

	let mut index = 2;
	while state_vector[index].as_slice().is_empty() {
		index = index + 1;
	}

	let memory_used : f64 = match state_vector[index].parse() {
		Ok (f64) => { f64 }
		Err (_) => { 
			println!("UNKNOWN: The memory used data is incorrect."); 
			env::set_exit_status(3);	
			return;
		}
	};

	index = index + 1;
	while state_vector[index].as_slice().is_empty() {
		index = index + 1;
	}

	let mut memory_limit : f64 = match state_vector[index].parse() {
		Ok (f64) => { f64 }
		Err (_) => { 
			println!("UNKNOWN: The memory limit data is incorrect."); 
			env::set_exit_status(3);	
			return;
		}
	};


	memory_limit = memory_limit + memory_used;

	let memory_used_percentage = memory_used / memory_limit;

	let mem_quota_used = f64::to_str_exact(memory_used / 1073741824.0, 2);
	let mem_quota_limit = f64::to_str_exact(memory_limit / 1073741824.0, 2);
	let mem_quota_percentage = f64::to_str_exact(memory_used_percentage * 100.0, 2);

	println!("MEM OK: {} GiB {}%, limit {} GiB.", mem_quota_used, mem_quota_percentage, mem_quota_limit);
	
	env::set_exit_status(0);
	return;
}


