extern crate getopts;

use std::error;
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::time;

use logic::*;

pub fn new (
) -> Box <PluginProvider> {

	Box::new (
		CheckAptProvider {},
	)

}

struct CheckAptProvider {
}

struct CheckAptInstance {

	root_filesystem: Option <String>,

	update_warning: Option <time::Duration>,
	update_critical: Option <time::Duration>,

	reboot_warning: Option <time::Duration>,
	reboot_critical: Option <time::Duration>,

}

impl PluginProvider
for CheckAptProvider {

	fn name (
		& self,
	) -> & str {
		"check-apt"
	}

	fn prefix (
		& self,
	) -> & str {
		"APT"
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

		options_spec.optopt (
			"",
			"root-filesystem",
			"root file system in which to perform the checks",
			"PATH");

		options_spec.optopt (
			"",
			"update-warning",
			"package update warning threshold in hours",
			"HOURS");

		options_spec.optopt (
			"",
			"update-critical",
			"package update critical threshold in hours",
			"HOURS");

		options_spec.optopt (
			"",
			"reboot-warning",
			"reboot recommendation warning threshold in hours",
			"HOURS");

		options_spec.optopt (
			"",
			"reboot-critical",
			"reboot recommendation critical threshold in hours",
			"HOURS");

		options_spec

	}

	fn new_instance (
		& self,
		_options_spec: & getopts::Options,
		options_matches: & getopts::Matches,
	) -> Result <Box <PluginInstance>, Box <error::Error>> {

		let root_filesystem =
			options_matches.opt_str (
				"root-filesystem",
			);

		let update_warning =
			try! (
				arghelper::parse_duration (
					options_matches,
					"update-warning"));

		let update_critical =
			try! (
				arghelper::parse_duration (
					options_matches,
					"update-critical"));

		let reboot_warning =
			try! (
				arghelper::parse_duration (
					options_matches,
					"reboot-warning"));

		let reboot_critical =
			try! (
				arghelper::parse_duration (
					options_matches,
					"reboot-critical"));

		return Ok (Box::new (

			CheckAptInstance {

				root_filesystem: root_filesystem,

				update_warning: update_warning,
				update_critical: update_critical,

				reboot_warning: reboot_warning,
				reboot_critical: reboot_critical,

			}

		));

	}

}

impl PluginInstance
for CheckAptInstance {

	fn perform_check (
		& self,
		plugin_provider: & PluginProvider,
	) -> Result <CheckResult, Box <error::Error>> {

		let mut check_result_builder =
			CheckResultBuilder::new ();

		self.check_elapsed_hours (
			plugin_provider,
			& mut check_result_builder,
		).unwrap_or_else (
			|error|
			check_result_builder.unknown (
				format! (
					"error checking last update: {}",
					error.description ()))
		);

		self.check_reboot_recommendation (
			plugin_provider,
			& mut check_result_builder,
		).unwrap_or_else (
			|error|
			check_result_builder.unknown (
				format! (
					"error checking reboot recommendation: {}",
					error.description ()))
		);

		self.check_package_upgrades (
			plugin_provider,
			& mut check_result_builder,
		).unwrap_or_else (
			|error|
			check_result_builder.unknown (
				format! (
					"error checking package upgrades: {}",
					error.description ()))
		);

		Ok (
			check_result_builder.into_check_result (
				plugin_provider,
			)
		)

	}

}

impl CheckAptInstance {

	fn check_elapsed_hours (
		& self,
		_plugin_provider: & PluginProvider,
		check_result_builder: & mut CheckResultBuilder,
	) -> Result <(), Box <error::Error>> {

		let update_success_stamp_path =
			format! (
				"{}/var/lib/apt/periodic/update-success-stamp",
				self.root_filesystem.as_ref ().unwrap_or (
					& "".to_string ()));

		match try! (
			file_age_if_exists (
				update_success_stamp_path.as_str ())) {

			Some (elapsed) => {

				let elapsed_seconds =
					elapsed.as_secs ();

				if

					self.update_critical.is_some ()

					&& elapsed_seconds
						> self.update_critical.unwrap ().as_secs ()

				{

					check_result_builder.critical (
						format! (
							"last update {} hours ago (critical is {})",
							elapsed_seconds / 3600,
							self.update_critical.unwrap ().as_secs () / 3600));

				} else if

					self.update_warning.is_some ()

					&& elapsed_seconds
						> self.update_warning.unwrap ().as_secs ()

				{

					check_result_builder.warning (
						format! (
							"last update {} hours ago (warning is {})",
							elapsed_seconds / 3600,
							self.update_warning.unwrap ().as_secs () / 3600));

				} else {

					check_result_builder.ok (
						format! (
							"last update {} hours ago",
							elapsed_seconds / 3600));

				}

			},

			None => {

				if self.update_critical.is_some () {

					check_result_builder.critical (
						format! (
							"no record of successful update"));

				} else if self.update_warning.is_some () {

					check_result_builder.warning (
						format! (
							"no record of successful update"));

				} else {

					check_result_builder.ok (
						format! (
							"no record of successful update"));

				}

			},

		};

		Ok (())

	}

	fn check_reboot_recommendation (
		& self,
		_plugin_provider: & PluginProvider,
		check_result_builder: & mut CheckResultBuilder,
	) -> Result <(), Box <error::Error>> {

		let reboot_required_path =
			format! (
				"{}/var/run/reboot-required",
				self.root_filesystem.as_ref ().unwrap_or (
					& "".to_string ()));

		match try! (
			file_age_if_exists (
				reboot_required_path.as_str ())) {

			Some (elapsed_duration) => {

				let elapsed_seconds =
					elapsed_duration.as_secs ();

				if
					self.reboot_critical.is_some ()

					&& elapsed_seconds
						> self.reboot_critical.unwrap ().as_secs ()

				{

					check_result_builder.critical (
						format! (
							"reboot recommended for {} hours (critical is {})",
							elapsed_seconds / 3600,
							self.reboot_critical.unwrap ().as_secs () / 3600));

				} else if

					self.reboot_warning.is_some ()

					&& elapsed_seconds
						> self.reboot_warning.unwrap ().as_secs ()

				{

					check_result_builder.warning (
						format! (
							"reboot recommended for {} hours (warning is {})",
							elapsed_seconds / 3600,
							self.reboot_warning.unwrap ().as_secs () / 3600));

				} else {

					check_result_builder.ok (
						format! (
							"reboot recommended for {} hours",
							elapsed_seconds / 3600));

				}

			},

			_ => (),

		};

		Ok (())

	}

	fn check_package_upgrades (
		& self,
		_plugin_provider: & PluginProvider,
		check_result_builder: & mut CheckResultBuilder,
	) -> Result <(), Box <error::Error>> {

		let mut summary: AptcUpgradeSummary =
			AptcUpgradeSummary {
				upgrade: 0,
				remove: 0,
				install: 0,
				broken: 0,
				bad: 0,
				reserved05: 0,
				reserved06: 0,
				reserved07: 0,
				reserved08: 0,
				reserved09: 0,
				reserved10: 0,
				reserved11: 0,
				reserved12: 0,
				reserved13: 0,
				reserved14: 0,
				reserved15: 0,
			};

		let success =
			unsafe {

			aptc_upgrade_summary_get (
				& mut summary)

		};

		if success {

			let total =
				summary.upgrade +
				summary.remove +
				summary.install +
				summary.broken +
				summary.bad;

			if total == 0 {

				check_result_builder.ok (
					"no packages need upgrading");

			} else {

				if summary.upgrade > 0 {

					check_result_builder.warning (
						format! (
							"{} packages need upgrading",
							summary.upgrade));

				}

				if summary.remove > 0 {

					check_result_builder.warning (
						format! (
							"{} packages can be removed",
							summary.remove));

				}

				if summary.install > 0 {

					check_result_builder.warning (
						format! (
							"{} packages need installing",
							summary.install));

				}

				if summary.broken > 0 {

					check_result_builder.critical (
						format! (
							"{} packages are broken",
							summary.broken));

				}

				if summary.bad > 0 {

					check_result_builder.critical (
						format! (
							"{} packages failed to install",
							summary.bad));

				}

			}

		} else {

			check_result_builder.unknown (
				"error checking package upgrades");

		}

		// TODO list packages to upgrade
		// TODO show security updates

		Ok (())

	}

}

fn file_age_if_exists (
	file_path: & str,
) -> Result <Option <time::Duration>, Box <error::Error>> {

	let metadata =
		match fs::metadata (
			& file_path) {

		Ok (metadata) =>
			metadata,

		Err (io_error) =>
			match io_error.kind () {

			io::ErrorKind::NotFound =>
				return Ok (
					None),

			_ =>
				return Err (
					Box::new (
						io_error)),

		},

	};

	let timestamp =
		time::UNIX_EPOCH
		+ time::Duration::new (
			metadata.mtime () as u64,
			0);

	let elapsed_duration =
		try! (
			time::SystemTime::now ().duration_since (
				timestamp));

	Ok (Some (
		elapsed_duration))

}

#[ repr (C) ]
struct AptcUpgradeSummary {
	upgrade: u64,
	remove: u64,
	install: u64,
	broken: u64,
	bad: u64,
	reserved05: u64,
	reserved06: u64,
	reserved07: u64,
	reserved08: u64,
	reserved09: u64,
	reserved10: u64,
	reserved11: u64,
	reserved12: u64,
	reserved13: u64,
	reserved14: u64,
	reserved15: u64,
}

#[ link (name = "apt-pkg") ]
#[ link (name = "stdc++") ]
#[ link (name = "aptc", kind = "static") ]
extern "C" {

	fn aptc_upgrade_summary_get (
		summary: * mut AptcUpgradeSummary,
	) -> bool;

}

// ex: noet ts=4 filetype=rust
