//Rust file
#![feature(env)]
#![feature(core)]
#![feature(io)]
#![feature(std_misc)]

extern crate getopts;

use getopts::Options;
use std::env;
use std::option::{ Option };
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
			"w",
			"warning",
			"warning usage quota level",
			"<warning-level>");

	opts.reqopt (
			"c",
			"critical",
			"critical usage quota level",
			"<critical-level>");

	opts.reqopt (
			"r",
			"rootfs",
			"root of the file system in which the checks will be performed",
			"<rootfs>");

	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("check_disk_quota", opts);
			return None;
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_disk_quota", opts);
		return None;
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
		match Command::new ("sudo")
			.arg ("btrfs".to_string ())
			.arg ("subvolume".to_string())
			.arg ("list".to_string())
			.arg ("/btrfs".to_string())
			.output () {
		Ok (output) => { output }
		Err (err) => { return format!("DISK ERROR: {}.", err); }
	};

	let subvolume = String::from_utf8_lossy(list_output.output.as_slice()).to_string();

	let subvolume_lines: Vec<&str> = subvolume.as_slice().split('\n').collect(); 

	let mut rootfs_id = "0/".to_string();	
	let mut found = false;

	let mut to_search: String = "lxc/".to_string() +rootfs.as_slice();
	to_search = to_search + "/rootfs\n";

	for line in subvolume_lines.iter() { 

		let str_line: String = line.to_string() + "\n";
		if str_line.contains(to_search.as_slice()) { 
			let rootfs_info: Vec<&str> = line.as_slice().split(' ').collect();
			rootfs_id = rootfs_id.to_string() + rootfs_info[1];
			found = true;
			break;
		}	
	}

	if !found { return "DISK ERROR".to_string(); }

	let qgroup_output = 
		match Command::new ("sudo")
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

	let qgroup = String::from_utf8_lossy(qgroup_output.output.as_slice()).to_string(); 

	let qgroup_lines: Vec<&str> = qgroup.as_slice().split('\n').collect();

	if qgroup_lines.len() == 1 { return "DISK ERROR".to_string(); }

	let mut disk_used_str = "".to_string();
	let mut disk_limit_str = "".to_string();
	found = false;

	for line in qgroup_lines.iter() { 

		if line.contains(rootfs_id.as_slice()) {

			let disk_info: Vec<&str> = line.as_slice().split(' ').collect();


			let mut index = 1; 
			while disk_info[index].as_slice().is_empty() {
				index = index + 1; 
			}

			disk_used_str = disk_info[index].to_string();

			index = index + 1;
			while disk_info[index].as_slice().is_empty() {
				index = index + 1; 
			}
			index = index + 1; 

			while disk_info[index].as_slice().is_empty() {
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

	let disk_used_quota = f64::to_str_exact(disk_used, num_decimals);
	let disk_limit_quota = f64::to_str_exact(disk_limit, num_decimals);
	let disk_used_percentage_quota = f64::to_str_exact(disk_used_percentage * 100.0, 0);

	let warning_quota_level = f64::to_str_exact(warning_level * 100.0, 0);
	let critical_quota_level = f64::to_str_exact(critical_level * 100.0, 0);

	if disk_limit == 0.0 {
		println!("DISK-Q OK: {} GiB used, no limit.", disk_used_quota);
		return "OK".to_string();
	}
	else if disk_used_percentage < warning_level {
		println!("DISK-Q OK: {} GiB {}%, limit {} GiB, warning {}%.", disk_used_quota, disk_used_percentage_quota, disk_limit_quota, warning_quota_level);
		return "OK".to_string();
	}
	else if disk_used_percentage >= warning_level && disk_used_percentage < critical_level {
		println!("DISK-Q WARNING: {} GiB {}%, limit {} GiB, critical {}%.", disk_used_quota, disk_used_percentage_quota, disk_limit_quota, critical_quota_level);
		return "WARNING".to_string();
	}
	else {
		println!("DISK-Q CRITICAL: {} GiB {}%, limit {} GiB, critical {}%.", disk_used_quota, disk_used_percentage_quota, disk_limit_quota, critical_quota_level);
		return "CRITICAL".to_string();
	}
}


fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => { return }
	};

	let disk_warning : f64 = match opts.warning.as_slice().parse() {
		Ok (f64) => { f64 }
		Err (_) => { 
			println!("UNKNOWN: Warning level must be a value between 0.0 and 1.0."); 
			env::set_exit_status(3);	
			return;
		}
	};
	
	let disk_critical : f64 = match opts.critical.as_slice().parse() {
		Ok (f64) => { f64 }
		Err (_) => { 
			println!("UNKNOWN: Critical level must be a value between 0.0 and 1.0."); 
			env::set_exit_status(3);	
			return;
		}
	};

	let disk_str = check_disk(opts.rootfs.as_slice(), disk_warning, disk_critical);
	if disk_str.contains("DISK ERROR") {
		println!("DISK-Q UNKNOWN: Could not execute disk check: {}.", disk_str); 
		env::set_exit_status(3);	
	}
	else if disk_str == "OK" {
		env::set_exit_status(0);	
	}
	else if disk_str == "WARNING" {
		env::set_exit_status(1);	
	}
	else if disk_str == "CRITICAL" {
		env::set_exit_status(2);	
	}
	else {
		println!("DISK-Q UNKNOWN: Could not execute disk check. Unknown error."); 
		env::set_exit_status(3);	
	}
	
	return;
}


