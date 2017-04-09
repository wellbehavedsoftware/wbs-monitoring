extern crate getopts;
extern crate time;

use std::env;
use std::process;

use getopts::Options;

fn print_usage (program: &str, opts: Options) {
	let brief = format!("Usage: {} [options]", program);
	println!("{}", opts.usage(&brief));
}

fn print_help (program: &str, opts: Options) {
	let brief = format!("Help: {} [options]", program);
	println!("{}", opts.usage(&brief));
}

struct Opts {
	rootfs: String,
	mails: String,
	option: String,
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
			"rootfs",
			"root of the file system in which the checks will be performed",
			"<rootfs>");

	opts.reqopt (
			"",
			"mails",
			"mails in which the checks will be performed, separated by comma",
			"<option>");


	opts.reqopt (
			"",
			"option",
			"which mails are going to be checked: all, seen or unseen",
			"<option>");

	opts.reqopt (
			"",
			"warning",
			"queue time for which the script returns a warning state",
			"<warning>");

	opts.reqopt (
			"",
			"critical",
			"queue time for which the script returns a critical state",
			"<critical>");

	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("check_dovecot", opts);
			process::exit(3);
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_dovecot", opts);
		process::exit(3);
	}

	let rootfs = matches.opt_str ("rootfs").unwrap ();
	let mails = matches.opt_str ("mails").unwrap ();
	let option = matches.opt_str ("option").unwrap ();
	let warning = matches.opt_str ("warning").unwrap ();
	let critical = matches.opt_str ("critical").unwrap ();

	return Some (Opts {
		rootfs: rootfs,
		mails: mails,
		option: option,
		warning: warning,
		critical: critical,
	});

}

fn check_email_list (rootfs: &str, mail: &str, option: &str, warning_th: f64, critical_th: f64) -> (String, i32) {

	let doveadm_output: String;

	if !rootfs.is_empty() {

		//check emails list

		let output =
			match process::Command::new ("sudo")
			.arg ("lxc-attach".to_string ())
			.arg ("--name".to_string ())
			.arg (rootfs.to_string ())
			.arg ("--".to_string ())
			.arg ("doveadm".to_string ())
			.arg ("fetch".to_string ())
			.arg ("-u".to_string ())
			.arg (mail)
			.arg ("date.received".to_string ())
			.arg (option)
			.output () {
		Ok (output) => { output }
		Err (err) => { return (format!("Check email: {}.", err), 0); }
		};

		doveadm_output = String::from_utf8_lossy(&output.stdout).to_string();

	}
	else {

		//check emails list

		let output =
			match process::Command::new ("sudo")
			.arg ("doveadm".to_string ())
			.arg ("fetch".to_string ())
			.arg ("-u".to_string ())
			.arg (mail)
			.arg ("date.received".to_string ())
			.arg (option)
			.output () {
		Ok (output) => { output }
		Err (err) => { return (format!("Check email: {}.", err), 0); }
		};

		doveadm_output = String::from_utf8_lossy(&output.stdout).to_string();

	}

	let mut warning = false;
	let mut critical = false;
	let mut warning_msg = "".to_string();
	let mut critical_msg = "".to_string();
	let mut num_messages = 0;

	let now = time::now ();

	let doveadm_lines: Vec<&str> = doveadm_output.split("\n").collect();

	for line in doveadm_lines {

		let line_tokens: Vec<&str> = line.split(" ").collect();

		if line_tokens.len() <= 1 { continue; }

		num_messages = num_messages + 1;

		let complete_date = format!("{} {}", line_tokens[1], line_tokens[2]);

		let date_object =
			time::strptime (
				& complete_date,
				"%Y-%m-%d %H:%M:%S");

		let date = match date_object {

			Ok (date) => { date }
			Err (e) => {

				return (format!("DOVECOT-ERROR: {}.\n", e), 0);
			}
		};

		let date_diff = now - date;

		let diffseconds = date_diff.num_seconds() as f64;
		let diffhours = (diffseconds / 3600.0) as f64;

		let diffhoursfmt = format!("{0:.1$}", diffhours, 1);

		if diffhours > warning_th && diffhours < critical_th {

			warning = true;
			warning_msg = warning_msg + &format!("DOVECOT-WARNING: Message in {} for more than {} hours.\n", mail, diffhoursfmt);

		}
		else if diffhours > critical_th {

			critical = true;
			critical_msg = critical_msg + &format!("DOVECOT-CRITICAL: Message in {} for more than {} hours.\n", mail, diffhoursfmt);

		}

	}

	if warning || critical {

		return (format!("{}{}", critical_msg, warning_msg), num_messages);

	}
	else {
		return (format!("DOVECOT-OK: {} shared mailbox is OK.\n", mail), num_messages);
	}


}


fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => {
			println!("UNKNOWN: Wrong arguments.");
			process::exit(3);
		}
	};

	let rootfs = &opts.rootfs;
	let option = &opts.option;
	let mails = &opts.mails;

	let warning : f64 = match opts.warning.parse() {
		Ok (f64) => { f64 }
		Err (_) => {
			println!("UNKNOWN: The hours warning threshold must be a double!");
			process::exit(3);
		}
	};
	let critical : f64 = match opts.critical.parse() {
		Ok (f64) => { f64 }
		Err (_) => {
			println!("UNKNOWN: The hours critical threshold must be a double!");
			process::exit(3);
		}
	};

	let mail_list: Vec<&str> = mails.split(",").collect();

	let final_result: String;
	let mut critical_result = "".to_string();
	let mut warning_result = "".to_string();
	let mut ok_result = "".to_string();
	let mut total_messages: i32 = 0;

	for mail in mail_list {

		let (result, messages) = check_email_list (rootfs, mail, option, warning, critical);

		total_messages = total_messages + messages;

		if result.contains("CRITICAL") {

			critical_result = critical_result + &result;

		}
		else if result.contains("WARNING") {

			warning_result = warning_result + &result;

		}
		else if result.contains("OK") {

			ok_result = ok_result + &result;

		}
		else {
			println!("DOVECOT-UNKNOWN: Error when performing the check: {}.\n", result);
			process::exit(3);
		}

	}

	final_result = format!("{}{}{}", critical_result, warning_result, ok_result);
	println!("{} | num_messages={};;;;", final_result, total_messages);

	if final_result.contains("CRITICAL") {

		process::exit(2);

	}
	else if final_result.contains("WARNING") {

		process::exit(1);

	}
	else {

		process::exit(0);
	}

}
