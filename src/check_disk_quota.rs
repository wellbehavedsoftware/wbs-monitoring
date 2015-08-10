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
			"",
			"warning",
			"warning usage quota level",
			"<warning-level>");

	opts.reqopt (
			"",
			"critical",
			"critical usage quota level",
			"<critical-level>");

	opts.reqopt (
			"",
			"rootfs",
			"root of the file system in which the checks will be performed",
			"<rootfs>");

	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("check_disk_quota", opts);
			process::exit(3);
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_disk_quota", opts);
		process::exit(3);	
	}

	let warning = matches.opt_str ("warning").unwrap ();
	let critical = matches.opt_str ("critical").unwrap ();
	let rootfs = matches.opt_str ("rootfs").unwrap ();


	return Some (Opts {
		warning: warning,
		critical: critical,
		rootfs: rootfs,

	});

}


fn check_disk(rootfs: &str, warning_level: f64, critical_level: f64) -> String {

	let list_output =
		match process::Command::new ("sudo")
			.arg ("btrfs".to_string ())
			.arg ("subvolume".to_string())
			.arg ("list".to_string())
			.arg ("/btrfs".to_string())
			.output () {
		Ok (output) => { output }
		Err (err) => { return format!("DISK ERROR: {}.", err); }
	};

	let subvolume = String::from_utf8_lossy(&list_output.stdout).to_string();

	let subvolume_lines: Vec<&str> = subvolume.split('\n').collect(); 

	let mut rootfs_id = "0/".to_string();	
	let mut found = false;

	let mut to_search: String = "lxc/".to_string() + &rootfs;
	to_search = to_search + "/rootfs\n";

	for line in subvolume_lines.iter() { 

		let str_line: String = line.to_string() + "\n";
		if str_line.contains(&to_search) { 
			let rootfs_info: Vec<&str> = line.split(' ').collect();
			rootfs_id = rootfs_id.to_string() + rootfs_info[1];
			found = true;
			break;
		}	
	}

	if !found { return "DISK ERROR".to_string(); }

	let qgroup_output = 
		match process::Command::new ("sudo")
			.arg ("btrfs".to_string ())
			.arg ("qgroup".to_string())
			.arg ("show".to_string())
			.arg ("/btrfs".to_string())
			.arg ("-e".to_string())
			.arg ("-r".to_string())
			.output () {
		Ok (output) => { output }
		Err (err) => { return format!("DISK ERROR: {}.", err); }
	};

	let qgroup = String::from_utf8_lossy(&qgroup_output.stdout).to_string(); 

	let qgroup_lines: Vec<&str> = qgroup.split('\n').collect();

	if qgroup_lines.len() == 1 { return "DISK ERROR".to_string(); }

	let mut disk_used_str = "".to_string();
	let mut disk_limit_str = "".to_string();
	found = false;

	for line in qgroup_lines.iter() { 

		if line.contains(&rootfs_id) {

			let disk_info: Vec<&str> = line.split(' ').collect();


			let mut index = 1; 
			while disk_info[index].is_empty() {
				index = index + 1; 
			}

			disk_used_str = disk_info[index].to_string();

			index = index + 1;
			while disk_info[index].is_empty() {
				index = index + 1; 
			}
			index = index + 1; 

			while disk_info[index].is_empty() {
				index = index + 1; 
			}

			disk_limit_str = disk_info[index].to_string();

			found = true;
			break;
		}		
	}

	if !found { return "DISK ERROR".to_string(); }

	let mut disk_used : f64 = match disk_used_str.parse() {
		Ok (f64) => { f64 }
		Err (_) => { return "MEM ERROR".to_string(); }
	};
	disk_used = disk_used / 1073741824.0;

	let mut disk_limit : f64 = match disk_limit_str.parse() {
		Ok (f64) => { f64 }
		Err (_) => { return "MEM ERROR".to_string(); }
	};
	disk_limit = disk_limit / 1073741824.0;
	
	let disk_used_percentage = disk_used / disk_limit;

	let mut num_decimals = 0;
	if disk_limit < 10.0 { num_decimals = 2; }
	else if disk_limit < 100.0 { num_decimals = 1; }

	let disk_used_quota = format!("{0:.1$}", disk_used, num_decimals);
	let disk_limit_quota = format!("{0:.1$}", disk_limit, num_decimals);
	let disk_used_percentage_quota = format!("{0:.1$}", disk_used_percentage * 100.0, 0);

	let warning_quota_level = format!("{0:.1$}", warning_level * 100.0, 0);
	let critical_quota_level = format!("{0:.1$}", critical_level * 100.0, 0);

	if disk_limit == 0.0 {
		println!("DISK-Q OK: {} GiB used, no limit.", disk_used_quota);
		return "OK".to_string();
	}
	else if disk_used_percentage < warning_level {
		println!("DISK-Q OK: {} GiB {}%, limit {} GiB, warning {}%. | disk={}%;50.0;75.0;;", disk_used_quota, disk_used_percentage_quota, disk_limit_quota, warning_quota_level, disk_used_percentage_quota);
		return "OK".to_string();
	}
	else if disk_used_percentage >= warning_level && disk_used_percentage < critical_level {
		println!("DISK-Q WARNING: {} GiB {}%, limit {} GiB, critical {}%. | disk={}%;50.0;75.0;;", disk_used_quota, disk_used_percentage_quota, disk_limit_quota, critical_quota_level, disk_used_percentage_quota);
		return "WARNING".to_string();
	}
	else {
		println!("DISK-Q CRITICAL: {} GiB {}%, limit {} GiB, critical {}%. | disk={}%;50.0;75.0;;", disk_used_quota, disk_used_percentage_quota, disk_limit_quota, critical_quota_level, disk_used_percentage_quota);
		return "CRITICAL".to_string();
	}
}


fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => { return }
	};

	let disk_warning : f64 = match opts.warning.parse() {
		Ok (f64) => { f64 }
		Err (_) => { 
			println!("UNKNOWN: Warning level must be a value between 0.0 and 1.0."); 
			process::exit(3);	
		}
	};
	
	let disk_critical : f64 = match opts.critical.parse() {
		Ok (f64) => { f64 }
		Err (_) => { 
			println!("UNKNOWN: Critical level must be a value between 0.0 and 1.0."); 
			process::exit(3);	
		}
	};

	let disk_str = check_disk(&opts.rootfs, disk_warning, disk_critical);
	if disk_str.contains("DISK ERROR") {
		println!("DISK-Q UNKNOWN: Could not execute disk check: {}.", disk_str); 
		process::exit(3);	
	}
	else if disk_str == "OK" {
		process::exit(0);	
	}
	else if disk_str == "WARNING" {
		process::exit(1);	
	}
	else if disk_str == "CRITICAL" {
		process::exit(2);	
	}
	else {
		println!("DISK-Q UNKNOWN: Could not execute disk check. Unknown error."); 
		process::exit(3);	
	}

}


