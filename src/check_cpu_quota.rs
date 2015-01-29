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
			"w",
			"warning",
			"warning usage quota level",
			"<warning-level>"),

		reqopt (
			"c",
			"critical",
			"critical usage quota level",
			"<critical-level>"),
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

	let warning = matches.opt_str ("warning").unwrap ();
	let critical = matches.opt_str ("critical").unwrap ();

	return Some (Options {
		warning: warning,
		critical: critical,
	});

}


fn check_cpu(warning_level: f64, critical_level: f64) -> String {

	let stat_output =
		match Command::new ("cat")
			.arg ("/proc/stat".to_string ())
			.output () {
		Ok (output) => { output }
		Err (err) => { return format!("CPU ERROR: {}.", err); }
	};	
	let stat = String::from_utf8_lossy(stat_output.output.as_slice()).to_string();
	let stat_lines: Vec<&str> = stat.as_slice().split('\n').collect();
	let stat_cpu: Vec<&str> = stat_lines[0].as_slice().split(' ').collect();
	
	let user_option: Option<f64> = stat_cpu[2].parse();
	if user_option.is_none() { return "CPU ERROR".to_string(); }
	let user: f64 = user_option.unwrap();

	let kernel_option: Option<f64> = stat_cpu[4].parse();
	if kernel_option.is_none() { return "CPU ERROR".to_string(); }
	let kernel: f64 = kernel_option.unwrap();

	let busy = user + kernel;

	let iddle_option: Option<f64> = stat_cpu[5].parse();
	if iddle_option.is_none() { return "CPU ERROR".to_string(); }
	let iddle: f64 = iddle_option.unwrap();

	let cpu_quota = busy / (busy + iddle);
	let cpu_quota_used = f64::to_str_exact(cpu_quota * 100.0, 2);

	let warning_level_quota = f64::to_str_exact(warning_level * 100.0, 2);
	let critical_level_quota = f64::to_str_exact(critical_level * 100.0, 2);

	if cpu_quota < warning_level {
		println!("CPU OK: used {}%, warning {}%.", cpu_quota_used, warning_level_quota);
		return "OK".to_string();
	}
	else if cpu_quota >= warning_level && cpu_quota < critical_level {
		println!("CPU WARNING: used {}%, critical {}%.", cpu_quota_used, critical_level_quota);
		return "WARNING".to_string();
	}
	else {
		println!("CPU CRITICAL: used {}%, critical {}%.", cpu_quota_used, critical_level_quota);
		return "CRITICAL".to_string();
	}
	
}

fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => { return }
	};
	
	let cpu_warning = match opts.warning.as_slice().parse() {
		Some (f64) => { f64 }
		None => {
			println!("UNKNOWN: Warning level must be a value between 0.0 and 1.0."); 
			os::set_exit_status(3);	
			return;
		}
	};

	let cpu_critical = match opts.critical.as_slice().parse() {
		Some (f64) => { f64 }
		None => {
			println!("UNKNOWN: Critical level must be a value between 0.0 and 1.0."); 
			os::set_exit_status(3);	
			return;
		}
	};

	let cpu_str = check_cpu(cpu_warning, cpu_critical);
	if cpu_str.contains("CPU ERROR") {
		println!("CPU UNKNOWN: Could not execute CPU check: {}.", cpu_str); 
		os::set_exit_status(3);	
	}
	else if cpu_str == "OK" {
		os::set_exit_status(0);	
	}
	else if cpu_str == "WARNING" {
		os::set_exit_status(1);	
	}
	else if cpu_str == "CRITICAL" {
		os::set_exit_status(2);	
	}
	else {
		println!("CPU UNKNOWN: Could not execute disk check. Unknown error."); 
		os::set_exit_status(3);	
	}
	
	return;
}

