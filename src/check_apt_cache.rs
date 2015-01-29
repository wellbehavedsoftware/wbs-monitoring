#![allow(unstable)]
extern crate getopts;

use getopts::{ optflag, reqopt, getopts, short_usage, usage, OptGroup };
use std::os;
use std::option::{ Option };
use std::io::{ Command };

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
		os::set_exit_status(3);	
		return None;
	}

	if ! matches.free.is_empty () {
		print_usage (program.as_slice (), opts);
		os::set_exit_status(3);	
		return None;
	}

	let rootfs = matches.opt_str ("rootfs").unwrap ();


	return Some (Options {
		rootfs: rootfs,

	});

}


fn check_apt_cache(rootfs: &str) -> String {

	let mut ls_apt;
	let mut ls_apt_archive;
	let mut ls_apt_archive_partial;

	if rootfs.as_slice().is_empty() {
		let ls_apt_output =
			match Command::new ("ls")
				.arg ("-l".to_string ())
				.arg ("/var/cache/apt".to_string ())
				.output () {
			Ok (output) => { output }
			Err (err) => { return format!("APT CHACHE ERROR: {}.", err); }
		};
		ls_apt = String::from_utf8_lossy(ls_apt_output.output.as_slice()).to_string();

		let ls_apt_archive_output =
			match Command::new ("ls")
				.arg ("-l".to_string ())
				.arg ("/var/cache/apt/archives".to_string ())
				.output () {
			Ok (output) => { output }
			Err (err) => { return format!("APT CHACHE ERROR: {}.", err); }
		};
		ls_apt_archive = String::from_utf8_lossy(ls_apt_archive_output.output.as_slice()).to_string();

		let ls_apt_archive_partial_output =
			match Command::new ("ls")
				.arg ("-l".to_string ())
				.arg ("/var/cache/apt/archives/partial".to_string ())
				.output () {
			Ok (output) => { output }
			Err (err) => { return format!("APT CHACHE ERROR: {}.", err); }
		};
		ls_apt_archive_partial = String::from_utf8_lossy(ls_apt_archive_partial_output.output.as_slice()).to_string();
	}
	else { 		

		let name: String = rootfs.to_string();
		let mut apt_route = "/var/lib/lxc/".to_string() + name.as_slice();
		apt_route = apt_route + "/rootfs/var/cache/apt";

		let ls_apt_output =
			match Command::new ("ls")
				.arg ("-l".to_string ())
				.arg (apt_route.clone())
				.output () {
			Ok (output) => { output }
			Err (err) => { return format!("APT CHACHE ERROR: {}.", err); }
		};
		ls_apt = String::from_utf8_lossy(ls_apt_output.output.as_slice()).to_string();

		let ls_apt_archive_output =
			match Command::new ("ls")
				.arg ("-l".to_string ())
				.arg (apt_route.clone() + "/archives")
				.output () {
			Ok (output) => { output }
			Err (err) => { return format!("APT CHACHE ERROR: {}.", err); }
		};
		ls_apt_archive = String::from_utf8_lossy(ls_apt_archive_output.output.as_slice()).to_string();

		let ls_apt_archive_partial_output =
			match Command::new ("ls")
				.arg ("-l".to_string ())
				.arg (apt_route.clone() + "/archives/partial")
				.output () {
			Ok (output) => { output }
			Err (err) => { return format!("APT CHACHE ERROR: {}.", err); }
		};
		ls_apt_archive_partial = String::from_utf8_lossy(ls_apt_archive_partial_output.output.as_slice()).to_string();		
	}

	let ls_apt_array: Vec<&str> = ls_apt.as_slice().split_str("\n").collect();
	let ls_apt_archive_array: Vec<&str> = ls_apt_archive.as_slice().split_str("\n").collect();
	let ls_apt_archive_partial_array: Vec<&str> = ls_apt_archive_partial.as_slice().split_str("\n").collect();

	if ls_apt_array.len() <= 5 && ls_apt_archive_array.len() <= 4 && ls_apt_archive_partial_array.len() <= 2 {
		return "OK".to_string();
	}
	else {
		return "WARNING".to_string();
	}

}


fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => { return }
	};
	
	let apt_cache_msg = check_apt_cache(opts.rootfs.as_slice());
	if apt_cache_msg.contains("APT CHACHE ERROR") {
		println!("UNKNOWN: Could not execute apt cache check: {}.", apt_cache_msg); 
		os::set_exit_status(3);	
	}
	else if apt_cache_msg == "OK" {
		println!("OK: Apt cache is empty.");
		os::set_exit_status(0);	
	}
	else if apt_cache_msg == "WARNING" {
		println!("WARNING: Apt cache is not empty. Use apt-get clean to erase it.");
		os::set_exit_status(1);	
	}
	else {
		println!("UNKNOWN: Could not execute apt cache check. Unknown error."); 
		os::set_exit_status(3);	
	}
	
	return;
}
