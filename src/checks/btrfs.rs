extern crate getopts;
extern crate libc;

use std::error;

use logic::*;
use lowlevel::btrfs;

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

	balance_ratio_warning: Option <f64>,
	balance_ratio_critical: Option <f64>,

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

		options_spec.optopt (
			"",
			"balance-ratio-warning",
			"block balance warning threshold",
			"RATIO");

		options_spec.optopt (
			"",
			"balance-ratio-critical",
			"block balance critical threshold",
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

		// balance ratio

		let balance_ratio_warning =
			try! (
				arghelper::parse_decimal_fraction (
					options_matches,
					"balance-ratio-warning"));

		let balance_ratio_critical =
			try! (
				arghelper::parse_decimal_fraction (
					options_matches,
					"balance-ratio-critical"));

		// return

		Ok (Box::new (

			CheckBtrfsInstance {

				path: path,

				space_ratio_warning: space_ratio_warning,
				space_ratio_critical: space_ratio_critical,

				balance_ratio_warning: balance_ratio_warning,
				balance_ratio_critical: balance_ratio_critical,

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
			self.check_btrfs_space_info (
				& mut check_result_builder));

		Ok (
			check_result_builder.into_check_result (
				plugin_provider,
			)
		)

	}

}

impl CheckBtrfsInstance {

	fn check_btrfs_space_info (
		& self,
		check_result_builder: & mut CheckResultBuilder,
	) -> Result <(), Box <error::Error>> {

		let space_infos =
			try! (
				btrfs::get_space_info (
					& self.path));

		let data_space_infos =
			space_infos.iter ().filter (
				|space_info|
				space_info.group_type
					== btrfs::GroupType::Data
			);

		let (total_space, used_space) =
			data_space_infos.fold (
				(0, 0),
				|(total, used), space_info|
				(
					total + space_info.total_bytes,
					used + space_info.used_bytes,
				));

		let free_space =
			total_space - used_space;

		let balance_space_ratio: f64 =
			free_space as f64
			/ total_space as f64;

		try! (
			checkhelper::check_ratio_lesser_than (
				check_result_builder,
				self.balance_ratio_warning,
				self.balance_ratio_critical,
				& format! (
					"data block free space is {}",
					checkhelper::display_data_size_ratio (
						free_space,
						total_space)),
				balance_space_ratio));

		Ok (())

	}

}

// ex: noet ts=4 filetype=rust
