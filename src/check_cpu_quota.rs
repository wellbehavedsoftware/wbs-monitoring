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
	warning: String,
	critical: String,
}

fn parse_options () -> Option<Opts> {

	let args = env::args ();

	let mut opts = Options::new();

	opts.optflag (	
			"h",
			"help",
			"print this help menu");

	opts.reqopt (
			"w",
			"warning",
			"warning usage quota level",
			"<warning-level>");

	opts.reqopt (
			"c",
			"critical",
			"critical usage quota level",
			"<critical-level>");

	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("check_cpu_quota", opts);
			process::exit(3);
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_cpu_quota", opts);
		process::exit(3);
	}

	let warning = matches.opt_str ("warning").unwrap ();
	let critical = matches.opt_str ("critical").unwrap ();

	return Some (Opts {
		warning: warning,
		critical: critical,
	});

}


fn check_cpu(warning_level: f64, critical_level: f64) -> String {

	let stat_output =
		match process::Command::new ("cat")
			.arg ("/proc/stat".to_string ())
			.output () {
		Ok (output) => { output }
		Err (err) => { return format!("CPU ERROR: {}.", err); }
	};	
	let stat = String::from_utf8_lossy(&stat_output.stdout).to_string();
	let stat_lines: Vec<&str> = stat.split('\n').collect();
	let stat_cpu: Vec<&str> = stat_lines[0].split(' ').collect();

	let user : f64 = match stat_cpu[2].parse() {
		Ok (f64) => { f64 }
		Err (_) => { return "CPU ERROR".to_string(); }
	};
	
	let kernel : f64 = match stat_cpu[4].parse() {
		Ok (f64) => { f64 }
		Err (_) => { return "CPU ERROR".to_string(); }
	};

	let busy = user + kernel;

	let iddle : f64 = match stat_cpu[5].parse() {
		Ok (f64) => { f64 }
		Err (_) => { return "CPU ERROR".to_string(); }
	};

	let cpu_quota = busy / (busy + iddle);
	let cpu_quota_used = format!("{0:.1$}", cpu_quota * 100.0, 1);

	let warning_level_quota = format!("{0:.1$}", warning_level * 100.0, 1);
	let critical_level_quota = format!("{0:.1$}", critical_level * 100.0, 1);

	if cpu_quota < warning_level {
		println!("CPU OK: used {}%, warning {}%. | cpu={}%;20.0;50.0;;", cpu_quota_used, warning_level_quota, cpu_quota_used);
		return "OK".to_string();
	}
	else if cpu_quota >= warning_level && cpu_quota < critical_level {
		println!("CPU WARNING: used {}%, critical {}%. | cpu={}%;20.0;50.0;;", cpu_quota_used, critical_level_quota, cpu_quota_used);
		return "WARNING".to_string();
	}
	else {
		println!("CPU CRITICAL: used {}%, critical {}%. | cpu={}%;20.0;50.0;;", cpu_quota_used, critical_level_quota, cpu_quota_used);
		return "CRITICAL".to_string();
	}
	
}

fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => { return }
	};

	let cpu_warning : f64 = match opts.warning.parse() {
		Ok (f64) => { f64 }
		Err (_) => {
			println!("UNKNOWN: Warning level must be a value between 0.0 and 1.0."); 
			process::exit(3);
		}
	};
	
	let cpu_critical : f64 = match opts.critical.parse() {
		Ok (f64) => { f64 }
		Err (_) => {
			println!("UNKNOWN: Critical level must be a value between 0.0 and 1.0."); 
			process::exit(3);
		}
	};

	let cpu_str = check_cpu(cpu_warning, cpu_critical);
	if cpu_str.contains("CPU ERROR") {
		println!("CPU UNKNOWN: Could not execute CPU check: {}.", cpu_str); 
		process::exit(3);	
	}
	else if cpu_str == "OK" {
		process::exit(0);	
	}
	else if cpu_str == "WARNING" {
		process::exit(1);	
	}
	else if cpu_str == "CRITICAL" {
		process::exit(2);	
	}
	else {
		println!("CPU UNKNOWN: Could not execute disk check. Unknown error."); 
		process::exit(3);	
	}
}

