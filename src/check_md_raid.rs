//Rust file

extern crate regex;
extern crate getopts;

use getopts::Options;
use std::env;
use std::process;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use regex::Regex;

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
			"",
			"help",
			"print this help menu");

	opts.reqopt (
			"",
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

	// Read current mdstat for this host
	let stat_route = "/proc/mdstat";
	let stat_path = Path::new(&stat_route);

	let mut stat_file = match File::open(&stat_path) {
	    Ok(file) => { file }
	    Err(e)  => { return format!("MEMORY-UNKNOWN: Failed to read /proc/mdstat: {}", e); }
	};

	let mut md_raid: String = "".to_string();
	stat_file.read_to_string(&mut md_raid);

	// Read the mdstat template of this host
	let template_route = "/etc/templates/".to_string() + &host_name + "-mdstat";
	let path = Path::new(&template_route);

	let mut file = match File::open(&path) {
	    Ok(file) => { file }
	    Err(e)  => { return format!("MEMORY-UNKNOWN: Failed to read the template: {}", e); }
	};

	let mut template: String = "".to_string();
	file.read_to_string(&mut template);

	let re = Regex::new(r"^(.+)\n").unwrap();

	// Get headers of current md status and template
	let mut md_raid_header = "";
	for cap in re.captures_iter(&md_raid) {
	    md_raid_header = cap.at(1).unwrap_or("");
	}

	let mut template_header = "";
	for cap in re.captures_iter(&template) {
	    template_header = cap.at(1).unwrap_or("");
	}

	// Compare the headers first
	let template_header_tokens: Vec<&str> = template_header.trim().split(' ').collect();
	let md_raid_header_tokens: Vec<&str> = md_raid_header.trim().split(' ').collect();

	if template_header_tokens.len() != md_raid_header_tokens.len() {
		return format!("MD-RAID-CRITICAL: Header changed!\nCurrent:\n{}\nPrevious:\n{}\n", md_raid_header, template_header);
	}

	for token in template_header_tokens.iter() {			
			
		if !template_header.contains(token) { 
			return format!("MD-RAID-CRITICAL: {} was not present!\nNow:\n{}\nBefore:\n{}\n", token, md_raid_header, template_header); 
		}
	}

	// Get the devices info of current md status and template
	let md_raid_devices_string = re.replace_all(&md_raid, "");
	let template_devices_string = re.replace_all(&template, "");

	// Compare the devices
	let md_raid_devices: Vec<&str> = md_raid_devices_string.split("\n      \n").collect();
	let template_devices: Vec<&str> = template_devices_string.split("\n      \n").collect();

	if template_devices.len() != md_raid_devices.len() {
		return format!("MD-RAID-CRITICAL: The number of devices changed!\nCurrent:\n{}\nPrevious:\n{}\n", md_raid_devices_string, template_devices_string);
	}

	let space_re = Regex::new(r" +").unwrap();

	for md_raid_device in md_raid_devices.iter() {

		let normalized_md_raid_device = space_re.replace_all(&md_raid_device, " ");

		let md_raid_device_tokens: Vec<&str> = normalized_md_raid_device.trim().split(" ").collect();
	
		let mut is_present = false;

		for template_device in template_devices.iter() {

			if template_device.contains(md_raid_device_tokens[0]) {
				is_present = true;

				let normalized_template_device = space_re.replace_all(&template_device, " ");
				let template_device_tokens: Vec<&str> = normalized_template_device.trim().split(" ").collect();
				if md_raid_device_tokens.len() != template_device_tokens.len() {
			
					return format!("MD-RAID-CRITICAL: The device {} has changed!\nCurrent:\n{}\nPrevious:\n{}\n", md_raid_device_tokens[0], md_raid_device, template_device);

				}

				for token in md_raid_device_tokens.iter() {
					if !template_device.contains(token) {
						return format!("MD-RAID-CRITICAL: The device {} has changed!\nCurrent:\n{}\nPrevious:\n{}\n", md_raid_device_tokens[0], md_raid_device, template_device);
					}
				}

			}					

		}
		
		if !is_present {
			return format!("MD-RAID-CRITICAL: The device {} was not present!\n", md_raid_device_tokens[0]);
		}
	}


	return format!("MD-RAID-OK: No changes detected.\nNow:\n{}\nBefore:\n{}\n", md_raid_devices_string, template_devices_string); 
	
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
		println!("{}", md_raid_str); 
		process::exit(0);	
	}
	else if md_raid_str.contains("CRITICAL") {
		println!("{}", md_raid_str); 
		process::exit(2);	
	}
	else {
		println!("UNKNOWN: Could not execute MD raid check. Unknown error."); 
		process::exit(3);	
	}
	
}

