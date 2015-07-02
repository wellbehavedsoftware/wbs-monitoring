//Rust file
extern crate getopts;

use getopts::Options;
use std::env;
use std::process;

fn print_usage (program: &str, opts: Options) {
	let brief = format!("Usage: {} [options]", program);
	println!("{}", opts.usage(&brief));
}

fn print_help (program: &str, opts: Options) {
	let brief = format!("Help: {} [options]", program);
	println!("{}", opts.usage(&brief));
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
		process::exit(3);	
	}

}

fn memory_state () -> String {

	let list_output =
		match process::Command::new ("free")
			.arg ("-b".to_string ())
			.output () {
		Ok (output) => { output }
		Err (_) => { return "ERROR".to_string(); }
	};

	String::from_utf8_lossy(&list_output.stdout).to_string()
}


fn main () {

	parse_options ();

	let state = memory_state ();

	if state == "ERROR".to_string() {
		println!("MEM UNKNOWN: Could not execute memory check command."); 
		process::exit(3);	
	}

	let mut state_vector: Vec<&str> = state.split('\n').collect();
	state_vector = state_vector[2].split(' ').collect();

	let mut index = 2;
	while state_vector[index].is_empty() {
		index = index + 1;
	}

	let mut memory_used : f64 = match state_vector[index].parse() {
		Ok (f64) => { f64 }
		Err (_) => { 
			println!("UNKNOWN: The memory used data is incorrect."); 
			process::exit(3);	
		}
	};
	memory_used = memory_used / 1073741824.0;

	index = index + 1;
	while state_vector[index].is_empty() {
		index = index + 1;
	}

	let mut memory_limit : f64 = match state_vector[index].parse() {
		Ok (f64) => { f64 }
		Err (_) => { 
			println!("UNKNOWN: The memory limit data is incorrect."); 
			process::exit(3);	
		}
	};


	memory_limit = memory_limit + memory_used;
	memory_limit = memory_limit / 1073741824.0;

	let memory_used_percentage = memory_used / memory_limit;

	let mut num_decimals = 0;
	if memory_limit < 10.0 { num_decimals = 2; }
	else if memory_limit < 100.0 { num_decimals = 1; }

	let mem_quota_used = format!("{0:.1$}", memory_used, num_decimals);
	let mem_quota_limit = format!("{0:.1$}", memory_limit, num_decimals);
	let mem_quota_percentage = format!("{0:.1$}", memory_used_percentage * 100.0, 0);

	println!("MEM OK: {} GiB {}%, limit {} GiB. | memory={}%;;;;", mem_quota_used, mem_quota_percentage, mem_quota_limit, mem_quota_percentage);
	
	process::exit(0);
}


