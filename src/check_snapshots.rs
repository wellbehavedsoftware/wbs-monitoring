//Rust file
extern crate chrono;
extern crate getopts;

use getopts::Options;
use std::env;
use std::option::{ Option };
use std::process;
use std::fs;
use chrono::*;

fn print_usage (program: &str, opts: Options) {
	let brief = format!("Usage: {} [options]", program);
	println!("{}", opts.usage(&brief));
}

fn print_help (program: &str, opts: Options) {
	let brief = format!("Help: {} [options]", program);
	println!("{}", opts.usage(&brief));
}

struct Opts {
	container: String,
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
			"container-name",
			"name of the container whose snapshots are going to be checked",
			"<container>");

	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("check_snapshots", opts);
			process::exit(3);
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_snapshots", opts);
		process::exit(3);
	}

	let container = matches.opt_str ("container-name").unwrap ();

	return Some (Opts {
		container: container,

	});

}


fn check_snapshots(container: &str) -> String {


	let snapshots_route: String = format!("/var/lib/lxc/{}/snapshots", container);

	// Get the elements in the dir
	let snapshots_elements = match fs::read_dir(&snapshots_route) {
		Ok(rd) => { rd }
		Err(_) => { return format!("SNAPSHOTS-UNKNOWN: could not read the directory {}.", snapshots_route); }
	};

	// Compare the existing snapshots with todays and yesterdays date
	let year = UTC::today().year();
	let month = UTC::today().month();
	let day = UTC::today().day();
	let yester = day - 1;

	let mut day_str: String = format!("{}", day);
	let mut yesterday_str: String = format!("{}", yester);
	let mut month_str: String = format!("{}", month);

	if (month/10) < 1 {
		month_str = format!("0{}", month);
	}
	if (day/10) < 1 {
		day_str = format!("0{}", day);
	}
	if (yester/10) < 1 {
		yesterday_str = format!("0{}", yester);
	}

	let today = format!("{}-{}-{}", year, month_str, day_str);
	let yesterday = format!("{}-{}-{}", year, month_str, yesterday_str);

	for entry in snapshots_elements {

		let dir = match entry {
			Ok(d) => { d }
			Err(_) => { return format!("SNAPSHOTS-UNKNOWN: could not read the directory {}.", snapshots_route); }
		};

		let entry_name = dir.file_name().clone();
		let file_name = entry_name.to_str().unwrap();

		if yesterday.contains(file_name) {
			return format!("SNAPSHOTS-OK: Last snapshots for {} created on {}", container, yesterday);
		}
		if today.contains(file_name) {
			return format!("SNAPSHOTS-OK: Last snapshots for {} created on {}", container, today);
		}

	}

	return format!("SNAPSHOTS-WARNING: No recent snapshots for {} found!", container);

}


fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => { return }
	};

	let snapshots_msg = check_snapshots(&opts.container);

	println!("{}", snapshots_msg);

	if snapshots_msg.contains("UNKNOWN") {
		process::exit(3);
	}
	else if snapshots_msg.contains("OK") {
		process::exit(0);
	}
	else if snapshots_msg.contains("WARNING") {
		process::exit(1);
	}
	else {
		println!("APT-CACHE-UNKNOWN: Could not execute snapshots check. Unknown error.");
		process::exit(3);
	}

}
