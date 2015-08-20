//Rust file
extern crate getopts;
extern crate regex;

use getopts::Options;
use std::env;
use std::process;
use std::option::{ Option };
use regex::*;

fn print_usage (program: &str, opts: Options) {
	let brief = format!("Usage: {} [options]", program);
	println!("{}", opts.usage(&brief));
}

fn print_help (program: &str, opts: Options) {
	let brief = format!("Help: {} [options]", program);
	println!("{}", opts.usage(&brief));
}

struct Opts {
	route: String,
	warning: String,
	critical: String,
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
			"route",
			"route to the btrfs filesystem",
			"<route>");
	
	opts.reqopt (
			"",
			"warning",
			"queue time for which the script returns a warning state",
			"<warning>");

	opts.reqopt (
			"",
			"critical",
			"queue time for which the script returns a critical state",
			"<critical>");

	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("check_btrfs", opts);
			process::exit(3);
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_btrfs", opts);
		process::exit(3);
	}

	let route = matches.opt_str ("route").unwrap ();
	let warning = matches.opt_str ("warning").unwrap ();
	let critical = matches.opt_str ("critical").unwrap ();

	return Some (Opts {
		route: route,
		warning: warning,
		critical: critical,
	});

}

fn check_btrfs (route: &str, warning_th: f64, critical_th: f64) -> (String, String) {

	let mut btrfs_fsshow_output: String;
	
	//check emails list

	let output =
		match process::Command::new ("sudo")
		.arg ("btrfs".to_string ())
		.arg ("filesystem".to_string ())
		.arg ("show".to_string ())
		.arg (route)
		.output () {
	Ok (output) => { output }
	Err (err) => { return (format!("BTRFS-UNKNOWN: {}.", err), "".to_string()); }
	};

	btrfs_fsshow_output = String::from_utf8_lossy(&output.stdout).to_string();

	let btrfs_lines: Vec<&str> = btrfs_fsshow_output.split("\n").collect();
	
	if btrfs_lines.len() < 5 { return (format!("BTRFS-UNKNOWN: No devices present."), "".to_string()); }

	let used_line = btrfs_lines[1];

	let device_line = btrfs_lines[2];

	let re = Regex::new(r"(\d+.\d+)([a-zA-Z]{3,})").unwrap();
	let mut size: f64 = 0.0;
	let mut used: f64 = 0.0;
	let mut size_str = "".to_string();
	let mut used_str = "".to_string();

	// Get total space
	for cap in re.captures_iter(device_line) {

		let value_str = cap.at(1).unwrap_or("");
		let mut value : f64 = match value_str.parse() {
			Ok (f64) => { f64 }
			Err (_) => {
				println!("BTRFS-UNKNOWN: Error while executing the command!"); 
				process::exit(3);
			}
		};

		let unit = cap.at(2).unwrap_or("");

		if unit == "MiB".to_string() { value = value / 1024.0; }
		if unit == "TiB".to_string() { value = value * 1024.0; }

		size = value; size_str = format!("{} {}", value_str, unit); 
		break;
		
	}

	// Get used space
	for cap in re.captures_iter(used_line) {

		let value_str = cap.at(1).unwrap_or("");
		let mut value : f64 = match value_str.parse() {
			Ok (f64) => { f64 }
			Err (_) => {
				println!("BTRFS-UNKNOWN: Error while executing the command!"); 
				process::exit(3);
			}
		};

		let unit = cap.at(2).unwrap_or("");

		if unit == "MiB".to_string() { value = value / 1024.0; }
		if unit == "TiB".to_string() { value = value * 1024.0; }

		used = value; used_str = format!("{} {}", value_str, unit);
		break;
		
	}

	let used_quota = used / size;
	let used_quota_str = format!("{0:.1$}", used_quota * 100.0, 1);
	let warning_str = format!("{0:.1$}", warning_th * 100.0, 1);
	let critical_str = format!("{0:.1$}", critical_th * 100.0, 1);

	if used_quota > warning_th && used_quota < critical_th {

		return (format!("BTRFS-WARNING: {} {}%, limit {}, critical {}%.\n", used_str, used_quota_str, size_str, critical_str), format!("{}", used_quota_str));

	}
	else if used_quota > critical_th {

		return (format!("BTRFS-CRITICAL: {} {}%, limit {}, critical {}%.\n", used_str, used_quota_str, size_str, critical_str), format!("{}", used_quota_str));

	}
	else {

		return (format!("BTRFS-OK: {} {}%, limit {}, warning {}%.\n", used_str, used_quota_str, size_str, warning_str), format!("{}", used_quota_str));

	}
}


fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => { 
			println!("BTRFS-UNKNOWN: Wrong arguments.");
			process::exit(3);
		}
	};

	let route = &opts.route;

	let warning : f64 = match opts.warning.parse() {
		Ok (f64) => { f64 }
		Err (_) => {
			println!("BTRFS-UNKNOWN: The warning threshold must be a double between 0.0 and 1.0!"); 
			process::exit(3);
		}
	};
	let critical : f64 = match opts.critical.parse() {
		Ok (f64) => { f64 }
		Err (_) => {
			println!("BTRFS-UNKNOWN: The critical threshold must be a double between 0.0 and 1.0!"); 
			process::exit(3);
		}
	};

	let (result, quota) = check_btrfs (route, warning, critical);

	println!("{} | btrfs_disk_used={}%;{};{};;", result, quota, warning*100.0, critical*100.0);

	if result.contains("CRITICAL") {

		process::exit(2);

	}
	else if result.contains("WARNING") {

		process::exit(1);

	}
	else if result.contains("OK") {

		process::exit(0);
	}
	else {
		
		process::exit(3);
	}
	
}
