//Rust file
extern crate getopts;

use getopts::Options;
use std::env;
use std::option::{ Option };
use std::process;

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

	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("check_mem_quota", opts);
			return None;
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_mem_quota", opts);
		return None;
	}

	let rootfs = matches.opt_str ("rootfs").unwrap ();

	return Some (Opts {
		rootfs: rootfs,
	});

}

fn check_mem(rootfs: &str) -> String {

	let usage_output =
		match process::Command::new ("sudo")
			.arg ("/usr/bin/lxc-cgroup".to_string ())
			.arg ("--name".to_string())
			.arg (rootfs.to_string())
			.arg ("memory.usage_in_bytes".to_string())
			.output () {
		Ok (output) => { output }
		Err (_) => { return "MEM ERROR".to_string(); }
	};

	let limit_output =
		match process::Command::new ("sudo")
			.arg ("/usr/bin/lxc-cgroup".to_string ())
			.arg ("--name".to_string())
			.arg (rootfs.to_string())
			.arg ("memory.limit_in_bytes".to_string())
			.output () {
		Ok (output) => { output }
		Err (_) => { return "MEM ERROR".to_string(); }
	};

	let usage_str = String::from_utf8_lossy(&usage_output.stdout).trim().to_string();
	let limit_str = String::from_utf8_lossy(&limit_output.stdout).trim().to_string();

	if usage_str.contains("is not running")	|| limit_str.contains("is not running") {
		return "MEM ERROR".to_string();
	}

	let mut mem_used : f64 = match usage_str.parse() {
		Ok (f64) => { f64 }
		Err (_) => { return "MEM ERROR".to_string(); }
	};
	mem_used = mem_used / 1073741824.0;

	let mut mem_limit : f64 = match limit_str.parse() {
		Ok (f64) => { f64 }
		Err (_) => { return "MEM ERROR".to_string(); }
	};
	mem_limit = mem_limit / 1073741824.0;

	let mem_used_percentage = mem_used / mem_limit;

	let mut num_decimals = 0;
	if mem_limit < 10.0 { num_decimals = 2; }
	else if mem_limit < 100.0 { num_decimals = 1; }

	let mem_used_quota = format!("{0:.1$}", mem_used, num_decimals);
	let mem_limit_quota = format!("{0:.1$}", mem_limit, num_decimals);
	let mem_used_percentage_quota = format!("{0:.1$}", mem_used_percentage * 100.0, 0);

	println!("MEM-Q OK: {} GiB {}%, limit {} GiB. | memory={}%;;;;", mem_used_quota, mem_used_percentage_quota, mem_limit_quota, mem_used_percentage_quota);
	return "OK".to_string();
	
}


fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => { return }
	};
	
	let mem_str = check_mem(&opts.rootfs);
	if mem_str == "MEM ERROR" {
		println!("MEM-Q UNKNOWN: Could not execute memory check. Shell commands failed to execute."); 
		process::exit(3);	
	}
	else if mem_str == "OK" {
		process::exit(0);	
	}
	else {
		println!("MEM-Q UNKNOWN: Could not execute mem check. Unknown error."); 
		process::exit(3);	
	}

}



