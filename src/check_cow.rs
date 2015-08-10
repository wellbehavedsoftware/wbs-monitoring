//Rust file
extern crate getopts;

use getopts::Options;
use std::env;
use std::process;
use std::option::{ Option };
use std::io::BufReader as BR;
use std::io::BufRead;
use std::fs::File;
use std::path::Path;

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
	file: String,
	directory: String,
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
			"directory",
			"The specified file is a directory",
			"<directory>");
	opts.reqopt (
			"",
			"rootfs",
			"root of the file system in which the checks will be performed",
			"<rootfs>");

	opts.reqopt (
			"",
			"file",
			"Route of the file that is going to be check",
			"<file>");


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
	let file = matches.opt_str ("file").unwrap ();
	let directory = matches.opt_str ("directory").unwrap ();

	return Some (Opts {
		rootfs: rootfs,
		file: file,
		directory: directory,
	});

}

fn check_cow (rootfs: &str, file: &str, directory: &str) -> String {

	let path = Path::new(file);

	let file = match File::open(&path) {
	    Ok(file) => file,
	    Err(..)  => panic!("I/O Error!"),
	};

	let file_lines: Vec<String> = BR::new(&file).lines().map(|x| x.unwrap()).collect();

	let mut result: String = "".to_string();

	for line in file_lines.iter() {	

		let chars_to_trim: &[char] = &[' ', '\n'];
	   	let trimmed_line: &str = line.trim_matches(chars_to_trim);		

		let mut out: String;

		if directory == "true" {

			let cow_output =
				match process::Command::new ("sudo")
				.arg ("lxc-attach".to_string ())
				.arg ("--name".to_string ())
				.arg (rootfs.to_string ())
				.arg ("--".to_string ())
				.arg ("lsattr".to_string ())
				.arg ("-d".to_string ())
				.arg (trimmed_line.to_string ())
				.output () {
			Ok (output) => { output }
			Err (err) => { return format!("Check CoW: {}.", err); }
			};

			out = String::from_utf8_lossy(&cow_output.stdout).to_string();

		}
		else {

			let cow_output =
				match process::Command::new ("sudo")
				.arg ("lxc-attach".to_string ())
				.arg ("--name".to_string ())
				.arg (rootfs.to_string ())
				.arg ("--".to_string ())
				.arg ("lsattr".to_string ())
				.arg (trimmed_line.to_string ())
				.output () {
			Ok (output) => { output }
			Err (err) => { return format!("Check CoW: {}.", err); }
			};

			out = String::from_utf8_lossy(&cow_output.stdout).to_string();

		}

		if out.contains("No such file or directory") || out.is_empty() {
	
			result = result + &format!("COW-OK: The file {} does not exist in {}.\n", trimmed_line, rootfs);

		}
		else if out.contains("Permission denied") {
			return "COW-UNKNOWN: No enough permissions".to_string();
		}
		else {

			let output_vector: Vec<&str> = out.split(' ').collect();
			let file_attr = output_vector[0];

			if file_attr.contains("C") {

				result = result + &format!("COW-OK: The file {} exist in {} with COW disabled.\n", trimmed_line, rootfs);

			}
			else {

				result = result + &format!("COW-WARNING: The file {} exist in {} with COW enabled.\n", trimmed_line, rootfs);

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
	let file = &opts.file;
	let directory = &opts.directory;

	let result = check_cow (rootfs, file, directory);

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
