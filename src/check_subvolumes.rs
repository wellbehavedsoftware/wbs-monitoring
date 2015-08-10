//Rust file
extern crate getopts;

use getopts::Options;
use std::option::{ Option };
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
			print_usage ("check_subvolumes", opts);
			process::exit(3);
		}
	};


	if matches.opt_present ("help") {
		print_help ("check_subvolumes", opts);
		process::exit(3);
	}

	let rootfs = matches.opt_str ("rootfs").unwrap ();

	return Some (Opts {
		rootfs: rootfs,
	});

}

fn check_subvolumes(rootfs: &str) -> String {

	let subvolume_list_output =
		match process::Command::new ("sudo")
			.arg ("btrfs".to_string ())
			.arg ("subvolume".to_string ())
			.arg ("list".to_string ())
			.arg (".".to_string ())
			.output () {
		Ok (output) => { output }
		Err (err) => { return format!("SUBVOLUME ERROR: {}.", err); }
	};
	
	let subvolume_list = String::from_utf8_lossy(&subvolume_list_output.stdout).to_string();
	let subvolume_list_lines: Vec<&str> = subvolume_list.split('\n').collect();

	let mut warning_msgs: String = "".to_string();
	let mut ok_msgs: String = "".to_string();

	for subvolume in subvolume_list_lines.iter() {

		if !subvolume.contains("rootfs") || 
		   !subvolume.contains(rootfs) {
			continue;
		}

		let subvolume_tokens: Vec<&str> = subvolume.split('/').collect();

		let mut index = 0;		

		while index < subvolume_tokens.len() {

			if subvolume_tokens[index] == "rootfs" {

				if index == subvolume_tokens.len() - 1 {

					ok_msgs = ok_msgs + &format!("SUBVOLUME-OK: The container {} does not have inner subvolumes.\n", rootfs);

				}
				else {

					let mut subvolume_path: String = "".to_string();

					while index < subvolume_tokens.len() {
					
						subvolume_path = subvolume_path + "/" + subvolume_tokens[index];
						index = index + 1;

					}

					warning_msgs = warning_msgs + &format!("SUBVOLUME-WARNING: The container {} has an inner subvolume in {}.\n", rootfs, subvolume_path);
				}

			}
			index = index + 1;

		}

	}

	let message: String;

	if warning_msgs.is_empty() && ok_msgs.is_empty() {
		message = format!("SUBVOLUME-OK: The container {} does not have inner subvolumes.\n", rootfs);
	}
	else {
		message = warning_msgs + &ok_msgs;
	}

	return message;
	
}

fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => { process::exit(0); }
	};

	let result = check_subvolumes(&opts.rootfs);
	
	if result.contains("SUBVOLUME ERROR") {
		println!("SUBVOLUME-UNKNOWN: Subvolumes check failed: {}.", result); 
		process::exit(3);	
	}
	else if result.contains("WARNING") {
		println!("{}", result); 
		process::exit(1);	
	}
	else if result.contains("OK") {
		println!("SUBVOLUME-OK: {} doesn't have inner subvolumes.\n{}", opts.rootfs, result); 
		process::exit(0);	
	}
	else {
		println!("SUBVOLUME-UNKNOWN: Could not execute subvolumes check. Error.\n{}", result); 
		process::exit(3);	
	}
	
}

