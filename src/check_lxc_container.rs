extern crate getopts;

use std::env;
use std::error;
use std::fs;
use std::process;

fn usage_error (
	program: & str,
	options_spec: & getopts::Options,
) {

	let brief =
		format! (
			"Usage: {} [options]",
			program);

	println! (
		"{}",
		options_spec.usage (
			& brief));

	process::exit (3)

}

fn print_help (
	program: & str,
	options_spec: & getopts::Options,
) {

	let brief =
		format! (
			"Help: {} [options]",
			program);

	println! (
		"{}",
		options_spec.usage (
			& brief));

}

#[ derive (PartialEq, PartialOrd) ]
enum ContainerState {
	Present,
	NotPresent
}

fn container_state_from_str (
	string: & str,
) -> ContainerState {

	match string {
		"present" => ContainerState::Present,
		"not-present" => ContainerState::NotPresent,
		_ => panic! (),
	}

}

struct Arguments {
	container_name: String,
	critical_states: Vec <ContainerState>,
}

fn build_options_spec (
) -> getopts::Options {

	let mut options_spec =
		getopts::Options::new ();

	options_spec.optflag (
		"",
		"help",
		"print this help menu");

	options_spec.reqopt (
		"",
		"container-name",
		"name of the container to check",
		"<container-name>");

	options_spec.optmulti (
		"",
		"critical-state",
		"container state which cause a critical status",
		"<state>");

	options_spec

}

fn parse_options (
	options_spec: & getopts::Options,
) -> Result <Arguments, Box <error::Error>> {

	let arguments =
		env::args ();

	let matches =
		try! (
			options_spec.parse (
				arguments));

	if matches.opt_present ("help") {

		print_help (
			"check_lxc_memory",
			& options_spec);

		process::exit (3);

	}

	let container_name =
		matches.opt_str (
			"container-name",
		).unwrap ();

	let mut critical_states: Vec <ContainerState> =
		vec! [];

	for container_state_string in matches.opt_strs (
		"critical-state",
	) {

		critical_states.push (
			container_state_from_str (
				container_state_string.as_str ()));

	}

	return Ok (Arguments {
		container_name: container_name,
		critical_states: critical_states,
	});

}

fn check_not_present (
	arguments: & Arguments,
) {

	if arguments.critical_states.contains (
		& ContainerState::NotPresent,
	) {

		println! (
			"LXC CRITICAL: Container {} not present",
			arguments.container_name);

		process::exit (2);
		
	}

	println! (
		"LXC OK: Container {} not present",
		arguments.container_name);

	process::exit (0);

}

fn check_present (
	arguments: & Arguments,
) {

	if arguments.critical_states.contains (
		& ContainerState::Present,
	) {

		println! (
			"LXC CRITICAL: Container {} present",
			arguments.container_name);

		process::exit (2);
		
	}

	println! (
		"LXC OK: Container {} present",
		arguments.container_name);

	process::exit (0);

}

fn run_check (
	arguments: & Arguments,
) {

	let container_path =
		format! (
			"/var/lib/lxc/{}",
			arguments.container_name);

	let metadata =
		match fs::metadata (
			container_path,
		) {

		Err (_) => {

			return check_not_present (
				arguments)

		},

		Ok (metadata) => {

			return check_present (
				arguments)

		},

	};

}

fn main () {

	let options_spec =
		build_options_spec ();

	match parse_options (
		& options_spec,
	) {

		Err (_) => {

			usage_error (
				"check_lxc_memory",
				& options_spec)

		}

		Ok (arguments) => {

			run_check (
				& arguments)

		}

	}

}

