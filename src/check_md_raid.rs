//Rust file

extern crate getopts;

use getopts::Options;
use std::env;
use std::process;
use std::io::BufReader;
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
	host: String,
}

fn parse_options () -> Option<Opts> {

	let args = env::args ();

	let mut opts = Options::new();

	opts.optflag (	
			"h",
			"help",
			"print this help menu");

	opts.reqopt (
			"n",
			"host",
			"host name in which the check is performed",
			"<host-name>");

	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("check_md_raid", opts);
			process::exit(3);
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_md_raid", opts);
		process::exit(3);
	}


	let host = matches.opt_str ("host").unwrap ();

	return Some (Opts {
		host: host,
	});

}

fn check_md_raid(host_name: &str) -> String {

	let md_raid_output =
		match process::Command::new ("cat")
			.arg ("/proc/mdstat".to_string ())
			.output () {
		Ok (output) => { output }
		Err (err) => { return format!("MD RAID ERROR: {}.", err); }
	};	
	let md_raid = String::from_utf8_lossy(&md_raid_output.stdout).to_string();
	let mut md_raid_lines: Vec<&str> = md_raid.split('\n').collect();

	let template_route = "/etc/templates/".to_string() + &host_name + "-mdstat";
	let path = Path::new(&template_route);

	let file = match File::open(&path) {
	    Ok(file) => file,
	    Err(_)  => panic!("I/O Error!"),
	};

	let file = BufReader::new(&file);
	let template_lines: Vec<String> = file.lines().map(|x| x.unwrap()).collect();

	let mut template_content: String = "".to_string();
	for line in template_lines.iter() {
		template_content = template_content + &line;
	}

	let num_lines = md_raid_lines.len();
	md_raid_lines.remove(num_lines - 1);
	
	if md_raid_lines.len() != template_lines.len() {
		return format!("CRITICAL:\nNow:\n{}\nBefore:\n{}\n", md_raid, template_content);
	}
	
	//the first line		
	if md_raid_lines[0].contains("Personalities") && template_lines[0].contains("Personalities") {

		let md_raid_line_array: Vec<&str> = md_raid_lines[0].trim().split(' ').collect();
		let template_line_array: Vec<&str> = template_lines[0].trim().split(' ').collect();

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
		let md_raid_line_array: Vec<&str> = md_raid_line.trim().split(' ').collect();	

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
		let template_line_array: Vec<&str> = template_line.trim().split(' ').collect();

		if md_raid_line != template_line {				
			for raid_token in md_raid_line_array.iter() {			
		
				let mut found = false;
				
				for template_token in template_line_array.iter() {
					if raid_token == template_token { found = true; break; } 					}
				if !found { return format!("CRITICAL:\nNow:\n{}\nBefore:\n{}\n", md_raid, template_content); }
			}
		}

		let ref md_raid_nextline = md_raid_lines[index+1].to_string() + "\n";
		let md_raid_nextline_array: Vec<&str> = md_raid_nextline.trim().split(' ').collect();	

		let ref template_nextline = template_lines[i+1];
		let template_nextline_array: Vec<&str>= template_nextline.trim().split(' ').collect();

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
	
	let md_raid_str = check_md_raid(&opts.host);
	
	if md_raid_str.contains("MD RAID ERROR") {
		println!("UNKNOWN: Could not execute MD raid check: {}.", md_raid_str); 
		process::exit(3);	
	}
	else if md_raid_str.contains("OK") {
		println!("OK: MD raid status is OK.\n{}", md_raid_str); 
		process::exit(0);	
	}
	else if md_raid_str.contains("WARNING") {
		println!("WARNING: MD raid status changed. Some blocks may be missing.\n{}", md_raid_str); 
		process::exit(1);	
	}
	else if md_raid_str.contains("CRITICAL") {
		println!("CRITICAL:  MD raid status changed. A device stopped running or is missing.\n{}", md_raid_str); 
		process::exit(2);	
	}
	else {
		println!("UNKNOWN: Could not execute MD raid check. Unknown error."); 
		process::exit(3);	
	}
	
}

