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

	update_warning_seconds: Option <u64>,
	update_critical_seconds: Option <u64>,

	reboot_warning_seconds: Option <u64>,
	reboot_critical_seconds: Option <u64>,

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

		let update_warning_seconds =
			try! (
				option_hours_string_to_seconds (
					& options_matches.opt_str (
						"update-warning")));

		let update_critical_seconds =
			try! (
				option_hours_string_to_seconds (
					& options_matches.opt_str (
						"update-critical")));

		let reboot_warning_seconds =
			try! (
				option_hours_string_to_seconds (
					& options_matches.opt_str (
						"reboot-warning")));

		let reboot_critical_seconds =
			try! (
				option_hours_string_to_seconds (
					& options_matches.opt_str (
						"reboot-critical")));

		return Ok (Box::new (

			CheckAptInstance {

				root_filesystem: root_filesystem,

				update_warning_seconds: update_warning_seconds,
				update_critical_seconds: update_critical_seconds,

				reboot_warning_seconds: reboot_warning_seconds,
				reboot_critical_seconds: reboot_critical_seconds,

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

		try! (
			self.check_elapsed_hours (
				plugin_provider,
				& mut check_result_builder));

		try! (
			self.check_reboot_recommendation (
				plugin_provider,
				& mut check_result_builder));

		try! (
			self.check_package_upgrades (
				plugin_provider,
				& mut check_result_builder));

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
			file_age_if_exists_in_seconds (
				update_success_stamp_path.as_str ())) {

			Some (elapsed_seconds) => {

				let elapsed_hours =
					elapsed_seconds / 3600;

				if
					self.update_critical_seconds.is_some ()

					&& elapsed_seconds
						> * self.update_critical_seconds.as_ref ().unwrap ()

				{

					check_result_builder.critical (
						format! (
							"last update {} hours ago (critical is {})",
							elapsed_hours,
							self.update_critical_seconds.as_ref ().unwrap ()
								/ 3600));

				} else if

					self.update_warning_seconds.is_some ()

					&& elapsed_seconds
						> * self.update_warning_seconds.as_ref ().unwrap ()

				{

					check_result_builder.warning (
						format! (
							"last update {} hours ago (warning is {})",
							elapsed_hours,
							self.update_warning_seconds.as_ref ().unwrap ()
								/ 3600));

				} else {

					check_result_builder.ok (
						format! (
							"last update {} hours ago",
							elapsed_hours));

				}

			},

			None => {

				if self.update_critical_seconds.is_some () {

					check_result_builder.critical (
						format! (
							"no record of successful update"));

				} else if self.update_warning_seconds.is_some () {

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
			file_age_if_exists_in_seconds (
				reboot_required_path.as_str ())) {

			Some (elapsed_seconds) => {

				let elapsed_hours =
					elapsed_seconds / 3600;

				if
					self.reboot_critical_seconds.is_some ()

					&& elapsed_seconds
						> * self.reboot_critical_seconds.as_ref ().unwrap ()

				{

					check_result_builder.critical (
						format! (
							"reboot recommended for {} hours (critical is {})",
							elapsed_hours,
							self.reboot_critical_seconds.as_ref ().unwrap ()
								/ 3600));

				} else if

					self.reboot_warning_seconds.is_some ()

					&& elapsed_seconds
						> * self.reboot_warning_seconds.as_ref ().unwrap ()

				{

					check_result_builder.warning (
						format! (
							"reboot recommended for {} hours (warning is {})",
							elapsed_hours,
							self.reboot_warning_seconds.as_ref ().unwrap ()
								/ 3600));

				} else {

					check_result_builder.ok (
						format! (
							"reboot recommended for {} hours",
							elapsed_hours));

				}

			},

			None => {

				check_result_builder.ok (
					format! (
						"no recommended reboot"));

			},

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

		Ok (())

	}

	/*
	fn check_packages(rootfs: &str) -> (isize, String) {

		let mut packages_update_needed = "KO".to_string();

		let mut dpkg_output;

		if rootfs.is_empty() {

			dpkg_output =
				match process::Command::new ("dpkg")
					.arg ("--get-selections".to_string ())
					.output () {
				Ok (output) => { output }
				Err (_) => { return (0, "CHECK PACKAGES ERROR".to_string()); }
			};
		}
		else {

			dpkg_output =
				match process::Command::new ("sudo")
					.arg ("lxc-attach".to_string ())
					.arg ("--name".to_string ())
					.arg (&rootfs)
					.arg ("--".to_string ())
					.arg ("dpkg".to_string ())
					.arg ("--get-selections".to_string ())
					.output () {
				Ok (output) => { output }
				Err (_) => { return (0, "CHECK PACKAGES ERROR".to_string()); }
			};
		}

		let mut xargs_output =
			match process::Command::new ("xargs")
				.arg ("apt-cache".to_string())
				.arg ("policy".to_string())
			        	.stdin(std::process::Stdio::piped())
			       	.stdout(std::process::Stdio::piped())
				.spawn () {
			Ok (output) => { output }
			Err (_) => {  return (0, "CHECK PACKAGES ERROR".to_string()); }
			};

		xargs_output.stdin.unwrap().write(String::from_utf8_lossy(&dpkg_output.stdout).as_bytes());
		drop(dpkg_output);

		let mut out: String = "".to_string();
		xargs_output.stdout.as_mut().unwrap().read_to_string(&mut out);

		// Check if the installed version is the latest available for each package
		let output_lines: Vec<&str> = out.split("\n").collect();
		if output_lines.len() == 1 { return (0, "CHECK PACKAGES ERROR".to_string()); }

		let mut package_list = vec![];
		let mut i = 0;

		while i < output_lines.len() {

			let line: Vec<&str> = output_lines[i].split(':').collect();
			if line[0] == "  Installed" {
				let package = format!("{}\n{}\n{}", output_lines[i-1], output_lines[i], output_lines[i+1]);
				package_list.push(package);
			}
			i = i + 1;
		}

		let (num_packages, packages_msg) = packages_updated(package_list);

		if !packages_msg.is_empty() {
			packages_update_needed = packages_msg;
		}

		return (num_packages, packages_update_needed);

	}

	fn packages_updated(package_list: Vec<String>) -> (isize, String) {

		let mut num_packages = 0;

		let mut message: String = "".to_string();

		for package in package_list.iter() {

			let package_array: Vec<&str> = package.split('\n').collect();

			let installed: Vec<&str> = package_array[1].trim().split(' ').collect();
			let candidate: Vec<&str> = package_array[2].trim().split(' ').collect();

			if (installed[1] != "(none)") && (installed[1] != candidate[1]) {
				message = format!("{}APT-WARNING: {} new version available.\n", message, package_array[0]);
				num_packages = num_packages + 1;

			}

		}

		return (num_packages, message);
	}
	*/

}

fn option_hours_string_to_seconds (
	hours_string_option: & Option <String>,
) -> Result <Option <u64>, Box <error::Error>> {

	match hours_string_option {

		& Some (ref string) =>
			Ok (Some (3600 * try! (
				u64::from_str_radix (
					string.as_str (),
					10)))),

		& None =>
			Ok (None),

	}

}

fn file_age_if_exists_in_seconds (
	file_path: & str,
) -> Result <Option <u64>, Box <error::Error>> {

	let metadata =
		match fs::metadata (
			& file_path) {

		Ok (metadata) =>
			metadata,

		Err (io_error) =>
			match io_error.kind () {

			io::ErrorKind::PermissionDenied =>
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

	let elapsed_seconds =
		elapsed_duration.as_secs ();

	Ok (Some (
		elapsed_seconds))

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
