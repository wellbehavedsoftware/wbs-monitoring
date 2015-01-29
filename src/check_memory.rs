#![allow(unstable)]
extern crate getopts;

use getopts::{ optflag, getopts, short_usage, usage, OptGroup };
use std::os;
use std::option::{ Option };
use std::old_io::{ Command };
use std::f64;

fn print_usage (program: &str, opts: &[OptGroup]) {
	println! ("{}", short_usage (program, opts));
}

fn print_help (program: &str, opts: &[OptGroup]) {
	println! ("{}", usage (program, opts));
}

fn parse_options () {

	let args: Vec<String> = os::args ();

	let program = args [0].clone ();

	let opts = &[

		optflag (
			"h",
			"help",
			"print this help menu"),

	];

	let matches = match getopts (args.tail (), opts) {
		Ok (m) => { m }
		Err (_) => {
			print_usage (program.as_slice (), opts);
			panic!("");
		}
	};

	if matches.opt_present ("help") {
		print_help (program.as_slice (), opts);
		os::set_exit_status(3);	
		panic!("");
	}

	if ! matches.free.is_empty () {
		print_usage (program.as_slice (), opts);
		os::set_exit_status(3);	
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
		os::set_exit_status(3);	
		return;
	}

	let mut state_vector: Vec<&str> = state.as_slice().split('\n').collect();
	state_vector = state_vector[2].as_slice().split(' ').collect();

	let mut index = 2;
	while state_vector[index].as_slice().is_empty() {
		index = index + 1;
	}

	let memory_used_aux: Option<f64> = state_vector[index].parse();
	let memory_used: f64 = memory_used_aux.unwrap();

	index = index + 1;
	while state_vector[index].as_slice().is_empty() {
		index = index + 1;
	}

	let memory_limit_aux: Option<f64> = state_vector[index].parse();
	let mut memory_limit: f64 = memory_limit_aux.unwrap();
	memory_limit = memory_limit + memory_used;

	let memory_used_percentage = memory_used / memory_limit;

	let mem_quota_used = f64::to_str_exact(memory_used / 1073741824.0, 2);
	let mem_quota_limit = f64::to_str_exact(memory_limit / 1073741824.0, 2);
	let mem_quota_percentage = f64::to_str_exact(memory_used_percentage * 100.0, 2);

	println!("MEM OK: {} GiB {}%, limit {} GiB.", mem_quota_used, mem_quota_percentage, mem_quota_limit);
	
	os::set_exit_status(0);
	return;
}


