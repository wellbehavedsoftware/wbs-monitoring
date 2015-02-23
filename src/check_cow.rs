//Rust file
#![feature(env)]
#![feature(core)]
#![feature(io)]
#![feature(path)]

extern crate getopts;

use getopts::Options;
use std::env;
use std::option::{ Option };
use std::old_io::{ Command };
use std::old_io::BufferedReader;
use std::old_io::File;

fn print_usage (program: &str, opts: Options) {
	let brief = format!("Usage: {} [options]", program);
	println!("{}", opts.usage(brief.as_slice()));
}

fn print_help (program: &str, opts: Options) {
	let brief = format!("Help: {} [options]", program);
	println!("{}", opts.usage(brief.as_slice()));
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
			"h",
			"help",
			"print this help menu");

	opts.reqopt (	
			"d",
			"directory",
			"The specified file is a directory",
			"<directory>");
	opts.reqopt (
			"r",
			"rootfs",
			"root of the file system in which the checks will be performed",
			"<rootfs>");

	opts.reqopt (
			"f",
			"file",
			"Route of the file that is going to be check",
			"<file>");


	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("check_cow", opts);
			return None;
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_cow", opts);
		return None;
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
	let mut file_content = BufferedReader::new(File::open(&path));
	let file_lines: Vec<String> = file_content.lines().map(|x| x.unwrap()).collect();

	let mut result: String = "".to_string();

	for line in file_lines.iter() {	

		let chars_to_trim: &[char] = &[' ', '\n'];
	   	let trimmed_line: &str = line.as_slice().trim_matches(chars_to_trim);		

		let mut out: String;

		if directory == "true" {


			let cow_output =
				match Command::new ("sudo")
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

			out = String::from_utf8_lossy(cow_output.output.as_slice()).to_string();

		}
		else {

			let cow_output =
				match Command::new ("sudo")
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

			out = String::from_utf8_lossy(cow_output.output.as_slice()).to_string();

		}

		if out.contains("No such file or directory") || out.is_empty() {
	
			result = result + format!("COW-OK: The file {} does not exist in {}.\n", trimmed_line, rootfs).as_slice();

		}
		else if out.contains("Permission denied") {
			return "COW-UNKNOWN: No enough permissions".to_string();
		}
		else {

			let output_vector: Vec<&str> = out.as_slice().split(' ').collect();
			let file_attr = output_vector[0];

			if file_attr.contains("C") {

				result = result + format!("COW-OK: The file {} exist in {} with COW disabled.\n", trimmed_line, rootfs).as_slice();

			}
			else {

				result = result + format!("COW-WARNING: The file {} exist in {} with COW enabled.\n", trimmed_line, rootfs).as_slice();

			}

		}

	}

	if result.contains("WARNING") {

		result = "COW-WARNING: Some of the next files have COW disabled.\n".to_string() + result.as_slice();

	}

	return result;
	
}

fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => { 
			env::set_exit_status(3);
			println!("UNKNOWN: Wrong arguments.");
			return;
		}
	};


	let rootfs = opts.rootfs.as_slice();
	let file = opts.file.as_slice();
	let directory = opts.directory.as_slice();

	let result = check_cow (rootfs, file, directory);

	if result.contains("UNKNOWN") {

		env::set_exit_status(3);
		println!("{}", result);

	}
	else if result.contains("WARNING") {

		env::set_exit_status(1);
		println!("{}", result);

	} else {

		env::set_exit_status(0);
		println!("{}", result);

	}
	
	return;
}
