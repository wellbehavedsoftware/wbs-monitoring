use getopts;

use std::env;
use std::process;

use logic::checkresult::*;
use logic::pluginprovider::*;

fn show_help (
	plugin_provider: & PluginProvider,
	options_spec: & getopts::Options,
) {

	let brief =
		format! (
			"Usage: {} [options]",
			plugin_provider.name ());

	println! (
		"{}",
		options_spec.usage (
			& brief));

}

fn show_usage_error (
	plugin_provider: & PluginProvider,
	_options_spec: & getopts::Options,
	error: & getopts::Fail,
) {

	println! (
		"");

	match error {

		& getopts::Fail::OptionMissing (
			ref argument_name,
		) => {

			println! (
				"Missing required argument: --{}",
				argument_name.as_str ());

		},

		& _ =>

			println! (
				"{:?}",
				error),

	}

	println! (
		"");

	println! (
		"For detailed usage information, run:");

	println! (
		"  {} --help",
		plugin_provider.name ());

	println! (
		"");

}

pub fn run_generically (
	plugin_provider: & PluginProvider,
	plugin_instance: & Box <PluginInstance>,
) -> CheckResult {

	match plugin_instance.perform_check (
		plugin_provider,
	) {

		Ok (check_result) =>
			check_result,

		Err (error) =>
			CheckResult::new (
				CheckStatus::Unknown,
				plugin_provider.prefix ().to_string (),
				vec! [
					"check did not run correctly due to program \
						error".to_string (),
					error.description ().to_string (),
				],
				vec! [],
				vec! [],
			)

	}

}

pub fn run_from_options_matches (
	plugin_provider: & PluginProvider,
	options_spec: & getopts::Options,
	options_matches: & getopts::Matches,
) -> CheckResult {

	let plugin_instance =
		plugin_provider.new_instance (
			& options_spec,
			& options_matches,
		);

	match plugin_instance {

		Ok (plugin_instance) =>
			run_generically (
				plugin_provider,
				& plugin_instance),

		Err (error) =>
			CheckResult::new (
				CheckStatus::Unknown,
				plugin_provider.prefix ().to_string (),
				vec! [
					"unable to process command line arguments due to program \
						error".to_string (),
					error.description ().to_string (),
				],
				vec! [],
				vec! [],
			)

	}

}

pub fn run_from_command_line (
	plugin_provider: & PluginProvider,
) {

	let options_spec =
		plugin_provider.build_options_spec ();

	let environment_arguments: Vec <String> =
		env::args ().into_iter ().collect ();

	// handle --help specially

	if environment_arguments.contains (
		& "--help".to_string (),
	) {

		show_help (
			plugin_provider,
			& options_spec);

		process::exit (0);

	}

	// parse options

	let options_matches =
		match options_spec.parse (
			environment_arguments,
		) {

		Ok (options_matches) =>
			options_matches,

		Err (error) => {

			show_usage_error (
				plugin_provider,
				& options_spec,
				& error);

			process::exit (1);

		},

	};

	// delegate

	let check_result =
		run_from_options_matches (
			plugin_provider,
			& options_spec,
			& options_matches,
		);

	// display result

	println! (
		"{} {}: {}",
		plugin_provider.prefix (),
		check_result.status ().prefix (),
		check_result.status_message (),
	);

	for extra_line in check_result.extra_information () {

		println! (
			"{}",
			extra_line);

	}

	process::exit (
		* check_result.status () as i32);

}

// ex: noet ts=4 filetype=rust
