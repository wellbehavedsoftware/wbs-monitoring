//Rust file
extern crate getopts;

use getopts::Options;
use std::env;
use std::process;
use std::option::{ Option };

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
	files: Vec<String>,
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

	opts.optmulti (
			"",
			"files",
			"Route of the files that are going to be checked",
			"<files>");


	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("check_cow", opts);
			process::exit(3);
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_cow", opts);
		process::exit(3);
	}

	let rootfs = matches.opt_str ("rootfs").unwrap ();
	let files = matches.opt_strs ("files");

	return Some (Opts {
		rootfs: rootfs,
		files: files,
	});

}

fn check_cow (rootfs: &str, files: &Vec<String>) -> String {

	let mut result: String = "".to_string();

	for route in files.iter() {	

		let cow_output =
			match process::Command::new ("sudo")
			.arg ("lxc-attach".to_string ())
			.arg ("--name".to_string ())
			.arg (rootfs.to_string ())
			.arg ("--".to_string ())
			.arg ("lsattr".to_string ())
			.arg ("-d".to_string ())
			.arg (route.to_string ())
			.output () {
		Ok (output) => { output }
		Err (err) => { return format!("Check CoW: {}.", err); 
		}
		};

		let out = String::from_utf8_lossy(&cow_output.stdout).to_string();

		if out.contains("No such file or directory") || out.is_empty() {
	
			result = result + &format!("COW-OK: The file {} does not exist in {}.\n", route, rootfs);

		}
		else if out.contains("Permission denied") {
			return "COW-UNKNOWN: No enough permissions".to_string();
		}
		else {

			let output_vector: Vec<&str> = out.split(' ').collect();
			let file_attr = output_vector[0];

			if file_attr.contains("C") {

				result = result + &format!("COW-OK: The file {} exist in {} with COW disabled.\n", route, rootfs);

			}
			else {

				result = result + &format!("COW-WARNING: The file {} exist in {} with COW enabled.\n", route, rootfs);

			}

		}

	}

	if result.contains("WARNING") {

		result = "COW-WARNING: Some of the next files have COW enabled.\n".to_string() + &result;

	}

	return result;
	
}

fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => { 
			println!("UNKNOWN: Wrong arguments.");
			process::exit(3);
		}
	};


	let rootfs = &opts.rootfs;
	let files = opts.files;

	let result = check_cow (rootfs, & files);

	if result.contains("UNKNOWN") {

		println!("{}", result);
		process::exit(3);

	}
	else if result.contains("WARNING") {

		println!("{}", result);
		process::exit(1);

	} else {

		println!("{}", result);
		process::exit(0);

	}
	
}
