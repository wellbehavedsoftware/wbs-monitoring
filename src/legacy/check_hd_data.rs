//Rust file
extern crate getopts;

use getopts::Options;
use std::env;
use std::process;

fn print_usage (program: &str, opts: Options) {
	let brief = format!("Usage: {} [options]", program);
	println!("{}", opts.usage(&brief));
}

fn print_help (program: &str, opts: Options) {
	let brief = format!("Help: {} [options]", program);
	println!("{}", opts.usage(&brief));
}

fn parse_options () -> String {

	let args = env::args ();

	let mut opts = Options::new();

	opts.optflag (
			"",
			"help",
			"print this help menu");

	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("check_hd_data", opts);
			process::exit(3);
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_hd_data", opts);
		process::exit(3);
	}

	return "OK".to_string();

}

fn check_hd_data() -> (i32, String) {

	let smartctl_output =
		match process::Command::new ("sudo")
			.arg ("smartctl".to_string ())
			.arg ("-A".to_string())
			.arg ("/dev/sda".to_string())
			.output () {
		Ok (output) => { output }
		Err (_) => { return (3, "HD DATA ERROR".to_string()); }
	};

	let smartctl_str = String::from_utf8_lossy(&smartctl_output.stdout).trim().to_string();

	if smartctl_str.contains("is currently not installed") {
		println!("Package \"smartmontools\" is not installed.");
		return (3, "HD DATA ERROR".to_string());
	}

	let mut status_array: Vec<&str> = smartctl_str.split("RAW_VALUE\n").collect();
	status_array = status_array[1].split('\n').collect();

	let mut warning = false;
	let mut critical = false;
	let mut message = "OK".to_string();

	for attr in status_array.iter() {
		let attribute = attr.trim();

		let attr_array: Vec<&str> = attribute.split("0x").collect();
		let attr_name_array: Vec<&str> = attr_array[0].split(' ').collect();
		let attr_name = attr_name_array[1];

		let attr_info_array: Vec<&str> = attr_array[1].split(' ').collect();

		let mut i = 1;

		while attr_info_array[i].is_empty() && i < attr_info_array.len() { i = i + 1; }
		let attr_value: isize = attr_info_array[i].parse().unwrap();
		i = i + 1;

		while attr_info_array[i].is_empty() && i < attr_info_array.len() { i = i + 1; }
		let attr_worst: isize = attr_info_array[i].parse().unwrap();
		i = i + 1;

		while attr_info_array[i].is_empty() && i < attr_info_array.len() { i = i + 1; }
		let attr_thresh: isize = attr_info_array[i].parse().unwrap();
		i = i + 1;

		while attr_info_array[i].is_empty() && i < attr_info_array.len() { i = i + 1; }
		let attr_tipo = attr_info_array[i];

		if attr_value == 0 && attr_worst == 0 && attr_thresh == 0 { continue; }

		if attr_value <= attr_thresh {
			if attr_tipo == "Pre-fail" {
				let msg = format!("CRITICAL: Attribute {} is failing. HD approaching to end-of-product life.\n", attr_name);
				message = format!("{}{}", message, msg);
				critical = true;
			}
			else {
				let msg = format!("WARNING: Attribute {} is failing. HD approaching to end-of-product life.\n", attr_name);
				message = format!("{}{}", message, msg);
				warning = true;
			}
		}
		else if attr_worst <= attr_thresh {
			if attr_tipo == "Pre-fail" {
				let msg = format!("CRITICAL: Attribute {} is failing. HD approaching to end-of-product life.\n", attr_name);
				message = format!("{}{}", message, msg);
				critical = true;
			}
			else {
				let msg = format!("WARNING: Attribute {} is failing. HD approaching to end-of-product life.\n", attr_name);
				message = format!("{}{}", message, msg);
				warning = true;
			}
		}
	}

	let mut exit_status = 0;
	if critical { exit_status = 2; }
	else if warning { exit_status = 1; }

	return (exit_status, message);
}


fn main () {

	let options = parse_options ();
	if &options != "OK" {
		process::exit(3);
	}

	let (exit_status, hd_message) = check_hd_data();

	if hd_message == "OK" { println!("OK: HD status is OK."); }
	else { println!("{}", hd_message); }

	process::exit(exit_status);

}
