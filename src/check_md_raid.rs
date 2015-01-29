#![allow(unstable)]
extern crate getopts;

use getopts::{ optflag, reqopt, getopts, short_usage, usage, OptGroup };
use std::os;
use std::io::{ Command };
use std::io::BufferedReader;
use std::io::File;

fn print_usage (program: &str, opts: &[OptGroup]) {
	println! ("{}", short_usage (program, opts));
}

fn print_help (program: &str, opts: &[OptGroup]) {
	println! ("{}", usage (program, opts));
}

struct Options {
	host: String,
}

fn parse_options () -> Option<Options> {

	let args: Vec<String> = os::args ();

	let program = args [0].clone ();

	let opts = &[

		optflag (
			"h",
			"help",
			"print this help menu"),

		reqopt (
			"n",
			"host",
			"host name in which the check is performed",
			"<host-name>"),

	];

	let matches = match getopts (args.tail (), opts) {
		Ok (m) => { m }
		Err (_) => {
			print_usage (program.as_slice (), opts);
			return None;
		}
	};

	if matches.opt_present ("help") {
		print_help (program.as_slice (), opts);
		return None;
	}

	if ! matches.free.is_empty () {
		print_usage (program.as_slice (), opts);
		return None;
	}


	let host = matches.opt_str ("host").unwrap ();

	return Some (Options {
		host: host,
	});

}

fn check_md_raid(host_name: &str) -> String {

	let md_raid_output =
		match Command::new ("cat")
			.arg ("/proc/mdstat".to_string ())
			.output () {
		Ok (output) => { output }
		Err (err) => { return format!("MD RAID ERROR: {}.", err); }
	};	
	let md_raid = String::from_utf8_lossy(md_raid_output.output.as_slice()).to_string();
	let mut md_raid_lines: Vec<&str> = md_raid.as_slice().split('\n').collect();

	let template_route = "/etc/templates/".to_string() + host_name.as_slice() + "-mdstat";
	let path = Path::new(template_route);
	let mut file = BufferedReader::new(File::open(&path));
	let template_lines: Vec<String> = file.lines().map(|x| x.unwrap()).collect();

	let mut template_content: String = "".to_string();
	for line in template_lines.iter() {
		template_content = template_content + line.as_slice();
	}

	let num_lines = md_raid_lines.len();
	md_raid_lines.remove(num_lines - 1);
	
	if md_raid_lines.len() != template_lines.len() {
		return format!("CRITICAL:\nNow:\n{}\nBefore:\n{}\n", md_raid, template_content);
	}
	
	//the first line		
	if md_raid_lines[0].contains("Personalities") && template_lines[0].contains("Personalities") {

		let md_raid_line_array: Vec<&str> = md_raid_lines[0].as_slice().trim().split(' ').collect();
		let template_line_array: Vec<&str> = template_lines[0].as_slice().trim().split(' ').collect();

		for raid_token in md_raid_line_array.iter() {			
			let mut found = false;
			
			for template_token in template_line_array.iter() {
				if raid_token == template_token { found = true; break; } 						}
			if !found { return format!("CRITICAL:\nNow:\n{}\nBefore:\n{}\n", md_raid, template_content); }
		}
	}

	let mut warning = false;
	let mut index = 1;

	//the rest of the lines
	while index < md_raid_lines.len() {

		let ref md_raid_line = md_raid_lines[index].to_string() + "\n";
		let md_raid_line_array: Vec<&str> = md_raid_line.as_slice().trim().split(' ').collect();	

		let mut line_found = false;
		let mut i = 1;

		while i < template_lines.len() {
			if template_lines[i].contains(md_raid_line_array[0]) {
				line_found = true;
				break;
			}
			else { i = i + 3; }
		}	
		if !line_found { return format!("CRITICAL:\nNow:\n{}\nBefore:\n{}\n", md_raid, template_content); }

		let ref template_line = template_lines[i];
		let template_line_array: Vec<&str> = template_line.as_slice().trim().split(' ').collect();

		if md_raid_line != template_line {				
			for raid_token in md_raid_line_array.iter() {			
		
				let mut found = false;
				
				for template_token in template_line_array.iter() {
					if raid_token == template_token { found = true; break; } 					}
				if !found { return format!("CRITICAL:\nNow:\n{}\nBefore:\n{}\n", md_raid, template_content); }
			}
		}

		let ref md_raid_nextline = md_raid_lines[index+1].to_string() + "\n";
		let md_raid_nextline_array: Vec<&str> = md_raid_nextline.as_slice().trim().split(' ').collect();	

		let ref template_nextline = template_lines[i+1].as_slice();
		let template_nextline_array: Vec<&str>= template_nextline.as_slice().trim().split(' ').collect();

		if md_raid_nextline != template_nextline {				
			if md_raid_nextline_array[md_raid_nextline_array.len()-1] != template_nextline_array[template_nextline_array.len()-1] { return format!("CRITICAL:\nNow:\n{}\nBefore:\n{}\n", md_raid, template_content); }
			else { warning = true; }
		}
	
		index = index + 3;
		if index == md_raid_lines.len()-1 { break; }
	}
	
	if warning { return format!("WARNING:\nNow:\n{}\nBefore:\n{}\n", md_raid, template_content); }
	else { return format!("OK:\nNow:\n{}\nBefore:\n{}\n", md_raid, template_content); }
	
}

fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => { return }
	};
	
	let md_raid_str = check_md_raid(opts.host.as_slice());
	
	if md_raid_str.contains("MD RAID ERROR") {
		println!("UNKNOWN: Could not execute MD raid check: {}.", md_raid_str); 
		os::set_exit_status(3);	
	}
	else if md_raid_str.contains("OK") {
		println!("OK: MD raid status is OK.\n{}", md_raid_str); 
		os::set_exit_status(0);	
	}
	else if md_raid_str.contains("WARNING") {
		println!("WARNING: MD raid status changed. Some blocks may be missing.\n{}", md_raid_str); 
		os::set_exit_status(1);	
	}
	else if md_raid_str.contains("CRITICAL") {
		println!("CRITICAL:  MD raid status changed. A device stopped running or is missing.\n{}", md_raid_str); 
		os::set_exit_status(2);	
	}
	else {
		println!("UNKNOWN: Could not execute MD raid check. Unknown error."); 
		os::set_exit_status(3);	
	}
	
	return;
}

