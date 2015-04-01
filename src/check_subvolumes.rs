//Rust file
#![feature(env)]
#![feature(core)]
#![feature(io)]

extern crate getopts;

use getopts::Options;
use std::env;
use std::old_io::{ Command };

fn print_help (program: &str, opts: Options) {
	let brief = format!("Help: {} [options]", program);
	println!("{}", opts.usage(brief.as_slice()));
}

fn parse_options () {

	let args = env::args ();

	let mut opts = Options::new();

	opts.optflag (	
			"h",
			"help",
			"print this help menu");

	let matches = opts.parse (args);

	if matches.unwrap().opt_present ("help") {
		print_help ("check_subvolumes", opts);
	}

}

fn check_subvolumes() -> String {

	let subvolume_list_output =
		match Command::new ("sudo")
			.arg ("btrfs".to_string ())
			.arg ("subvolume".to_string ())
			.arg ("list".to_string ())
			.arg (".".to_string ())
			.output () {
		Ok (output) => { output }
		Err (err) => { return format!("SUBVOLUME ERROR: {}.", err); }
	};
	
	let subvolume_list = String::from_utf8_lossy(subvolume_list_output.output.as_slice()).to_string();
	let subvolume_list_lines: Vec<&str> = subvolume_list.as_slice().split('\n').collect();

	let mut warning_msgs: String = "".to_string();
	let mut ok_msgs: String = "".to_string();

	for subvolume in subvolume_list_lines.iter() {
		let subvolume_tokens: Vec<&str> = subvolume.as_slice().split('/').collect();		
	
		if subvolume_tokens.len() > 5 {

			let mut index = 5;		
			let mut subvolume_path: String = "".to_string();

			while index < subvolume_tokens.len() {

				subvolume_path = subvolume_path + "/" + subvolume_tokens[index];
				index = index + 1;

			}

			warning_msgs = warning_msgs + format!("SUBVOLUME-WARNING: The container {} has an inner subvolume in {}.\n", subvolume_tokens[3], subvolume_path).as_slice();

		}
		else if subvolume_tokens.len() == 5 {
			ok_msgs = ok_msgs + "SUBVOLUME-OK: The container " + subvolume_tokens[3] + " does not have inner subvolumes.\n";
		}
		else if subvolume_tokens.len() == 2 {
			ok_msgs = ok_msgs + "SUBVOLUME-OK: The container " + subvolume_tokens[1] + " does not have inner subvolumes.\n";
		}
	}

	let message: String = warning_msgs + ok_msgs.as_slice();

	return message;
	
}

fn main () {

	parse_options ();
	
	let result = check_subvolumes();
	
	if result.contains("SUBVOLUME ERROR") {
		println!("SUBVOLUME-UNKNOWN: Subvolumes check failed: {}.", result); 
		env::set_exit_status(3);	
	}
	else if result.contains("WARNING") {
		println!("SUBVOLUME-WARNING: {}", result); 
		env::set_exit_status(1);	
	}
	else if result.contains("OK") {
		println!("SUBVOLUME-OK: Containers don't have inner subvolumes.\n{}", result); 
		env::set_exit_status(0);	
	}
	else {
		println!("SUBVOLUME-UNKNOWN: Could not execute subvolumes check. Error.\n{}", result); 
		env::set_exit_status(3);	
	}
	
	return;
}

