//Rust file

extern crate getopts;
extern crate chrono;
extern crate regex;

use getopts::Options;
use std::env;
use std::option::{ Option };
use std::process;
use chrono::UTC;
use chrono::offset::TimeZone;
use std::io::Write;
use std::io::Read;
use std::fs::File;
use std::path::Path;
use std::os::unix::fs::MetadataExt;
use regex::Regex;

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
	warning: String,
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
			"rootfs",
			"root of the file system in which the checks will be performed",
			"<rootfs>");

	opts.reqopt (
			"",
			"warning",
			"package update warning threshold in hours",
			"<update-warning-threshold-hours>");

	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("check_apt", opts);
			process::exit(3);
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_apt", opts);
		process::exit(3);
	}

	
	let rootfs = matches.opt_str ("rootfs").unwrap ();
	let warning = matches.opt_str ("warning").unwrap ();

	return Some (Opts {
		rootfs: rootfs,
		warning: warning,
	});

}

fn check_last_update (rootfs: &str) -> String {

	let update_stamp: String;	
	
	if rootfs.is_empty() {
		// Get last modification datetime from file metadata	
		let success_stamp_route = "/var/lib/apt/periodic/update-success-stamp".to_string();

		let metadata = match std::fs::metadata(&success_stamp_route) {
			Ok(m) => { m }
			Err(e) => { return format!("APT-UNKNOWN: Failed to read {}: {}", success_stamp_route, e); } 
		};

		update_stamp = UTC.timestamp(metadata.mtime(), metadata.mtime_nsec() as u32).to_string();
	}
	else {
		let stat_output =
			match process::Command::new ("sudo")
				.arg ("lxc-attach".to_string ())
				.arg ("--name".to_string ())
				.arg (&rootfs)
				.arg ("--".to_string ())
				.arg ("stat".to_string ())
				.arg ("-c".to_string ())
				.arg ("%y".to_string ())
				.arg ("/var/lib/apt/periodic/update-success-stamp".to_string ())
				.output () {
			Ok (output) => { output }
			Err (err) => { return format!("LAST UPDATE ERROR: {}.", err); }
		};
		update_stamp = String::from_utf8_lossy(&stat_output.stdout).to_string();		
	}
	

	// Compare last update datetime with current datetime
	let mut day_time: Vec<&str> = update_stamp.split('.').collect();
	if day_time.len() == 1 { return "LAST UPDATE ERROR".to_string(); }

	day_time =  day_time[0].split(' ').collect();

	let date_array: Vec<&str> = day_time[0].split('-').collect();
	let time_array: Vec<&str> = day_time[1].split(':').collect();
	
	let last_update_datetime = UTC.ymd(date_array[0].parse().unwrap(), date_array[1].parse().unwrap(), date_array[2].parse().unwrap())
				      .and_hms(time_array[0].parse().unwrap(), time_array[1].parse().unwrap(), time_array[2].parse().unwrap());

	let current_datetime = UTC::now(); 

	let diff = current_datetime - last_update_datetime;

	let diffseconds = diff.num_seconds() as f64;
	let diffhours = diffseconds / 3600.0;

	return diffhours.to_string();
}


fn check_reboot(rootfs: &str) -> String {

	let mut motd: String = "".to_string();
	let mut reboot_needed = "NO".to_string();

	if rootfs.is_empty() {
		// Get the motd from the motd file
		let motd_route = "/var/run/motd.dynamic".to_string();

		let path = Path::new(&motd_route);

		let mut file = match File::open(&path) {
		    Ok(file) => { file }
		    Err(e)  => { return format!("APT-UNKNOWN: Failed to read {}: {}", motd_route, e); }
		};

		let mut motd: String = "".to_string();
		file.read_to_string(&mut motd);
	}
	else {
		let cat_output =
		match process::Command::new ("sudo")
			.arg ("lxc-attach".to_string ())
			.arg ("--name".to_string ())
			.arg (&rootfs)
			.arg ("cat".to_string ())
			.arg ("/var/run/motd.dynamic".to_string ())
			.output () {
		Ok (output) => { output }
		Err (err) => { return format!("CHECK REBOOT ERROR: {}.", err); }
		};
		motd = String::from_utf8_lossy(&cat_output.stdout).to_string();
	}

	// Check if the host or container needs reboot
	if motd.contains("System restart required") {
		reboot_needed = "YES".to_string();
	}

	return reboot_needed;
}

fn check_packages(rootfs: &str) -> (isize, String) {

	let mut packages_update_needed = "KO".to_string();

	let mut dpkg_output;
	
	// Get package list
	if rootfs.is_empty() {		

		dpkg_output =
			match process::Command::new ("dpkg")
				.arg ("--get-selections".to_string ())
				.output () {
			Ok (output) => { output }
			Err (_) => { return (0, "CHECK PACKAGES ERROR".to_string()); }
		};
	}
	else { 		

		dpkg_output =
			match process::Command::new ("sudo")
				.arg ("lxc-attach".to_string ())
				.arg ("--name".to_string ())
				.arg (&rootfs)
				.arg ("--".to_string ())
				.arg ("dpkg".to_string ())
				.arg ("--get-selections".to_string ())
				.output () {
			Ok (output) => { output }
			Err (_) => { return (0, "CHECK PACKAGES ERROR".to_string()); }
		};	
	}

	let mut selections = String::from_utf8_lossy(&dpkg_output.stdout).to_string();
	drop(dpkg_output);
	
	// Exclude the packaga that are not installed
	let expression = format!("(.+)(deinstall)\n");
	let re = Regex::new(&expression).unwrap();
	selections = re.replace_all(&selections, "");

	// Get versions of the installed packages, both installed and newest available
	let mut xargs_output =
		match process::Command::new ("xargs")
			.arg ("apt-cache".to_string())
			.arg ("policy".to_string())
		    	.stdin(std::process::Stdio::piped())
		   	.stdout(std::process::Stdio::piped())
			.spawn () {
		Ok (output) => { output }
		Err (_) => {  return (0, "CHECK PACKAGES ERROR".to_string()); }
		};

	xargs_output.stdin.unwrap().write(selections.as_bytes());

	let mut out: String = "".to_string();
	xargs_output.stdout.as_mut().unwrap().read_to_string(&mut out);

	// Check if the installed version is the latest available for each installed package
	let output_lines: Vec<&str> = out.split("\n").collect();
	if output_lines.len() == 1 { return (0, "CHECK PACKAGES ERROR".to_string()); }
	
	let mut package_list = vec![];	
	let mut i = 0;	
	
	while i < output_lines.len() {

		let line: Vec<&str> = output_lines[i].split(':').collect();
		if line[0] == "  Installed" {
			let package = format!("{}\n{}\n{}", output_lines[i-1], output_lines[i], output_lines[i+1]);
			package_list.push(package);
		}
		i = i + 1;
	}

	let (num_packages, packages_msg) = packages_updated(package_list);

	if !packages_msg.is_empty() {
		packages_update_needed = packages_msg;
	}

	return (num_packages, packages_update_needed);

}

fn packages_updated(package_list: Vec<String>) -> (isize, String) {

	let mut num_packages = 0;

	let mut message: String = "".to_string();

	for package in package_list.iter() {

		let package_array: Vec<&str> = package.split('\n').collect();

		let installed: Vec<&str> = package_array[1].trim().split(' ').collect();
		let candidate: Vec<&str> = package_array[2].trim().split(' ').collect();

		if installed[1] != candidate[1] {
				
			message = format!("{}APT-WARNING: {} new version available.\n", message, package_array[0]);
			num_packages = num_packages + 1;

		}

	}

	return (num_packages, message);
}


fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => { return }
	};

	let update_warning : f64 = match opts.warning.parse() {
		Ok (f64) => { f64 }
		Err (_) => {
			println!("APT-UNKNOWN: Warning level must be a value between 0.0 and 1.0."); 
			process::exit(3);	
		}
	};
	let last_update_str = check_last_update(&opts.rootfs);

	if last_update_str.contains("LAST UPDATE ERROR") {
		println!("APT-UNKNOWN: Could not last update check: {}.", last_update_str); 
		process::exit(3);	
	}

	let last_update : f64 = match last_update_str.parse() {
		Ok (f64) => { f64 }
		Err (_) => {
			println!("APT-UNKNOWN: Error while parsing last update time."); 
			process::exit(3);	
		}
	};


	let mut num_decimals = 0;
	if last_update < 10.0 { num_decimals = 1; }

	let last_update_formated =  format!("{0:.1$}", last_update, num_decimals);

	let reboot_needed = check_reboot(&opts.rootfs);

	if reboot_needed.contains("CHECK REBOOT ERROR") {
		println!("UNKNOWN: Could not execute reboot check: {}.", reboot_needed); 
		process::exit(3);	
	}

	let (num_packages, packages_update) = check_packages(&opts.rootfs);

	if packages_update == "CHECK PACKAGES ERROR".to_string() {
		println!("APT-UNKNOWN: Could not execute packages check."); 
		process::exit(3);	
	}

	let mut print_last_update = true;
	let mut print_reboot = true;
	let mut print_packages = true;

	if last_update >= update_warning {
		print_last_update = false;
	}
	if &packages_update != "KO" {
		print_packages = false;
	}
	if reboot_needed == "YES" {
		print_reboot = false;
	}	

	if print_last_update == true && print_reboot == true && print_packages == true {
		 println!("APT-OK: APT is up to date: Last update: {} hours ago, all packages up to date, reboot not required.", last_update_formated);
		process::exit(0);
	}
	else if print_last_update == false && print_reboot == false && print_packages == false {
		println!("APT-WARNING: Last update {} hours ago, {} packages need update, reboot needed.", last_update_formated, num_packages); 
		process::exit(1);
	}
	else if print_last_update == false && print_reboot == false && print_packages == true {
		println!("APT-WARNING: Last update {} hours ago, reboot needed.", last_update_formated); 
		process::exit(1);
	}
	else if print_last_update == false && print_reboot == true && print_packages == false {
		println!("APT-WARNING: Last update {} hours ago, {} packages need update.", last_update_formated, num_packages); 
		process::exit(1);
	}
	else if print_last_update == true && print_reboot == false && print_packages == false {
		println!("APT-WARNING: {} packages need update, reboot needed.", num_packages); 
		process::exit(1);
	}
	else if print_last_update == true && print_reboot == true && print_packages == false {
		println!("APT-WARNING: {} packages need update.", num_packages); 
		process::exit(1);
	}
	else if print_last_update == true && print_reboot == false && print_packages == true {
		println!("APT-WARNING: Reboot needed."); 
		process::exit(1);
	}
	else if print_last_update == false && print_reboot == true && print_packages == true {
		println!("APT-WARNING: Last update {} hours ago.", last_update_formated); 
		process::exit(1);
	}

	println!("APT-UNKNOWN: Unknown state.");
	process::exit(3);
}


