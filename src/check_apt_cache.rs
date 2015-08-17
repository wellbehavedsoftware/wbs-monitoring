//Rust file
extern crate getopts;

use getopts::Options;
use std::env;
use std::option::{ Option };
use std::process;
use std::fs;

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

	let mut apt_elements_count = 0;
	let mut apt_archive_elements_count = 0;
	let mut apt_archive_partial_elements_count = 0;

	let mut apt_elements_th = 0;
	let mut apt_archive_elements_th = 0;
	let mut apt_archive_partial_elements_th = 0;

	if rootfs.is_empty() {

		apt_elements_th = 3;
		apt_archive_elements_th = 2;
		apt_archive_partial_elements_th = 0;

		let ls_apt_route: String = "/var/cache/apt".to_string();
		let ls_apt_archive_route: String = "/var/cache/apt/archives".to_string();
		let ls_apt_archive_partial_route: String = "/var/cache/apt/archives/partial".to_string();

		// Count the elements present on the apt cache directories
		let ls_apt_elements = match fs::read_dir(&ls_apt_route) {
			Ok(rd) => { rd }
			Err(_) => { return format!("APT-CACHE-UNKNOWN: could not read the directory {}.", ls_apt_route); }
		};

		let ls_apt_archive_elements = match fs::read_dir(&ls_apt_archive_route) {
			Ok(rd) => { rd }
			Err(_) => { return format!("APT-CACHE-UNKNOWN: could not read the directory {}.", ls_apt_archive_route); }
		};

		let ls_apt_archive_partial_elements = match fs::read_dir(&ls_apt_archive_partial_route) {
			Ok(rd) => { rd }
			Err(_) => { return format!("APT-CACHE-UNKNOWN: could not read the directory {}.", ls_apt_archive_partial_route); }
		};

		apt_elements_count = ls_apt_elements.count();
		apt_archive_elements_count = ls_apt_archive_elements.count();
		apt_archive_partial_elements_count = ls_apt_archive_partial_elements.count();

	}
	else { 		
		apt_elements_th = 5;
		apt_archive_elements_th = 4;
		apt_archive_partial_elements_th = 2;

		let apt_route =  format!("/var/lib/lxc/{}/rootfs/var/cache/apt", rootfs);

		let ls_apt_output =
			match process::Command::new ("ls")
				.arg ("-l".to_string ())
				.arg (apt_route.clone())
				.output () {
			Ok (output) => { output }
			Err(_) => { return format!("APT-CACHE-UNKNOWN: could not read the directory {}.", apt_route); }
		};
		let ls_apt = String::from_utf8_lossy(&ls_apt_output.stdout).to_string();

		let ls_apt_archive_output =
			match process::Command::new ("ls")
				.arg ("-l".to_string ())
				.arg (apt_route.clone() + "/archives")
				.output () {
			Ok (output) => { output }
			Err(_) => { return format!("APT-CACHE-UNKNOWN: could not read the directory {}.", apt_route.clone() + "/archives"); }
		};
		let ls_apt_archive = String::from_utf8_lossy(&ls_apt_archive_output.stdout).to_string();

		let ls_apt_archive_partial_output =
			match process::Command::new ("ls")
				.arg ("-l".to_string ())
				.arg (apt_route.clone() + "/archives/partial")
			.	output () {
			Ok (output) => { output }
			Err(_) => { return format!("APT-CACHE-UNKNOWN: could not read the directory {}.", apt_route.clone() + "/archives/partial"); }
		};
		let ls_apt_archive_partial = String::from_utf8_lossy(&ls_apt_archive_partial_output.stdout).to_string();

		let ls_apt_array: Vec<&str> = ls_apt.split("\n").collect();
		let ls_apt_archive_array: Vec<&str> = ls_apt_archive.split("\n").collect();
		let ls_apt_archive_partial_array: Vec<&str> = ls_apt_archive_partial.split("\n").collect();

		apt_elements_count = ls_apt_array.len();
		apt_archive_elements_count = ls_apt_archive_array.len();
		apt_archive_partial_elements_count = ls_apt_archive_partial_array.len();

	}

	if apt_elements_count <= apt_elements_th && apt_archive_elements_count <= apt_archive_elements_th && apt_archive_partial_elements_count <= apt_archive_partial_elements_th {
		return "APT-CACHE-OK: Apt cache is empty.".to_string();
	}
	else {
		return "APT-CACHE-WARNING: Apt cache is not empty. Use apt-get clean to erase it.".to_string();
	}

}


fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => { return }
	};
	
	let apt_cache_msg = check_apt_cache(&opts.rootfs);

	println!("{}", apt_cache_msg);

	if apt_cache_msg.contains("UNKNOWN") {
		process::exit(3);	
	}
	else if apt_cache_msg.contains("OK") {
		process::exit(0);	
	}
	else if apt_cache_msg.contains("WARNING") {
		process::exit(1);	
	}
	else {
		println!("APT-CACHE-UNKNOWN: Could not execute apt cache check. Unknown error."); 
		process::exit(3);	
	}

}
