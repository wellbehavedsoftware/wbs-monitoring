#![allow(unstable)]
extern crate getopts;

use getopts::{ optflag, reqopt, getopts, short_usage, usage, OptGroup };
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

struct Options {
	rootfs: String,
}

fn parse_options () -> Option<Options> {

	let args: Vec<String> = os::args ();

	let program = args [0].clone ();

	let opts = &[

		optflag (
			"h",
			"help",
			"print this help menu"),

		reqopt (
			"r",
			"rootfs",
			"root of the file system in which the checks will be performed",
			"<rootfs>"),

	];

	let matches = match getopts (args.tail (), opts) {
		Ok (m) => { m }
		Err (_) => {
			print_usage (program.as_slice (), opts);
			return None;
		}
	};

	if matches.opt_present ("help") {
		print_help (program.as_slice (), opts);
		return None;
	}

	if ! matches.free.is_empty () {
		print_usage (program.as_slice (), opts);
		return None;
	}

	let rootfs = matches.opt_str ("rootfs").unwrap ();


	return Some (Options {
		rootfs: rootfs,
	});

}

fn check_mem(rootfs: &str) -> String {

	let usage_output =
		match Command::new ("sudo")
			.arg ("/usr/bin/lxc-cgroup".to_string ())
			.arg ("--name".to_string())
			.arg (rootfs.to_string())
			.arg ("memory.usage_in_bytes".to_string())
			.output () {
		Ok (output) => { output }
		Err (_) => { return "MEM ERROR".to_string(); }
	};

	let limit_output =
		match Command::new ("sudo")
			.arg ("/usr/bin/lxc-cgroup".to_string ())
			.arg ("--name".to_string())
			.arg (rootfs.to_string())
			.arg ("memory.limit_in_bytes".to_string())
			.output () {
		Ok (output) => { output }
		Err (_) => { return "MEM ERROR".to_string(); }
	};

	let usage_str = String::from_utf8_lossy(usage_output.output.as_slice()).trim().to_string();
	let limit_str = String::from_utf8_lossy(limit_output.output.as_slice()).trim().to_string();

	if usage_str.contains("is not running")	|| limit_str.contains("is not running") {
		return "MEM ERROR".to_string();
	}

	let mem_used_aux: Option<f64> = usage_str.parse();
	if mem_used_aux.is_none() { return "MEM ERROR".to_string(); }
	let mem_used: f64 = mem_used_aux.unwrap();

	let mem_limit_aux: Option<f64> = limit_str.parse();
	if mem_limit_aux.is_none() { return "MEM ERROR".to_string(); }
	let mem_limit: f64 = mem_limit_aux.unwrap();

	let mem_used_percentage = mem_used / mem_limit;

	let mem_used_quota = f64::to_str_exact(mem_used / 1073741824.0, 2);
	let mem_limit_quota = f64::to_str_exact(mem_limit / 1073741824.0, 2);
	let mem_used_percentage_quota = f64::to_str_exact(mem_used_percentage * 100.0, 2);

	println!("MEM-Q OK: {} GiB {}%, limit {} GiB.", mem_used_quota, mem_used_percentage_quota, mem_limit_quota);
	return "OK".to_string();
	
}


fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => { return }
	};
	
	let mem_str = check_mem(opts.rootfs.as_slice());
	if mem_str == "MEM ERROR" {
		println!("MEM-Q UNKNOWN: Could not execute memory check. Shell commands failed to execute."); 
		os::set_exit_status(3);	
	}
	else if mem_str == "OK" {
		os::set_exit_status(0);	
	}
	else {
		println!("MEM-Q UNKNOWN: Could not execute mem check. Unknown error."); 
		os::set_exit_status(3);	
	}
	
	return;
}



