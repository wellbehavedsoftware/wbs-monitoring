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
	rootfs: String,
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

	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("check_apt_cache", opts);
			process::exit(3);
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_apt_cache", opts);
		process::exit(3);
	}

	let rootfs = matches.opt_str ("rootfs").unwrap ();

	return Some (Opts {
		rootfs: rootfs,

	});

}


fn check_apt_cache(rootfs: &str) -> String {

	let mut ls_apt;
	let mut ls_apt_archive;
	let mut ls_apt_archive_partial;

	if rootfs.is_empty() {
		let ls_apt_output =
			match process::Command::new ("ls")
				.arg ("-l".to_string ())
				.arg ("/var/cache/apt".to_string ())
				.output () {
			Ok (output) => { output }
			Err (err) => { return format!("APT CHACHE ERROR: {}.", err); }
		};
		ls_apt = String::from_utf8_lossy(&ls_apt_output.stdout).to_string();

		let ls_apt_archive_output =
			match process::Command::new ("ls")
				.arg ("-l".to_string ())
				.arg ("/var/cache/apt/archives".to_string ())
				.output () {
			Ok (output) => { output }
			Err (err) => { return format!("APT CHACHE ERROR: {}.", err); }
		};
		ls_apt_archive = String::from_utf8_lossy(&ls_apt_archive_output.stdout).to_string();

		let ls_apt_archive_partial_output =
			match process::Command::new ("ls")
				.arg ("-l".to_string ())
				.arg ("/var/cache/apt/archives/partial".to_string ())
				.output () {
			Ok (output) => { output }
			Err (err) => { return format!("APT CHACHE ERROR: {}.", err); }
		};
		ls_apt_archive_partial = String::from_utf8_lossy(&ls_apt_archive_partial_output.stdout).to_string();
	}
	else { 		

		let name: String = rootfs.to_string();
		let mut apt_route = "/var/lib/lxc/".to_string() + &name;
		apt_route = apt_route + "/rootfs/var/cache/apt";

		let ls_apt_output =
			match process::Command::new ("ls")
				.arg ("-l".to_string ())
				.arg (apt_route.clone())
				.output () {
			Ok (output) => { output }
			Err (err) => { return format!("APT CHACHE ERROR: {}.", err); }
		};
		ls_apt = String::from_utf8_lossy(&ls_apt_output.stdout).to_string();

		let ls_apt_archive_output =
			match process::Command::new ("ls")
				.arg ("-l".to_string ())
				.arg (apt_route.clone() + "/archives")
				.output () {
			Ok (output) => { output }
			Err (err) => { return format!("APT CHACHE ERROR: {}.", err); }
		};
		ls_apt_archive = String::from_utf8_lossy(&ls_apt_archive_output.stdout).to_string();

		let ls_apt_archive_partial_output =
			match process::Command::new ("ls")
				.arg ("-l".to_string ())
				.arg (apt_route.clone() + "/archives/partial")
				.output () {
			Ok (output) => { output }
			Err (err) => { return format!("APT CHACHE ERROR: {}.", err); }
		};
		ls_apt_archive_partial = String::from_utf8_lossy(&ls_apt_archive_partial_output.stdout).to_string();		
	}

	let ls_apt_array: Vec<&str> = ls_apt.split("\n").collect();
	let ls_apt_archive_array: Vec<&str> = ls_apt_archive.split("\n").collect();
	let ls_apt_archive_partial_array: Vec<&str> = ls_apt_archive_partial.split("\n").collect();

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
	
	let apt_cache_msg = check_apt_cache(&opts.rootfs);
	if apt_cache_msg.contains("APT CHACHE ERROR") {
		println!("UNKNOWN: Could not execute apt cache check: {}.", apt_cache_msg); 
		process::exit(3);	
	}
	else if apt_cache_msg == "OK" {
		println!("OK: Apt cache is empty.");
		process::exit(0);	
	}
	else if apt_cache_msg == "WARNING" {
		println!("WARNING: Apt cache is not empty. Use apt-get clean to erase it.");
		process::exit(1);	
	}
	else {
		println!("UNKNOWN: Could not execute apt cache check. Unknown error."); 
		process::exit(3);	
	}

}
