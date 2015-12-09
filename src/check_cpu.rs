//Rust file

extern crate regex;
extern crate getopts;

use getopts::Options;
use std::env;
use std::option::{ Option };
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
			"warning cpu usage level",
			"<warning-level>");

	opts.reqopt (
			"",
			"critical",
			"critical cpu usage level",
			"<critical-level>");

	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("check_cpu", opts);
			process::exit(3);
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_cpu", opts);
		process::exit(3);
	}

	let warning = matches.opt_str ("warning").unwrap ();
	let critical = matches.opt_str ("critical").unwrap ();

	return Some (Opts {
		warning: warning,
		critical: critical,
	});

}


fn check_cpu(warning_level: f64, critical_level: f64) -> String {

	let stat_route = "/proc/stat";
	let path = Path::new(&stat_route);

	let mut file = match File::open(&path) {
	    Ok(file) => { file }
	    Err(e)  => { return format!("MEMORY-UNKNOWN: Failed to read /proc/stat: {}", e); }
	};

	let mut stat: String = "".to_string();
	file.read_to_string(&mut stat);

	let stat_lines: Vec<&str> = stat.split('\n').collect();

	// Taking multiple spaces away from cpu line
	let space_re = Regex::new(r" +").unwrap();
	let normalized_cpu_line = space_re.replace_all(&stat_lines[0], " ");

	let stat_cpu: Vec<&str> = normalized_cpu_line.split(' ').collect();

	// Building the perfdata string
	let user_str = stat_cpu[1];
	let niced = stat_cpu[2];
	let system = stat_cpu[3];
	let idle_str = stat_cpu[4];
	let iowait = stat_cpu[5];
	let irq = stat_cpu[6];
	let softirq = stat_cpu[7];

	let mut perf_data = format!("user={}c;;;; niced={}c;;;; system={}c;;;; idle={}c;;;; iowait={}c;;;; irq={};;;; softirq={}c;;;;", user_str, niced, system, idle_str, iowait, irq, softirq);

	let interesting_fields = [ "ctxt", "btime", "processes", "procs_running", "procs_blocked" ];

	let mut field_values: Vec<f64> = vec![];

	for field in interesting_fields.iter() {

		let expression = format!("{} (.+)\n", field);
		let re = Regex::new(&expression).unwrap();

		for cap in re.captures_iter(&stat) {
			let value = cap.at(1).unwrap_or("").trim();
			let int_value: f64 = match value.parse() {
				Ok(f64) => { f64 }
				Err(e) => { return format!("CPU-UNKNOWN: {} should be a number!", e); }
			};
			field_values.push(int_value);

			if !field.contains("procs_running") && !field.contains("procs_blocked") {
				perf_data = format!("{} {}={};;;;", perf_data, field, int_value);
				break;
			}
			else {
				perf_data = format!("{} {}={}c;;;;", perf_data, field, int_value);
				break;
			}

		}

	}

	// Computing the cpu usage
	let user : f64 = match user_str.parse() {
		Ok (f64) => { f64 }
		Err (_) => { return "CPU-UNKNOWN: Unable to parse /proc/stat.".to_string(); }
	};

	let kernel : f64 = match system.parse() {
		Ok (f64) => { f64 }
		Err (_) => { return "CPU-UNKNOWN: Unable to parse /proc/stat.".to_string(); }
	};

	let busy = user + kernel;

	let idle : f64 = match idle_str.parse() {
		Ok (f64) => { f64 }
		Err (_) => { return "CPU-UNKNOWN: Unable to parse /proc/stat.".to_string(); }
	};

	let cpu = busy / (busy + idle);
	let cpu_used = format!("{0:.1$}", cpu*100.0, 1);

	let warning_level_fmt = format!("{0:.1$}", warning_level * 100.0, 1);
	let critical_level_fmt = format!("{0:.1$}", critical_level * 100.0, 1);

	if cpu < warning_level {
		return format!("CPU-OK: used {}%, warning {}%. | cpu={}%;{};{};; {}", cpu_used, warning_level_fmt, cpu_used, warning_level_fmt, critical_level_fmt, perf_data);
	}
	else if cpu >= warning_level && cpu < critical_level {
		return format!("CPU-WARNING: used {}%, critical {}%. | cpu={}%;{};{};; {}", cpu_used, critical_level_fmt, cpu_used, warning_level_fmt, critical_level_fmt, perf_data);
	}
	else {
		return format!("CPU-CRITICAL: used {}%, critical {}%. | cpu={}%;{};{};; {}", cpu_used, critical_level_fmt, cpu_used, warning_level_fmt, critical_level_fmt, perf_data);
	}

}

fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => { return }
	};

	let cpu_warning : f64 = match opts.warning.parse() {
		Ok (f64) => { f64 }
		Err (_) => {
			println!("CPU-UNKNOWN: Warning level must be a value between 0.0 and 1.0.");
			process::exit(3);
		}
	};

	let cpu_critical : f64 = match opts.critical.parse() {
		Ok (f64) => { f64 }
		Err (_) => {
			println!("CPU-UNKNOWN: Critical level must be a value between 0.0 and 1.0.");
			process::exit(3);
		}
	};

	let cpu_str = check_cpu(cpu_warning, cpu_critical);
	println!("{}", cpu_str);

	if cpu_str.contains("UNKNOWN") {
		process::exit(3);
	}
	else if cpu_str.contains("CRITICAL") {
		process::exit(2);
	}
	else if cpu_str.contains("WARNING") {
		process::exit(1);
	}
	else if cpu_str.contains("OK") {
		process::exit(0);
	}
	else {
		process::exit(3);
	}
}

