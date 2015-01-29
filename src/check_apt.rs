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
	rootfs: String,
	warning: String,
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

		reqopt (
			"w",
			"warning",
			"package update warning threshold in hours",
			"<update-warning-threshold-hours>"),

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
	let warning = matches.opt_str ("warning").unwrap ();

	return Some (Options {
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
	let update_date = day_time[0];
	let update_time = day_time[1];

	let current_time_output =
		match Command::new ("timedatectl")
			.output () {
		Ok (output) => { output }
		Err (err) => { return format!("LAST UPDATE ERROR: {}.", err); }
	};	

	let current_stamp = String::from_utf8_lossy(current_time_output.output.as_slice()).to_string();

	day_time = current_stamp.as_slice().split_str(": ").collect();
	day_time = day_time[1].as_slice().split(' ').collect();
	let current_date = day_time[1];
	let current_time = day_time[2];
		
	let mut diffhours = datediff(current_date, update_date);
	diffhours = diffhours + timediff(current_time, update_time);

	return diffhours.to_string();
}


fn datediff(date_one: &str, date_two: &str) -> f64 {

	let current_date: Vec<&str>  = date_one.as_slice().split('-').collect();
	let update_date: Vec<&str>  = date_two.as_slice().split('-').collect();
	
	let current_year: f64 = current_date[0].parse().unwrap();
	let update_year: f64 = update_date[0].parse().unwrap();
	let diffyear = current_year - update_year;

	let current_month: f64 = current_date[1].parse().unwrap();
	let update_month: f64 = update_date[1].parse().unwrap();
	let diffmonth = current_month - update_month;

	let current_day: f64 = current_date[2].parse().unwrap();
	let update_day: f64 = update_date[2].parse().unwrap();
	let diffday = current_day - update_day;

	let hours = (diffyear*365.0*24.0) + (diffmonth*30.0*24.0) + (diffday*24.0);

	return hours;
}

fn timediff(time_one: &str, time_two: &str) -> f64 {

	let current_time: Vec<&str> = time_one.as_slice().split(':').collect();
	let update_time: Vec<&str> = time_two.as_slice().split(':').collect();
	
	let current_hour: f64 = current_time[0].parse().unwrap();
	let update_hour: f64 = update_time[0].parse().unwrap();
	let diffhour = current_hour - update_hour;

	let current_min: f64 = current_time[1].parse().unwrap();
	let update_min: f64 = update_time[1].parse().unwrap();
	let diffmin = current_min - update_min;

	let current_seg: f64 = current_time[2].parse().unwrap();
	let update_seg: f64 = update_time[2].parse().unwrap();
	let diffseg = current_seg - update_seg;

	let hours = (diffhour) + (diffmin/60.0) + (diffseg/3600.0);

	return hours;
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
	
	let update_warning = match opts.warning.as_slice().parse() {
		Some (f64) => { f64 }
		None => {
			println!("UNKNOWN: Warning level must be a value between 0.0 and 1.0."); 
			os::set_exit_status(3);	
			return;
		}
	};

	let last_update_str = check_last_update(opts.rootfs.as_slice());
	if last_update_str.contains("LAST UPDATE ERROR") {
		println!("UNKNOWN: Could not last update check: {}.", last_update_str); 
		os::set_exit_status(3);	
		return;
	}
	let last_update: f64 = last_update_str.parse().unwrap();

	let reboot_needed = check_reboot(opts.rootfs.as_slice());
	if reboot_needed.contains("CHECK REBOOT ERROR") {
		println!("UNKNOWN: Could not execute reboot check: {}.", reboot_needed); 
		os::set_exit_status(3);	
		return;
	}

	let (num_packages, packages_update) = check_packages(opts.rootfs.as_slice());
	
	if packages_update == "CHECK PACKAGES ERROR".to_string() {
		println!("UNKNOWN: Could not execute packages check."); 
		os::set_exit_status(3);	
		return;
	}

	let mut last_update_msg = format!("OK: Last update: {} hours ago.", last_update);
	let mut print_last_update = true;

	let mut reboot_msg = format!("OK: System restart not required.");
	let mut print_reboot = true;

	let mut packages_msg = format!("OK: All packages are up to date.");
	let mut print_packages = true;

	os::set_exit_status(0);
	
	if last_update >= update_warning {
		last_update_msg = format!("WARNING: Last update: {} hours ago.", f64::to_str_exact(last_update, 2));
		print_last_update = false;
		println!("{}", last_update_msg);
		os::set_exit_status(1);
	}
	if reboot_needed == "YES" {
		reboot_msg = format!("WARNING: System reboot required.");
		print_reboot = false;
		println!("{}", reboot_msg);
		os::set_exit_status(1);
	}
	if packages_update.as_slice() != "KO" {
		packages_msg = format!("WARNING: {} packages need to be updated.\n\n", num_packages) + packages_update.as_slice();
		print_packages = false;
		println!("{}", packages_msg);
		os::set_exit_status(1);
	}
	
	if print_last_update == true && print_reboot == true && print_packages == true { println!("APT is up to date."); }
	if print_last_update == true { println!("{}", last_update_msg); }
	if print_reboot == true { println!("{}", reboot_msg); }
	if print_packages == true { println!("{}", packages_msg); }
	
	return;
}


