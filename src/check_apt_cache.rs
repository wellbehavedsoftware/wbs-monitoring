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

	let mut ls_apt_route: String = "/var/cache/apt".to_string();
	let mut ls_apt_archive_route: String = "/var/cache/apt/archives".to_string();
	let mut ls_apt_archive_partial_route: String = "/var/cache/apt/archives/partial".to_string();

	if !rootfs.is_empty() {

		ls_apt_route = format!("/var/lib/lxc/{}{}", rootfs, ls_apt_route);
		ls_apt_archive_route = format!("/var/lib/lxc/{}{}", rootfs, ls_apt_archive_route);
		ls_apt_archive_partial_route = format!("/var/lib/lxc/{}{}", rootfs, ls_apt_archive_partial_route);

	}

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

	if ls_apt_elements.count() <= 3 && ls_apt_archive_elements.count() <= 2 && ls_apt_archive_partial_elements.count() <= 0 {
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
