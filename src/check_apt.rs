//Rust file
#![feature(env)]
#![feature(core)]
#![feature(io)]
#![feature(collections)]
#![feature(std_misc)]

extern crate getopts;
extern crate chrono;

use getopts::Options;
use std::env;
use std::option::{ Option };
use std::old_io::{ Command };
use std::f64;
use chrono::{ UTC, Offset };

fn print_usage (program: &str, opts: Options) {
	let brief = format!("Usage: {} [options]", program);
	println!("{}", opts.usage(brief.as_slice()));
}

fn print_help (program: &str, opts: Options) {
	let brief = format!("Help: {} [options]", program);
	println!("{}", opts.usage(brief.as_slice()));
}

struct Opts {
	rootfs: String,
	warning: String,
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

	opts.reqopt (
			"w",
			"warning",
			"package update warning threshold in hours",
			"<update-warning-threshold-hours>");

	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("check_apt", opts);
			return None;
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_apt", opts);
		return None;
	}

	
	let rootfs = matches.opt_str ("rootfs").unwrap ();
	let warning = matches.opt_str ("warning").unwrap ();

	return Some (Opts {
		rootfs: rootfs,
		warning: warning,
	});

}

fn check_last_update (rootfs: &str) -> String {

	let mut update_stamp;

	if rootfs.as_slice().is_empty() {
		let stat_output =
			match Command::new ("stat")
				.arg ("-c".to_string ())
				.arg ("%y".to_string ())
				.arg ("/var/lib/apt/periodic/update-success-stamp".to_string ())
				.output () {
			Ok (output) => { output }
			Err (err) => { return format!("LAST UPDATE ERROR: {}.", err); }
		};
		update_stamp = String::from_utf8_lossy(stat_output.output.as_slice()).to_string();
	}
	else { 		
		let stat_output =
			match Command::new ("sudo")
				.arg ("lxc-attach".to_string ())
				.arg ("--name".to_string ())
				.arg (rootfs.as_slice())
				.arg ("--".to_string ())
				.arg ("stat".to_string ())
				.arg ("-c".to_string ())
				.arg ("%y".to_string ())
				.arg ("/var/lib/apt/periodic/update-success-stamp".to_string ())
				.output () {
			Ok (output) => { output }
			Err (err) => { return format!("LAST UPDATE ERROR: {}.", err); }
		};
		update_stamp = String::from_utf8_lossy(stat_output.output.as_slice()).to_string();		
	}

	let mut day_time: Vec<&str> = update_stamp.as_slice().split('.').collect();
	if day_time.len() == 1 { return "LAST UPDATE ERROR".to_string(); }

	day_time =  day_time[0].as_slice().split(' ').collect();

	let date_array: Vec<&str> = day_time[0].as_slice().split('-').collect();
	let time_array: Vec<&str> = day_time[1].as_slice().split(':').collect();

	
	let last_update_datetime = UTC.ymd(date_array[0].parse().unwrap(), date_array[1].parse().unwrap(), date_array[2].parse().unwrap())
				      .and_hms(time_array[0].parse().unwrap(), time_array[1].parse().unwrap(), time_array[2].parse().unwrap());

	let current_datetime = UTC::now(); 

	let diff = current_datetime - last_update_datetime;

	let diffseconds = diff.num_seconds() as f64;
	let diffhours = diffseconds / 3600.0;

	return diffhours.to_string();
}


fn check_reboot(rootfs: &str) -> String {

	let mut motd;
	let mut reboot_needed = "NO".to_string();
	
	if rootfs.as_slice().is_empty() {
		let cat_output =
			match Command::new ("cat")
				.arg ("/var/run/motd.dynamic".to_string ())
				.output () {
			Ok (output) => { output }
			Err (err) => { return format!("CHECK REBOOT ERROR: {}.", err); }
		};
		motd = String::from_utf8_lossy(cat_output.output.as_slice()).to_string();
	}
	else { 	
		let cat_output =
		match Command::new ("sudo")
			.arg ("lxc-attach".to_string ())
			.arg ("--name".to_string ())
			.arg (rootfs.as_slice())
			.arg ("cat".to_string ())
			.arg ("/var/run/motd.dynamic".to_string ())
			.output () {
		Ok (output) => { output }
		Err (err) => { return format!("CHECK REBOOT ERROR: {}.", err); }
		};
		motd = String::from_utf8_lossy(cat_output.output.as_slice()).to_string();
	}

	if motd.contains("System restart required".as_slice()) {
		reboot_needed = "YES".to_string();
	}

	return reboot_needed;
}

fn check_packages(rootfs: &str) -> (isize, String) {

	let mut packages_update_needed = "KO".to_string();

	let mut package_list = "".to_string();

	let mut dpkg_output;

	if rootfs.as_slice().is_empty() {		

		dpkg_output =
			match Command::new ("dpkg")
				.arg ("--get-selections".to_string ())
				.output () {
			Ok (output) => { output }
			Err (_) => { return (0, "CHECK PACKAGES ERROR".to_string()); }
		};
	}
	else { 		

		dpkg_output =
			match Command::new ("sudo")
				.arg ("lxc-attach".to_string ())
				.arg ("--name".to_string ())
				.arg (rootfs.as_slice())
				.arg ("--".to_string ())
				.arg ("dpkg".to_string ())
				.arg ("--get-selections".to_string ())
				.output () {
			Ok (output) => { output }
			Err (_) => { return (0, "CHECK PACKAGES ERROR".to_string()); }
		};	
	}

	let mut xargs_output =
	match Command::new ("xargs")
		.arg ("apt-cache".to_string())
		.arg ("policy".to_string())
		.spawn () {
	Ok (output) => { output }
	Err (_) => {  return (0, "CHECK PACKAGES ERROR".to_string()); }
	};
	xargs_output.stdin.take().unwrap().write_str(String::from_utf8_lossy(dpkg_output.output.as_slice()).as_slice());
	drop(dpkg_output);

      	let out = match xargs_output.stdout.as_mut().unwrap().read_to_string() {
		 Ok(output) => {
		   output
		 },
		 Err(_) => { return (0, "CHECK PACKAGES ERROR".to_string()); }
	};

	drop(xargs_output);
	
	let output_lines: Vec<&str> = out.as_slice().split_str("\n").collect();
	if output_lines.len() == 1 { return (0, "CHECK PACKAGES ERROR".to_string()); }
	
	let mut i = 0;		
	
	while i < output_lines.len() {

		let line: Vec<&str> = output_lines[i].as_slice().split(':').collect();
		if line[0].as_slice() == "  Installed" {
			package_list = package_list + output_lines[i-1] + "\n" + output_lines[i] + "\n" + output_lines[i+1] + "\n--\n";
		}
		i = i + 1;
	}

	let (num_packages, packages_msg) = packages_updated(package_list.as_slice());

	if !packages_msg.as_slice().is_empty() {
		packages_update_needed = packages_msg;
	}

	return (num_packages, packages_update_needed);

}

fn packages_updated(package_list: &str) -> (isize, String) {

	let mut packages: Vec<&str> = package_list.as_slice().split_str("--\n").collect();
	let mut num_packages = 0;
	let size = packages.len();
	packages.remove(size - 1);

	let mut message: String = "".to_string();

	for package in packages.iter() {

		let package_array: Vec<&str> = package.as_slice().split('\n').collect();

		let installed: Vec<&str> = package_array[1].trim().as_slice().split(' ').collect();
		let candidate: Vec<&str> = package_array[2].trim().as_slice().split(' ').collect();

		if installed[1].as_slice() != "(none)" && installed[1].as_slice() != candidate[1].as_slice() {
				
			if message.as_slice().is_empty() {
				message = message + "WARNING: " + package_array[0] + " new version available.\n";
			}
			else { 
				message = message + "WARNING: " + package_array[0] + " new version available.\n";
			}
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

	let update_warning : f64 = match opts.warning.as_slice().parse() {
		Ok (f64) => { f64 }
		Err (_) => {
			println!("UNKNOWN: Warning level must be a value between 0.0 and 1.0."); 
			env::set_exit_status(3);	
			return;
		}
	};
	
	let last_update_str = check_last_update(opts.rootfs.as_slice());
	if last_update_str.contains("LAST UPDATE ERROR") {
		println!("UNKNOWN: Could not last update check: {}.", last_update_str); 
		env::set_exit_status(3);	
		return;
	}
	let last_update: f64 = last_update_str.parse().unwrap();
	let last_update_formated =  f64::to_str_exact(last_update, 2);

	let reboot_needed = check_reboot(opts.rootfs.as_slice());
	if reboot_needed.contains("CHECK REBOOT ERROR") {
		println!("UNKNOWN: Could not execute reboot check: {}.", reboot_needed); 
		env::set_exit_status(3);	
		return;
	}

	let (num_packages, packages_update) = check_packages(opts.rootfs.as_slice());
	
	if packages_update == "CHECK PACKAGES ERROR".to_string() {
		println!("UNKNOWN: Could not execute packages check."); 
		env::set_exit_status(3);	
		return;
	}

	let mut print_last_update = true;
	let mut print_reboot = true;
	let mut print_packages = true;

	env::set_exit_status(0);

	if last_update >= update_warning {
		print_last_update = false;
		env::set_exit_status(1);
	}
	if packages_update.as_slice() != "KO" {
		print_packages = false;
		env::set_exit_status(1);
	}
	if reboot_needed == "YES" {
		print_reboot = false;
		env::set_exit_status(1);
	}	

	if print_last_update == true && print_reboot == true && print_packages == true {
		 println!("APT-OK: APT is up to date: Last update: {} hours ago, all packages up to date, reboot not required.", last_update_formated);
	}
	else if print_last_update == false && print_reboot == false && print_packages == false {
		println!("APT-WARNING: Last update {} hours ago, {} packages need update, reboot needed.", last_update_formated, num_packages); 
	}
	else if print_last_update == false && print_reboot == false && print_packages == true {
		println!("APT-WARNING: Last update {} hours ago, reboot needed.", last_update_formated); 
	}
	else if print_last_update == false && print_reboot == true && print_packages == false {
		println!("APT-WARNING: Last update {} hours ago, {} packages need update.", last_update_formated, num_packages); 
	}
	else if print_last_update == true && print_reboot == false && print_packages == false {
		println!("APT-WARNING: {} packages need update, reboot needed.", num_packages); 
	}
	else if print_last_update == true && print_reboot == true && print_packages == false {
		println!("APT-WARNING: {} packages need update.", num_packages); 
	}
	else if print_last_update == true && print_reboot == false && print_packages == true {
		println!("APT-WARNING: Reboot needed."); 
	}
	else if print_last_update == false && print_reboot == true && print_packages == true {
		println!("APT-WARNING: Last update {} hours ago.", last_update_formated); 
	}
	
	return;
}


