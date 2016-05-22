//Rust file

extern crate regex;
extern crate getopts;

use getopts::Options;
use std::env;
use std::process;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::vec::Vec;
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

fn parse_options () -> Option<Opts>  {

	let args = env::args ();

	let mut opts = Options::new();

	opts.optflag (
			"",
			"help",
			"print this help menu");

	opts.reqopt (
			"",
			"warning",
			"warning usage level",
			"<warning-level>");

	opts.reqopt (
			"",
			"critical",
			"critical usage level",
			"<critical-level>");

	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("check_memory", opts);
			process::exit(3);
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_memory", opts);
		process::exit(3);
	}

	let warning = matches.opt_str ("warning").unwrap ();
	let critical = matches.opt_str ("critical").unwrap ();

	return Some (Opts {
		warning: warning,
		critical: critical,
	});

}

fn check_memory(warning_level: f64, critical_level: f64) -> String {

	let meminfo_route = "/proc/meminfo";
	let path = Path::new(&meminfo_route);

	let mut file = match File::open(&path) {
	    Ok(file) => { file }
	    Err(e)  => { return format!("MEMORY-UNKNOWN: Failed to read /proc/meminfo: {}", e); }
	};

	let mut meminfo: String = "".to_string();
	file.read_to_string(&mut meminfo);

	// Fields we want to check
	let interesting_fields = [ "MemTotal", "MemFree", "Buffers", "Cached", "SwapCached", "Active", "Inactive", "SwapTotal", "SwapFree", "Slab", "SReclaimable", "SUnreclaim" ];

	let mut field_values: Vec<f64> = vec![];

	let mut perf_data: String = "".to_string();

	// For each field, we store its value and build its perfdata string
	for field in interesting_fields.iter() {

		let expression = format!("{}:(.+) kB\n", field);
		let re = Regex::new(&expression).unwrap();

		for cap in re.captures_iter(&meminfo) {
			let value = cap.at(1).unwrap_or("").trim();
			let int_value: f64 = match value.parse() {
				Ok(f64) => { f64 }
				Err(e) => { return format!("MEMORY-UNKNOWN: Usage {} should be a number!", e); }
			};
			field_values.push(int_value);

			perf_data = format!("{} {}={}KB;;;;", perf_data, field, int_value);
			break;

		}

	}

	// Compute memory and swap usage
	let total_memory = field_values[0];
	let free_memory = field_values[1];
	let inactive = field_values[6];

	let total_swap = field_values[7];
	let free_swap = field_values[8];

	let memory_usage = (total_memory - (free_memory + inactive)) / total_memory;

	let mut swap_usage = 0.0;
	if total_swap != 0.0 {
		swap_usage = (total_swap - free_swap) / total_swap;
	}

	let memory_usage_fmt = format!("{0:.1$}", memory_usage * 100.0, 1);
	let swap_usage_fmt = format!("{0:.1$}", swap_usage * 100.0, 1);
	let warning_level_fmt = format!("{0:.1$}", warning_level * 100.0, 1);
	let critical_level_fmt = format!("{0:.1$}", critical_level * 100.0, 1);

	// Memory state
	let mut memory_message: String;
	if memory_usage < warning_level {
		memory_message = format!("MEMORY-OK: used {}%, warning {}%.", memory_usage_fmt, warning_level_fmt);
	}
	else if memory_usage >= warning_level && memory_usage < critical_level {
		memory_message = format!("MEMORY-WARNING: used {}%, critical {}%.", memory_usage_fmt, critical_level_fmt);
	}
	else {
		memory_message = format!("MEMORY-CRITICAL: used {}%, critical {}%.", memory_usage_fmt, critical_level_fmt);
	}

	// Swap state
	let mut swap_message: String;
	if swap_usage < warning_level {
		swap_message = format!("SWAP-OK: used {}%, warning {}%.", swap_usage_fmt, warning_level_fmt);
	}
	else if swap_usage >= warning_level && swap_usage < critical_level {
		swap_message = format!("SWAP-WARNING: used {}%, critical {}%.", swap_usage_fmt, critical_level_fmt);
	}
	else {
		swap_message = format!("SWAP-CRITICAL: used {}%, critical {}%.", swap_usage_fmt, critical_level_fmt);
	}

	let mut message: String;

	if (swap_message.contains("CRITICAL") && (memory_message.contains("WARNING") || memory_message.contains("OK"))) || (swap_message.contains("WARNING") && memory_message.contains("OK")) {
		message = format!("{}\n{}", swap_message, memory_message);
	}
	else {
		message = format!("{}\n{}", memory_message, swap_message);
	}

	return format!("{} | Memory={}%;{};{};; Swap={}%;{};{};;{}", message, memory_usage_fmt, warning_level_fmt, critical_level_fmt, swap_usage_fmt, warning_level_fmt, critical_level_fmt, perf_data);
}


fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => { return }
	};

	let mem_warning : f64 = match opts.warning.parse() {
		Ok (f64) => { f64 }
		Err (_) => {
			println!("UNKNOWN: Warning level must be a value between 0.0 and 1.0.");
			process::exit(3);
		}
	};

	let mem_critical : f64 = match opts.critical.parse() {
		Ok (f64) => { f64 }
		Err (_) => {
			println!("UNKNOWN: Critical level must be a value between 0.0 and 1.0.");
			process::exit(3);
		}
	};

	let mem_str = check_memory(mem_warning, mem_critical);
	println!("{}", mem_str);

	if mem_str.contains("UNKNOWN") {
		process::exit(3);
	}
	else if mem_str.contains("CRITICAL") {
		process::exit(2);
	}
	else if mem_str.contains("WARNING") {
		process::exit(1);
	}
	else if mem_str.contains("OK") {
		process::exit(0);
	}
	else {
		println!("MEMORY-UNKNOWN: Could not execute memory check. Unknown error.");
		process::exit(3);
	}
}


