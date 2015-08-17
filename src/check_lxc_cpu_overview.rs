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

fn parse_options () {

	let args = env::args ();

	let mut opts = Options::new();

	opts.optflag (	
			"",
			"help",
			"print this help menu");

	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("check_lxc_cpu_overview", opts);
			process::exit(3);
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_lxc_cpu_overview", opts);
		process::exit(3);
	}

}

fn check_cpu_overview() -> String {

	let mut perfdata = "|".to_string();

	// Get the containers list
	let ls_output =
		match process::Command::new ("sudo")
			.arg ("/usr/bin/lxc-ls".to_string ())
			.arg ("-1".to_string())
			.arg ("--running".to_string())
			.output () {
		Ok (output) => { output }
		Err (_) => { return format!("CPU-OVERVIEW-UNKNOWN: Unable to get containers list."); }
	};

	let ls_output_str = String::from_utf8_lossy(&ls_output.stdout).trim().to_string();
	let container_list: Vec<&str> = ls_output_str.split("\n").collect();

	// Get the cpuacct data for each container
	for container in container_list.iter() {

		let usage_output =
			match process::Command::new ("sudo")
				.arg ("/usr/bin/lxc-cgroup".to_string ())
				.arg ("--name".to_string())
				.arg (container.to_string())
				.arg ("cpuacct.usage".to_string())
				.output () {
			Ok (output) => { output }
			Err (_) => { return format!("CPU-OVERVIEW-UNKNOWN: Unable to get {} cpu usage.", container); }
		};

		let usage_str: String = String::from_utf8_lossy(&usage_output.stdout).trim().to_string();

		let usage_float: f64 = match usage_str.parse() {
			Ok(f64) => { f64 }
			Err(_) => { continue; }
		};

		perfdata = format!("{} {}_usage={}us;;;;", perfdata, container, usage_float / 1000.0);

		let stat_output =
			match process::Command::new ("sudo")
				.arg ("/usr/bin/lxc-cgroup".to_string ())
				.arg ("--name".to_string())
				.arg (container.to_string())
				.arg ("cpuacct.stat".to_string())
				.output () {
			Ok (output) => { output }
			Err (_) => { return format!("CPU-OVERVIEW-UNKNOWN: Unable to get {} cpu stats.", container); }
		};

		let stat_str = String::from_utf8_lossy(&stat_output.stdout).trim().to_string();
		let stat_list: Vec<&str> = stat_str.split("\n").collect();

		let stat_user_list: Vec<&str> = stat_list[0].split(' ').collect();
		perfdata = format!("{} {}_user={};;;;", perfdata, container, stat_user_list[1]);

		let stat_system_list: Vec<&str> = stat_list[1].split(' ').collect();
		perfdata = format!("{} {}_system={};;;;", perfdata, container, stat_system_list[1]);
	
	}

	return format!("CPU-OVERVIEW-OK: Click to display containers CPU usage. {}", perfdata);
	
}


fn main () {

	parse_options ();
	
	let cpu_str = check_cpu_overview();
	println!("{}", cpu_str);

	if cpu_str.contains("UNKNOWN") {
		process::exit(3);	
	}
	else if cpu_str.contains("OK") {
		process::exit(0);	
	}
	else {
		println!("CPU-OVERVIEW-UNKNOWN: Could not execute cpu overview check. Unknown error."); 
		process::exit(3);	
	}

}

