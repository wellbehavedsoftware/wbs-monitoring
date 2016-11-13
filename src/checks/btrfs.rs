extern crate getopts;
extern crate libc;

use std::error;

use logic::*;
use lowlevel;
use btrfs;

pub fn new (
) -> Box <PluginProvider> {

	Box::new (
		CheckBtrfsProvider {},
	)

}

struct CheckBtrfsProvider {
}

#[ derive (Clone, Copy, Debug) ]
enum SpaceRatioRaidLevel {
	None,
	Raid1,
	Raid5,
	Raid6,
	Raid10,
}

impl arghelper::EnumArg for SpaceRatioRaidLevel {

	fn from_string (
		string_value: & str,
	) -> Option <SpaceRatioRaidLevel> {

		match string_value {

			"none" => Some (SpaceRatioRaidLevel::None),
			"raid1" => Some (SpaceRatioRaidLevel::Raid1),
			"raid5" => Some (SpaceRatioRaidLevel::Raid5),
			"raid6" => Some (SpaceRatioRaidLevel::Raid6),
			"raid10" => Some (SpaceRatioRaidLevel::Raid10),

			_ => None,

		}

	}

}

struct CheckBtrfsInstance {

	path: String,

	space_ratio_warning: Option <f64>,
	space_ratio_critical: Option <f64>,
	space_ratio_raid_level: Option <SpaceRatioRaidLevel>,

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

		// path

		options_spec.reqopt (
			"",
			"path",
			"path of filesystem to check",
			"PATH");

		// space ratio

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
			"space-ratio-raid-level",
			"free disk space raid level (none, raid1, raid5, raid6, raid10)",
			"LEVEL");

		// balance ratio

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

		// return

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

		let space_ratio_raid_level =
			try! (
				arghelper::parse_enum (
					options_matches,
					"space-ratio-raid-level"));

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
				space_ratio_raid_level: space_ratio_raid_level,

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

		// open directory

		let file_descriptor =
			try! (
				lowlevel::FileDescriptor::open (
					& self.path,
					libc::O_DIRECTORY));

		// perform checks

		try! (
			self.check_space_ratio (
				& mut check_result_builder,
				file_descriptor.get_value ()));

		try! (
			self.check_balance_ratio (
				& mut check_result_builder,
				file_descriptor.get_value ()));

		// return

		Ok (
			check_result_builder.into_check_result (
				plugin_provider,
			)
		)

	}

}

impl CheckBtrfsInstance {

	fn check_space_ratio (
		& self,
		check_result_builder: & mut CheckResultBuilder,
		file_descriptor: libc::c_int,
	) -> Result <(), Box <error::Error>> {

		let filesystem_info =
			try! (
				btrfs::get_filesystem_info (
					file_descriptor));

		let device_infos: Vec <btrfs::DeviceInfo> =
			try! (
				btrfs::get_device_infos (
					file_descriptor,
					& filesystem_info));

		match self.space_ratio_raid_level {

			None | Some (SpaceRatioRaidLevel::None) =>
				try! (
					self.check_space_ratio_no_raid (
						check_result_builder,
						& device_infos)),

			Some (SpaceRatioRaidLevel::Raid1) =>
				try! (
					self.check_space_ratio_raid1 (
						check_result_builder,
						& device_infos)),

			Some (SpaceRatioRaidLevel::Raid5) => {

				check_result_builder.unknown (
					format! (
						"raid5 not yet supported"));

			},

			Some (SpaceRatioRaidLevel::Raid6) => {

				check_result_builder.unknown (
					format! (
						"raid6 not yet supported"));

			},

			Some (SpaceRatioRaidLevel::Raid10) => {

				check_result_builder.unknown (
					format! (
						"raid10 not yet supported"));

			},

		};

		Ok (())

	}

	fn check_space_ratio_no_raid (
		& self,
		check_result_builder: & mut CheckResultBuilder,
		device_infos: & Vec <btrfs::DeviceInfo>,
	) -> Result <(), Box <error::Error>> {

		let total_bytes_used: u64 =
			device_infos.iter ().fold (
				0u64,
				|sum, device_info|
				sum + device_info.bytes_used
			);

		let total_bytes: u64 =
			device_infos.iter ().fold (
				0u64,
				|sum, device_info|
				sum + device_info.total_bytes
			);

		let total_bytes_free =
			total_bytes - total_bytes_used;

		let total_free_ratio: f64 =
			total_bytes_free as f64
			/ total_bytes as f64;

		try! (
			checkhelper::check_ratio_greater_than (
				check_result_builder,
				self.space_ratio_warning,
				self.space_ratio_critical,
				& format! (
					"free space is {}",
					checkhelper::display_data_size_ratio (
						total_bytes_free,
						total_bytes)),
				total_free_ratio));

		Ok (())

	}

	fn check_space_ratio_raid1 (
		& self,
		check_result_builder: & mut CheckResultBuilder,
		device_infos: & Vec <btrfs::DeviceInfo>,
	) -> Result <(), Box <error::Error>> {

		// check enough devices

		if device_infos.len () == 0 {

			check_result_builder.critical (
				format! (
					"raid1 requires at least 2 devices (none present)"));

			return Ok (());

		}

		if device_infos.len () == 1 {

			check_result_builder.critical (
				format! (
					"raid1 requires at least 2 devices (only 1 present)"));

			return Ok (());

		}

		// check total space

		let total_bytes_used: u64 =
			device_infos.iter ().fold (
				0u64,
				|sum, device_info|
				sum + device_info.bytes_used
			);

		let total_bytes: u64 =
			device_infos.iter ().fold (
				0u64,
				|sum, device_info|
				sum + device_info.total_bytes
			);

		let total_bytes_free =
			total_bytes - total_bytes_used;

		// work out effective size for raid1

		let biggest_bytes: u64 =
			device_infos.iter ().map (
				|device_info|
				device_info.total_bytes
			).max ().unwrap ();

		let biggest_bytes_free: u64 =
			device_infos.iter ().map (
				|device_info|
				device_info.total_bytes - device_info.bytes_used
			).max ().unwrap ();

		let effective_bytes =
			if biggest_bytes * 2 < total_bytes {
				total_bytes - biggest_bytes
			} else {
				total_bytes / 2
			};

		let effective_bytes_free =
			if biggest_bytes_free * 2 > total_bytes_free {
				total_bytes_free - biggest_bytes_free
			} else {
				total_bytes_free / 2
			};

		// perform check

		let effective_free_ratio: f64 =
			effective_bytes_free as f64
			/ effective_bytes as f64;

		try! (
			checkhelper::check_ratio_greater_than (
				check_result_builder,
				self.space_ratio_warning,
				self.space_ratio_critical,
				& format! (
					"raid1 free space is {}",
					checkhelper::display_data_size_ratio (
						effective_bytes_free,
						effective_bytes)),
				effective_free_ratio));

		Ok (())

	}

	fn check_balance_ratio (
		& self,
		check_result_builder: & mut CheckResultBuilder,
		file_descriptor: libc::c_int,
	) -> Result <(), Box <error::Error>> {

		let space_infos =
			try! (
				btrfs::get_space_info (
					file_descriptor));

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
