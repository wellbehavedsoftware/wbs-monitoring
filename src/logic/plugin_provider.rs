use getopts;

use std::error;

use logic::check_result::*;

pub trait PluginProvider {

	fn name (
		& self,
	) -> & str;

	fn prefix (
		& self,
	) -> & str;

	fn build_options_spec (
		& self,
	) -> getopts::Options;

	fn new_instance (
		& self,
		options_spec: & getopts::Options,
		options_matches: & getopts::Matches,
	) -> Result <Box <PluginInstance>, Box <error::Error>>;

}

pub trait PluginInstance {

	fn perform_check (
		& self,
		plugin_provider: & PluginProvider,
	) -> Result <CheckResult, Box <error::Error>>;

}

// ex: noet ts=4 filetype=rust
