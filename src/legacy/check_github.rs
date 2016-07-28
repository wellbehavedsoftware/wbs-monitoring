//Rust file

extern crate getopts;
extern crate curl;
extern crate serde_json;
extern crate time;

use std::env;
use std::error;
use std::process;

fn print_usage (program: &str, opts: getopts::Options) {
	let brief = format!("Usage: {} [options]", program);
	println!("{}", opts.usage(&brief));
}

fn print_help (program: &str, opts: getopts::Options) {
	let brief = format!("Help: {} [options]", program);
	println!("{}", opts.usage(&brief));
}

struct Opts {
	owner: String,
	repository: String,
	version: String,
	timeout: String,
}

fn parse_options () -> Option<Opts> {

	let args = env::args ();

	let mut opts = getopts::Options::new();

	opts.optflag (
			"",
			"help",
			"print this help menu");

	opts.reqopt (
			"",
			"owner",
			"Owner of the repository that is going to be checked.",
			"<owner>");

	opts.reqopt (
			"",
			"repository",
			"Name of the repository that is going to be checked.",
			"<repository>");

	opts.reqopt (
			"",
			"version",
			"Installed version of the repository that is going to be checked.",
			"<version>");

	opts.reqopt (
			"",
			"timeout",
			"timeout in which the check stops waiting aborts the process",
			"<http-enabled>");

	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("check_github", opts);
			process::exit(3);
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_github", opts);
		process::exit(3);
	}

	let owner = matches.opt_str ("owner").unwrap ();
	let repository = matches.opt_str ("repository").unwrap ();
	let version = matches.opt_str ("version").unwrap ();
	let timeout = matches.opt_str ("timeout").unwrap ();

	return Some (Opts {
		owner: owner,
		repository: repository,
		version: version,
		timeout: timeout,

	});

}

fn check_github (
	owner: & str,
	repository: & str,
	version: & str,
	timeout: f64,
) -> Result <String, Box <error::Error>> {

	let prefix =
		"https://api.github.com/repos".to_string ();

	let url =
		format! (
			"{}/{}/{}/releases/latest",
			prefix,
			owner,
			repository);

	let start =
		time::PreciseTime::now ();

	let mut curl_easy =
		curl::easy::Easy::new ();

	try! (
		curl_easy.get (
			true));

	try! (
		curl_easy.url (
			url.as_str ()));

	let mut curl_headers =
		curl::easy::List::new ();

	try! (
		curl_headers.append (
			"Accept-Language: en"));

	try! (
		curl_headers.append (
			"User-Agent: check-github (wbs-monitoring)"));

	try! (
		curl_easy.http_headers (
			curl_headers));

	let mut response_buffer: Vec <u8> =
		vec! [];

	{

		let mut curl_transfer =
			curl_easy.transfer ();

		try! (
			curl_transfer.write_function (
				|data| {

			response_buffer.extend_from_slice (
				data);

			Ok (data.len ())

		}));

		try! (
			curl_transfer.perform ());

	}

	/*
	let response_code =
		try! (
			curl_easy.response_code ());
	*/

	let response_body =
		try! (
			String::from_utf8 (
				response_buffer));

	let end =
		time::PreciseTime::now ();

	let millis = start.to(end).num_milliseconds() as f64;

	if millis > timeout {

		return Ok (
			format! (
				"GITHUB-CRITICAL: The check timed out at {} milliseconds.",
				millis));

	}

	let data: serde_json::Value =
		serde_json::from_str (
			& response_body,
		).unwrap ();

	let obj = data.as_object().unwrap();
	let release = obj.get("tag_name").unwrap().as_str().unwrap();

	Ok (

		if release != version && (
			release.contains ("rc")
			|| release.contains ("alpha")
			|| release.contains ("beta")
		) {

			format! (
				"GITHUB-WARNING: {} new version available ({}). \
				{} currently installed.",
				repository,
				release,
				version)

		} else if release != version {

			format! (
				"GITHUB-WARNING: {} new release available ({}). \
				{} currently installed.",
				repository,
				release,
				version)

		} else {

			format! (
				"GITHUB-OK: {} version is up to date.",
				repository)

		}

	)

}

fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => {
			println!("GITHUB-UNKNOWN: Wrong arguments.");
			process::exit(3);
		}
	};

	let owner = &opts.owner;
	let repository = &opts.repository;
	let version = &opts.version;

	let timeout : f64 = match opts.timeout.parse() {
		Ok (f64) => { f64 }
		Err (_) => {
			println!("GITHUB-UNKNOWN: Timeout must be a number.");
			process::exit(3);
		}
	};

	let github_res =
		check_github (
			owner,
			repository,
			version,
			timeout,
		).unwrap ();

	println!("{}", github_res);

	if github_res.contains("UNKNOWN") {
		process::exit(3);
	}
	else if github_res.contains("CRITICAL") {
		process::exit(2);
	}
	else if github_res.contains("WARNING") {
		process::exit(1);
	}
	else {
		process::exit(0);
	}

}
