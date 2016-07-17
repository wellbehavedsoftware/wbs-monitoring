extern crate getopts;
extern crate libc;

use std::error;
use std::ffi;
use std::mem;

use logic::*;
use lowlevel::*;

pub fn new (
) -> Box <PluginProvider> {

	Box::new (
		CheckBtrfsProvider {},
	)

}

struct CheckBtrfsProvider {
}

struct CheckBtrfsInstance {

	path: String,

	space_ratio_warning: Option <f64>,
	space_ratio_critical: Option <f64>,

}

impl PluginProvider
for CheckBtrfsProvider {

	fn name (
		& self,
	) -> & str {
		"check-btrfs"
	}

	fn prefix (
		& self,
	) -> & str {
		"BTRFS"
	}

	fn build_options_spec (
		& self,
	) -> getopts::Options {

		let mut options_spec =
			getopts::Options::new ();

		options_spec.optflag (
			"",
			"help",
			"print this help menu");

		options_spec.reqopt (
			"",
			"path",
			"path of filesystem to check",
			"PATH");

		options_spec.optopt (
			"",
			"space-ratio-warning",
			"free disk space warning threshold",
			"RATIO");

		options_spec.optopt (
			"",
			"space-ratio-critical",
			"free disk space critical threshold",
			"RATIO");

		options_spec

	}

	fn new_instance (
		& self,
		_options_spec: & getopts::Options,
		options_matches: & getopts::Matches,
	) -> Result <Box <PluginInstance>, Box <error::Error>> {

		// path

		let path =
			options_matches.opt_str (
				"path",
			).unwrap ();

		// space ratio

		let space_ratio_warning =
			try! (
				arghelper::parse_decimal_fraction (
					options_matches,
					"space-ratio-warning"));

		let space_ratio_critical =
			try! (
				arghelper::parse_decimal_fraction (
					options_matches,
					"space-ratio-critical"));

		// return

		Ok (Box::new (

			CheckBtrfsInstance {

				path: path,

				space_ratio_warning: space_ratio_warning,
				space_ratio_critical: space_ratio_critical,

			}

		))

	}

}

impl PluginInstance
for CheckBtrfsInstance {

	fn perform_check (
		& self,
		plugin_provider: & PluginProvider,
	) -> Result <CheckResult, Box <error::Error>> {

		let mut check_result_builder =
			CheckResultBuilder::new ();

		try! (
			self.check_btrfs_df (
				& mut check_result_builder));

		Ok (
			check_result_builder.into_check_result (
				plugin_provider,
			)
		)

	}

}

impl CheckBtrfsInstance {

	fn check_btrfs_df (
		& self,
		check_result_builder: & mut CheckResultBuilder,
	) -> Result <(), Box <error::Error>> {

		let space_infos: Vec <btrfs::BtrfsSpaceInfo> =
			try! (
				btrfs::get_space_info (
					& self.path));

		Ok (())

	}

}

// ex: noet ts=4 filetype=rust
