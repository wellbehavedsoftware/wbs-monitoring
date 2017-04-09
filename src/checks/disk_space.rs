extern crate getopts;
extern crate libc;

use std::error;
use std::ffi;
use std::mem;

use logic::*;

check! {

	new = new,
	name = "check-disk-space",
	prefix = "DISK-SPACE",

	provider = CheckDiskSpaceProvider,

	instance = CheckDiskSpaceInstance {

		path: String,

		space_ratio_warning: Option <f64>,
		space_ratio_critical: Option <f64>,

	},

	options_spec = |options_spec| {

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

	},

	options_parse = |options_matches| {

		// path

		let path =
			options_matches.opt_str (
				"path",
			).unwrap ();

		// space ratio

		let space_ratio_warning =
			arghelper::parse_decimal_fraction (
				options_matches,
				"space-ratio-warning",
			) ?;

		let space_ratio_critical =
			arghelper::parse_decimal_fraction (
				options_matches,
				"space-ratio-critical",
			) ?;

		// return

		CheckDiskSpaceInstance {

			path: path,

			space_ratio_warning: space_ratio_warning,
			space_ratio_critical: space_ratio_critical,

		}

	},

	perform = |self, plugin_provider, check_result_builder| {

		let path_c =
			ffi::CString::new (
				self.path.to_owned (),
			) ?;

		let mut filesystem_stats: libc::statfs =
			unsafe {
				mem::zeroed ()
			};

		let statfs_result =
			unsafe {
				libc::statfs (
					path_c.as_ptr (),
					& mut filesystem_stats)
			};

		if statfs_result != 0 {

			check_result_builder.unknown (
				format! (
					"statfs returned {}",
					statfs_result));

		} else {

			self.perform_space_check (
				& mut check_result_builder,
				& filesystem_stats,
			) ?;

		}

	},

}

impl CheckDiskSpaceInstance {

	fn perform_space_check (
		& self,
		check_result_builder: & mut CheckResultBuilder,
		filesystem_stats: & libc::statfs,
	) -> Result <(), Box <error::Error>> {

		let block_size =
			filesystem_stats.f_bsize as u64;

		let total_space =
			filesystem_stats.f_blocks as u64 * block_size;

		let available_space =
			filesystem_stats.f_bavail as u64 * block_size;

		let available_space_ratio =
			available_space as f64 / total_space as f64;

		checkhelper::check_ratio_greater_than (
			check_result_builder,
			self.space_ratio_warning,
			self.space_ratio_critical,
			& format! (
				"free space is {}",
				checkhelper::display_data_size_ratio (
					available_space,
					total_space)),
			available_space_ratio,
		) ?;

		Ok (())

	}

}

// ex: noet ts=4 filetype=rust
