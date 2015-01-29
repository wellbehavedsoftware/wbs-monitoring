#![allow(unstable)]
extern crate getopts;

use getopts::{ optflag, reqopt, getopts, short_usage, usage, OptGroup };
use std::os;
use std::option::{ Option };
use std::io::{ Command };
use std::f64;

fn print_usage (program: &str, opts: &[OptGroup]) {
	println! ("{}", short_usage (program, opts));
}

fn print_help (program: &str, opts: &[OptGroup]) {
	println! ("{}", usage (program, opts));
}

struct Options {
	root: String,
	warning: String,
	critical: String,
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
			"root",
			"root of the filesystem to check",
			"<fs-root>"),

		reqopt (
			"w",
			"warning",
			"warning memory usage threshold",
			"<warning-threshold>"),

		reqopt (
			"c",
			"critical",
			"critical memory usage threshold",
			"<critical-threshold>"),

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

	let root = matches.opt_str ("root").unwrap ();
	let warning = matches.opt_str ("warning").unwrap ();
	let critical = matches.opt_str ("critical").unwrap ();

	return Some (Options {
		root: root,
		warning: warning,
		critical: critical,
	});

}

fn disk_state () -> String {

	let list_output =
		match Command::new ("df")
			.output () {
		Ok (output) => { output }
		Err (_) => { return "DISK ERROR".to_string(); }
	};

	String::from_utf8_lossy(list_output.output.as_slice()).to_string()
}


fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => { return }
	};

	let state = disk_state ();

	if state == "DISK ERROR".to_string() {
		println!("DISK UNKNOWN: Could not execute memory check command."); 
		os::set_exit_status(3);	
		return;
	}

	let to_check: String = opts.root;

	let warning_level: f64 = match opts.warning.as_slice().parse() {
		Some (f64) => { f64 }
		None => {
			println!("DISK UNKNOWN: Warning level must be a value between 0.0 and 1.0."); 
			os::set_exit_status(3);	
			return;
		}
	};

	let critical_level: f64 = match opts.critical.as_slice().parse() {
		Some (f64) => { f64 }
		None => {
			println!("DISK UNKNOWN: Critical level must be a value between 0.0 and 1.0."); 
			os::set_exit_status(3);	
			return;
		}
	};

	let state_vector: Vec<&str> = state.as_slice().split('\n').collect();

	let mut interest_line: &str = "";
	let mut found = false;

	for line in state_vector.iter() { 

		let str_line: String = line.to_string() + "\n";
		let to_check_aux = format!("{}\n", to_check.as_slice());
				
		if str_line.contains(to_check_aux.as_slice()) { 
			interest_line = line.as_slice();
			found = true;
			break;
		}	

	}

	if !found { 
		println!("DISK UNKNOWN: The {} volume does not exist.", to_check); 
		os::set_exit_status(3);	
		return;
	}

	let line_vector: Vec<&str> = interest_line.as_slice().split(' ').collect();	
	let percentage_vector: Vec<&str> = line_vector[line_vector.len()-2].as_slice().split('%').collect();

	let disk_quota_percentage = percentage_vector[0];
	let percentage_aux: Option<f64> = disk_quota_percentage.parse();
	let mut disk_used_percentage: f64 = percentage_aux.unwrap();
	disk_used_percentage = disk_used_percentage / 100.0;
	let mut index = 1;
	while line_vector[index].is_empty() { index = index + 1; }

	let disk_limit_aux: Option<f64> = line_vector[index].parse();
	let disk_limit: f64 = disk_limit_aux.unwrap();
	let disk_quota_limit = f64::to_str_exact(disk_limit / 1048576.0, 2);

	let disk_used = disk_used_percentage * disk_limit;
	let disk_quota_used = f64::to_str_exact(disk_used / 1048576.0, 2);

	let warning_quota_level = f64::to_str_exact(warning_level * 100.0, 2);
	let critical_quota_level = f64::to_str_exact(critical_level * 100.0, 2);

	if disk_used_percentage < warning_level {
		println!("DISK OK: {} GiB {}%, limit {} GiB, warning {}%.", disk_quota_used, disk_quota_percentage, disk_quota_limit, warning_quota_level);
		os::set_exit_status(0);
	}
	else if disk_used_percentage >= warning_level && disk_used_percentage < critical_level {
		println!("DISK WARNING: {} GiB {}%, limit {} GiB, critical {}%.", disk_quota_used, disk_quota_percentage, disk_quota_limit, critical_quota_level);
		os::set_exit_status(1);
	}
	else {
		println!("DISK CRITICAL: {} GiB {}%, limit {} GiB, critical {}%.", disk_quota_used, disk_quota_percentage, disk_quota_limit, critical_quota_level);
		os::set_exit_status(2);
	}

	return;
}
