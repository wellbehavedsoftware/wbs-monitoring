extern crate getopts;
extern crate libc;

use std::error;
use std::error::Error;
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

	root_filesystem_prefix: String,
	root_filesystem_path: String,

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

		let root_filesystem_prefix =
			options_matches.opt_str (
				"root-filesystem",
			).unwrap_or ("".to_string ());

		let root_filesystem_path =
			options_matches.opt_str (
				"root-filesystem",
			).unwrap_or ("/".to_string ());

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

				root_filesystem_prefix: root_filesystem_prefix,
				root_filesystem_path: root_filesystem_path,

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

		let root_filesystem_exists =
			try! (
				self.check_root_filesystem (
					& mut check_result_builder));

		if root_filesystem_exists {

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

			self.check_apt_cache (
				plugin_provider,
				& mut check_result_builder,
			).unwrap_or_else (
				|error|

				check_result_builder.unknown (
					format! (
						"error checking apt cache: {}",
						error.description ()))

			);

		}

		Ok (
			check_result_builder.into_check_result (
				plugin_provider,
			)
		)

	}

}

impl CheckAptInstance {

	fn check_root_filesystem (
		& self,
		check_result_builder: & mut CheckResultBuilder,
	) -> Result <bool, Box <error::Error>> {

		match fs::metadata (
			& self.root_filesystem_path) {

			Ok (_metadata) =>
				Ok (true),

			Err (io_error) => {

				check_result_builder.unknown (
					format! (
						"unable to see root filesystem: {}: {}",
						self.root_filesystem_path,
						io_error.description ()));

				Ok (false)

			},

		}

	}

	fn check_elapsed_hours (
		& self,
		_plugin_provider: & PluginProvider,
		check_result_builder: & mut CheckResultBuilder,
	) -> Result <(), Box <error::Error>> {

		let update_success_stamp_path =
			format! (
				"{}/var/lib/apt/periodic/update-success-stamp",
				self.root_filesystem_prefix);

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
				self.root_filesystem_prefix);

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

		try! (
			aptc::configuration_set_string (
				"Dir",
				& self.root_filesystem_path));

		try! (
			aptc::configuration_set_string (
				"Dir::State::Status",
				format! (
					"{}/var/lib/dpkg/status",
					self.root_filesystem_path)));

		let summary =
			try! (
				aptc::upgrade_summary_get ());

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
						"{} packages need upgrading (warning)",
						summary.upgrade));

			}

			if summary.remove > 0 {

				check_result_builder.ok (
					format! (
						"{} packages can be removed",
						summary.remove));

			}

			if summary.install > 0 {

				check_result_builder.ok (
					format! (
						"{} packages need installing",
						summary.install));

			}

			if summary.broken > 0 {

				check_result_builder.critical (
					format! (
						"{} packages are broken (critical)",
						summary.broken));

			}

			if summary.bad > 0 {

				check_result_builder.critical (
					format! (
						"{} packages failed to install (critical)",
						summary.bad));

			}

		}

		// TODO list packages to upgrade
		// TODO show security updates

		Ok (())

	}

	fn check_apt_cache (
		& self,
		_plugin_provider: & PluginProvider,
		check_result_builder: & mut CheckResultBuilder,
	) -> Result <(), Box <error::Error>> {

		let mut extra_files: Vec <String> =
			vec! [];

		try! (
			self.check_apt_cache_directory (
				check_result_builder,
				& mut extra_files,
				"/var/cache/apt",
				& vec! [
					"apt-file".to_string (),
					"archives".to_string (),
					"pkgcache.bin".to_string (),
					"srcpkgcache.bin".to_string (),
				]));

		try! (
			self.check_apt_cache_directory (
				check_result_builder,
				& mut extra_files,
				"/var/cache/apt/archives",
				& vec! [
					"lock".to_string (),
					"partial".to_string (),
				]));

		try! (
			self.check_apt_cache_directory (
				check_result_builder,
				& mut extra_files,
				"/var/cache/apt/archives/partial",
				& vec! []));

		if ! extra_files.is_empty () {

			check_result_builder.warning (
				format! (
					"Apt cache contains {} files",
					extra_files.len ()));

			check_result_builder.extra_information (
				"");

			check_result_builder.extra_information (
				"Extra files in APT cache:");

			check_result_builder.extra_information (
				"");

			for extra_file in extra_files {

				check_result_builder.extra_information (
					extra_file);

			}

		}

		Ok (())

	}

	fn check_apt_cache_directory (
		& self,
		check_result_builder: & mut CheckResultBuilder,
		extra_files: & mut Vec <String>,
		directory_name: & str,
		allowed_file_names: & Vec <String>,
	) -> Result <(), Box <error::Error>> {

		let full_directory_name =
			format! (
				"{}{}",
				self.root_filesystem_path,
				directory_name);

		for entry_result in
			try! (
				fs::read_dir (
					full_directory_name)) {

			match entry_result {

				Ok (entry) => {

					let entry_file_name =
						entry.file_name ()
							.to_string_lossy ()
							.into_owned ();

					if ! allowed_file_names.contains (
						& entry_file_name) {

						let entry_path =
							entry.path ()
								.to_string_lossy ()
								.into_owned ();

						let entry_path_from_root =
							& entry_path [
								self.root_filesystem_path.len () .. ];

						extra_files.push (
							entry_path_from_root.to_string ());

					}

				},

				Err (error) => {

					check_result_builder.unknown (
						format! (
							"Error reading apt cache directory: {}: {}",
							directory_name,
							error));

					break;

				},

			};

		}

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

mod aptc {

	use std::error;
	use std::ffi;
	use std::ptr;

	use logic::*;

	use checks::apt::aptc_extern;

	pub use checks::apt::aptc_extern::UpgradeSummary;

	pub fn configuration_set_string <
		NameAsStr: AsRef <str>,
		ValueAsStr: AsRef <str>,
	> (
		name_as_str: NameAsStr,
		value_as_str: ValueAsStr,
	) -> Result <(), Box <error::Error>> {

		let name =
			try! (
				ffi::CString::new (
					name_as_str.as_ref ()));

		let value =
			try! (
				ffi::CString::new (
					value_as_str.as_ref ()));

		unsafe {

			aptc_extern::aptc_configuration_set_string (
				name.as_ptr (),
				value.as_ptr ());

		}

		Ok (())

	}

	pub fn upgrade_summary_get (
	) -> Result <UpgradeSummary, Box <error::Error>> {

		let mut summary =
			UpgradeSummary {
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
				aptc_extern::aptc_upgrade_summary_get (
					& mut summary)
			};

		if success {

			Ok (summary)

		} else {

			let error_c_string =
				unsafe {
					aptc_extern::aptc_error_message ()
				};

			Err (
				Box::new (
					SimpleError::from (

				if error_c_string == ptr::null () {

					"unknown error".to_string ()

				} else {

					let error_c_str =
						unsafe {
							ffi::CStr::from_ptr (
								error_c_string)
						};

					error_c_str.to_string_lossy ().into_owned ()

				}

			)))

		}

	}

}

mod aptc_extern {

	extern crate libc;

	#[ repr (C) ]
	pub struct UpgradeSummary {
		pub upgrade: u64,
		pub remove: u64,
		pub install: u64,
		pub broken: u64,
		pub bad: u64,
		pub reserved05: u64,
		pub reserved06: u64,
		pub reserved07: u64,
		pub reserved08: u64,
		pub reserved09: u64,
		pub reserved10: u64,
		pub reserved11: u64,
		pub reserved12: u64,
		pub reserved13: u64,
		pub reserved14: u64,
		pub reserved15: u64,
	}

	#[ link (name = "apt-pkg") ]
	#[ link (name = "stdc++") ]
	#[ link (name = "aptc", kind = "static") ]
	extern "C" {

		pub fn aptc_configuration_set_string (
			name: * const libc::c_char,
			value: * const libc::c_char,
		);

		pub fn aptc_upgrade_summary_get (
			summary: * mut UpgradeSummary,
		) -> bool;

		pub fn aptc_error_message (
		) -> * const libc::c_char;

	}

}

// ex: noet ts=4 filetype=rust
