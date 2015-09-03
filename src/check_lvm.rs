//Rust file
extern crate getopts;
extern crate regex;

use getopts::Options;
use std::env;
use std::process;
use std::option::{ Option };
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
	warning: String,
	critical: String,
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
			"warning",
			"lvm group size warning threshold",
			"<warning>");

	opts.reqopt (
			"",
			"critical",
			"lvm group size critical threshold",
			"<critical>");

	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("check_lvm", opts);
			process::exit(3);
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_lvm", opts);
		process::exit(3);
	}

	let warning = matches.opt_str ("warning").unwrap ();
	let critical = matches.opt_str ("critical").unwrap ();

	return Some (Opts {
		warning: warning,
		critical: critical,
	});

}

fn check_lvm (warning_th: f64, critical_th: f64) -> String {

	let mut vgdisplay_output: String;
	
	//check emails list

	let output =
		match process::Command::new ("sudo")
		.arg ("vgdisplay".to_string ())
		.output () {
	Ok (output) => { output }
	Err (err) => { return format!("LVM-UNKNOWN: {}.", err); }
	};

	vgdisplay_output = String::from_utf8_lossy(&output.stdout).to_string();

	let mut perf_data: String = "".to_string();

	let mut size_values: Vec<f64> = vec![];
	let mut size_units: Vec<String> = vec![];

	let mut int_used: f64 = 0.0;
	let mut used_unit: String = "".to_string();
	
	let mut critical_message: String = "".to_string();
	let mut warning_message: String = "".to_string();
	let mut ok_message: String = "".to_string();

	// Get VG size

	let mut expression = format!("VG Size(.+) (\\b(TiB|GiB)\\b)\n");
	let mut re = Regex::new(&expression).unwrap();
	let mut i = 1;

	for cap in re.captures_iter(&vgdisplay_output) {
		let value = cap.at(1).unwrap_or("").trim();

		let int_value = match value.parse() {
			Ok(f64) => { f64 }
			Err(e) => { return format!("LVM-UNKNOWN: Usage {} should be a number!", e); }
		};

		size_values.push(int_value);
		size_units.push(cap.at(2).unwrap_or("").trim().to_string());
		
		i = i + 1;

	}

	// Get VG used

	i = 1;

	expression = format!("Alloc PE / Size(.+) (\\b(TiB|GiB)\\b)\n");
	re = Regex::new(&expression).unwrap();

	for cap in re.captures_iter(&vgdisplay_output) {
		let capt = cap.at(1).unwrap_or("").trim();
		let value_array: Vec<&str> = capt.split(" / ").collect();
		let value = value_array[1];		

		int_used = match value.parse() {
			Ok(f64) => { f64 }
			Err(e) => { return format!("LVM-UNKNOWN: Usage {} should be a number!", e); }
		};

		let used_unit = cap.at(2).unwrap_or("").trim();

		let mut used_percentage: f64 = 0.0;

		if size_units[i-1].contains(used_unit) {
			used_percentage = int_used / size_values[i-1];
		}
		else if (size_units[i-1].contains("TiB") && used_unit.contains("GiB")) ||
			(size_units[i-1].contains("GiB") && used_unit.contains("MiB")) {
			used_percentage = (int_used / 1024.0) / size_values[i-1];
		}
		else if size_units[i-1].contains("TiB") && used_unit.contains("MiB") {
			used_percentage = (int_used / (1024.0 * 1024.0)) / size_values[i-1];
		}

		let used_fmt = format!("{0:.1$}", used_percentage * 100.0, 1);
		let warning_fmt = format!("{0:.1$}", warning_th * 100.0, 1);
		let critical_fmt = format!("{0:.1$}", critical_th * 100.0, 1);

		if used_percentage < warning_th {
			ok_message = format!("{}\nLVM-OK: VG{} - used {}%, warning {}%.", ok_message, i, used_fmt, warning_fmt);
		}
		else if used_percentage >= warning_th && used_percentage < critical_th {
			warning_message = format!("{}\nLVM-WARNING: VG{} - used {}%, critical {}%.", warning_message, i, used_fmt, critical_fmt);
		}
		else {
			critical_message = format!("{}\nLVM-CRITICAL: VG{} - used {}%, critical {}%.", critical_message, i, used_fmt, critical_fmt);
		}

		perf_data = format!("{} VG{}_Used={}%;{};{};;", perf_data, i, used_fmt, warning_fmt, critical_fmt);

		i = i + 1;
	}

	return format!("{}\n{}\n{}\n |{}", critical_message, warning_message, ok_message, perf_data);

}


fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => { 
			println!("LVM-UNKNOWN: Wrong arguments.");
			process::exit(3);
		}
	};

	let warning : f64 = match opts.warning.parse() {
		Ok (f64) => { f64 }
		Err (_) => {
			println!("LVM-UNKNOWN: The size warning threshold must be a double!"); 
			process::exit(3);
		}
	};
	let critical : f64 = match opts.critical.parse() {
		Ok (f64) => { f64 }
		Err (_) => {
			println!("LVM-UNKNOWN: The size critical threshold must be a double!"); 
			process::exit(3);
		}
	};

	let result = check_lvm (warning, critical);
	println!("{}", result);

	if result.contains("CRITICAL") {

		process::exit(2);

	}
	else if result.contains("WARNING") {

		process::exit(1);

	}
	else if result.contains("OK") {

		process::exit(0);
	}
	else {
		println!("LVM-UNKNOWN: Error when performing the check: {}.\n", result);
		process::exit(3);
	}

}

