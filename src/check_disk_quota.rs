//Rust file
extern crate getopts;

use getopts::Options;
use std::env;
use std::process;

struct CheckResult {
	status: i32,
	message: String,
}

fn print_usage (
	program: & str,
	opts: Options,
) {

	let brief =
		format! (
			"Usage: {} [options]",
			program);

	println! (
		"{}",
		opts.usage (& brief));

}

fn print_help (
	program: & str,
	opts: Options,
) {

	let brief =
		format! (
			"Help: {} [options]",
			program);

	println! (
		"{}",
		opts.usage (& brief));

}

struct Opts {
	warning: String,
	critical: String,
	subvolume: String,
	limit: f64,
}

fn parse_options () -> Opts {

	let args =
		env::args ();

	let mut opts =
		Options::new ();

	opts.optflag (
		"h",
		"help",
		"print this help menu");

	opts.reqopt (
		"",
		"warning",
		"warning usage quota level",
		"WARNING");

	opts.reqopt (
		"",
		"critical",
		"critical usage quota level",
		"CRITICAL");

	opts.reqopt (
		"",
		"subvolume",
		"path to the subvolume to be checked",
		"PATH");

	opts.reqopt (
		"",
		"limit",
		"maximum size of the subvolume",
		"LIMIT");

	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("check_disk_quota", opts);
			process::exit (3);
		}
	};

	if matches.opt_present ("help") {

		print_help (
			"check_disk_quota",
			opts);

		process::exit (3);

	}

	Opts {

		warning:
			matches.opt_str (
				"warning"
			).unwrap (),

		critical:
			matches.opt_str (
				"critical"
			).unwrap (),

		subvolume:
			matches.opt_str (
				"subvolume"
			).unwrap (),

		limit:
			matches.opt_str (
				"limit"
			).unwrap ().parse::<f64> ().unwrap (),

	}

}

fn check_disk (
	path: & str,
	warning_level: f64,
	critical_level: f64,
	disk_limit: f64,
) -> CheckResult {

	let list_output =
		match process::Command::new ("sudo")
			.arg ("/sbin/btrfs")
			.arg ("subvolume")
			.arg ("list")
			.arg (path)
			.output () {

		Ok (ok) => ok,

		Err (err) => return CheckResult {
			status: 3,
			message: format! (
				"DISK-Q UNKNOWN: BTRFS subvolume list failed: {}.",
				err),
		},

	};

	let subvolume =
		String::from_utf8_lossy (
			& list_output.stdout
		).to_string ();

	let to_search_1 =
		format! (
			" @{}",
			path);

	let to_search_2 =
		format! (
			" {}",
			& path [1..]);

	let line =
		match subvolume.split ('\n').find (
			|line|
				line.ends_with (& to_search_1)
				|| line.ends_with (& to_search_2)
		) {

		Some (line) => line,

		None => return CheckResult {
			status: 3,
			message: format! (
				"DISK-Q UNKNOWN: Can't find subvolume."),
		},

	};

	let rootfs_id =
		format! (
			"0/{}",
			line.split (' ').nth (1).unwrap ());

	let qgroup_output =
		match process::Command::new ("sudo")
			.arg ("/sbin/btrfs")
			.arg ("qgroup")
			.arg ("show")
			.arg (path)
			.arg ("-e")
			.arg ("-r")
			.output () {

		Ok (ok) => ok,

		Err (err) => return CheckResult {
			status: 3,
			message: format! (
				"DISK-Q UNKNOWN: BTRFS qgroup show failed: {}.",
				err),
		},

	};

	let qgroup =
		String::from_utf8_lossy (
			& qgroup_output.stdout
		).to_string ();

	let qgroup_line =
		match qgroup.split ('\n').find (
			|line| line.starts_with (& rootfs_id)
		) {

		Some (value) => value,

		None => return CheckResult {
			status: 3,
			message: format! (
				"DISK-Q UNKNOWN: Can't find BTRFS qgroup"),
		},

	};

	let disk_info: Vec <& str> =
		qgroup_line.split_whitespace ().collect ();

	let disk_used =
		match disk_info [1].parse::<f64> () {

		Ok (value) => value,

		Err (err) => return CheckResult {
			status: 3,
			message: format! (
				"DISK-Q UNKNOWN: Error decoding BTRFS qgroup stats: {}",
				err),
		},

	} / 1073741824.0;

	let disk_used_percentage =
		disk_used / disk_limit;

	let num_decimals =
		if disk_limit < 10.0 { 2 }
		else if disk_limit < 100.0 { 1 }
		else { 0 };

	let disk_used_quota =
		format! (
			"{0:.1$}",
			disk_used,
			num_decimals);

	let disk_limit_quota =
		format! (
			"{0:.1$}",
			disk_limit,
			num_decimals);

	let disk_used_percentage_quota =
		format! (
			"{0:.1$}",
			disk_used_percentage * 100.0,
			0);

	let warning_quota_level =
		format! (
			"{0:.1$}",
			warning_level * 100.0,
			0);

	let critical_quota_level =
		format! (
			"{0:.1$}",
			critical_level * 100.0,
			0);

	if disk_limit == 0.0 {

		return CheckResult {
			status: 0,
			message: format! (
				"DISK-Q OK: {} GiB used, no limit.",
				disk_used_quota),
		};

	} else if disk_used_percentage < warning_level {

		return CheckResult {
			status: 0,
			message: format! (
				"DISK-Q OK: {} GiB {}%, \
				limit {} GiB, \
				warning {}%. \
				| disk={}%;50.0;75.0;;",
				disk_used_quota,
				disk_used_percentage_quota,
				disk_limit_quota,
				warning_quota_level,
				disk_used_percentage_quota),
		};

	} else if disk_used_percentage >= warning_level
		&& disk_used_percentage < critical_level {

		return CheckResult {
			status: 1,
			message: format! (
				"DISK-Q WARNING: {} GiB {}%, \
				limit {} GiB, \
				critical {}%. \
				| disk={}%;50.0;75.0;;",
				disk_used_quota,
				disk_used_percentage_quota,
				disk_limit_quota,
				critical_quota_level,
				disk_used_percentage_quota),
		};

	} else {

		return CheckResult {
			status: 2,
			message: format! (
				"DISK-Q CRITICAL: {} GiB {}%, \
				limit {} GiB, \
				critical {}%. \
				| disk={}%;50.0;75.0;;",
				disk_used_quota,
				disk_used_percentage_quota,
				disk_limit_quota,
				critical_quota_level,
				disk_used_percentage_quota),
		};

	}

}

fn main () {

	let opts = parse_options ();

	let disk_warning : f64 =
		match opts.warning.parse () {

		Ok (f64) => { f64 }

		Err (_) => {

			println! (
				"UNKNOWN: Warning level must be a value between 0.0 and 1.0.");

			process::exit (3);

		}

	};

	let disk_critical : f64 =
		match opts.critical.parse () {

		Ok (f64) => { f64 }

		Err (_) => {

			println! (
				"UNKNOWN: Critical level must be a value between 0.0 and 1.0.");

			process::exit (3);

		}

	};

	let result = check_disk (
		& opts.subvolume,
		disk_warning,
		disk_critical,
		opts.limit);

	println! (
		"{}\n",
		result.message);

	process::exit (result.status);

}

