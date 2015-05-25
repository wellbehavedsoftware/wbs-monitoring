//Rust file
extern crate getopts;

use getopts::Options;
use std::env;
use std::option::{ Option };
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
	root: String,
	warning: String,
	critical: String,
}

fn parse_options () -> Option<Opts> {

	let args = env::args ();

	let mut opts = Options::new();

	opts.optflag (	
			"h",
			"help",
			"print this help menu");

	opts.reqopt (
			"r",
			"root",
			"root of the filesystem to check",
			"<fs-root>");

	opts.reqopt (
			"w",
			"warning",
			"warning memory usage threshold",
			"<warning-threshold>");

	opts.reqopt (
			"c",
			"critical",
			"critical memory usage threshold",
			"<critical-threshold>");

	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("check_disk", opts);
			return None;
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_disk", opts);
		return None;
	}

	let root = matches.opt_str ("root").unwrap ();
	let warning = matches.opt_str ("warning").unwrap ();
	let critical = matches.opt_str ("critical").unwrap ();

	return Some (Opts {
		root: root,
		warning: warning,
		critical: critical,
	});

}

fn disk_state () -> String {

	let list_output =
		match process::Command::new ("df")
			.output () {
		Ok (output) => { output }
		Err (_) => { return "DISK ERROR".to_string(); }
	};

	String::from_utf8_lossy(&list_output.stdout).to_string()
}


fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => { return }
	};

	let state = disk_state ();

	if state == "DISK ERROR".to_string() {
		println!("DISK UNKNOWN: Could not execute memory check command."); 
		process::exit(3);	
	}

	let to_check: String = opts.root;

	let warning_level : f64 = match opts.warning.parse() {
		Ok (f64) => { f64 }
		Err (_) => { 
			println!("UNKNOWN: Warning level must be a value between 0.0 and 1.0."); 
			process::exit(3);	
		}
	};

	let critical_level : f64 = match opts.critical.parse() {
		Ok (f64) => { f64 }
		Err (_) => { 
			println!("UNKNOWN: Critical level must be a value between 0.0 and 1.0."); 
			process::exit(3);	
		}
	};

	let state_vector: Vec<&str> = state.split('\n').collect();

	let mut interest_line: &str = "";
	let mut found = false;

	for line in state_vector.iter() { 

		let str_line: String = line.to_string() + "\n";
		let to_check_aux = format!("{}\n", &to_check);
				
		if str_line.contains(&to_check_aux) { 
			interest_line = &line;
			found = true;
			break;
		}	

	}

	if !found { 
		println!("DISK UNKNOWN: The {} volume does not exist.", to_check); 
		process::exit(3);	
	}

	let line_vector: Vec<&str> = interest_line.split(' ').collect();	
	let percentage_vector: Vec<&str> = line_vector[line_vector.len()-2].split('%').collect();

	let disk_quota_percentage = percentage_vector[0];
	let mut disk_used_percentage : f64 = match disk_quota_percentage.parse() {
		Ok (f64) => { f64 }
		Err (_) => { 
			println!("UNKNOWN: The used disk limit is incorrect."); 
			process::exit(3);	
		}
	};
	disk_used_percentage = disk_used_percentage / 100.0;

	let mut index = 1;
	while line_vector[index].is_empty() { index = index + 1; }

	let mut disk_limit : f64 = match line_vector[index].parse() {
		Ok (f64) => { f64 }
		Err (_) => { 
			println!("UNKNOWN: The disk limit is incorrect."); 
			process::exit(3);	
		}
	};


	disk_limit = disk_limit / 1048576.0;
	let disk_used = disk_used_percentage * disk_limit;

	let mut num_decimals = 0;
	if disk_limit < 10.0 { num_decimals = 2; }
	else if disk_limit < 100.0 { num_decimals = 1; }

	let disk_quota_limit = format!("{0:.1$}", disk_limit, num_decimals);

	
	let disk_quota_used = format!("{0:.1$}", disk_used, num_decimals);

	let warning_quota_level = format!("{0:.1$}", warning_level * 100.0, 0);
	let critical_quota_level = format!("{0:.1$}", critical_level * 100.0, 0);

	if disk_used_percentage < warning_level {
		println!("DISK OK: {} GiB {}%, limit {} GiB, warning {}%.", disk_quota_used, disk_quota_percentage, disk_quota_limit, warning_quota_level);
		process::exit(0);
	}
	else if disk_used_percentage >= warning_level && disk_used_percentage < critical_level {
		println!("DISK WARNING: {} GiB {}%, limit {} GiB, critical {}%.", disk_quota_used, disk_quota_percentage, disk_quota_limit, critical_quota_level);
		process::exit(1);
	}
	else {
		println!("DISK CRITICAL: {} GiB {}%, limit {} GiB, critical {}%.", disk_quota_used, disk_quota_percentage, disk_quota_limit, critical_quota_level);
		process::exit(2);
	}

}
