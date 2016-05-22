extern crate getopts;

use std::error;
use std::fs;

use logic::*;

pub fn new (
) -> Box <PluginProvider> {

	Box::new (
		CheckLxcContainerProvider {},
	)

}

struct CheckLxcContainerProvider {
}

struct CheckLxcContainerInstance {
	container_name: String,
	critical_states: Vec <ContainerState>,
}

impl PluginProvider
for CheckLxcContainerProvider {

	fn name (
		& self,
	) -> & str {
		"check-lxc-container"
	}

	fn prefix (
		& self,
	) -> & str {
		"LXC"
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
			"container-name",
			"name of the container to check",
			"NAME");

		options_spec.optmulti (
			"",
			"critical-state",
			"container state which cause a critical status",
			"STATE");

		options_spec

	}

	fn new_instance (
		& self,
		options_spec: & getopts::Options,
		options_matches: & getopts::Matches,
	) -> Result <Box <PluginInstance>, Box <error::Error>> {

		let container_name =
			options_matches.opt_str (
				"container-name",
			).unwrap ();

		let mut critical_states: Vec <ContainerState> =
			vec! [];

		for container_state_string in options_matches.opt_strs (
			"critical-state",
		) {

			critical_states.push (
				container_state_from_str (
					container_state_string.as_str ()));

		}

		return Ok (Box::new (
			CheckLxcContainerInstance {
				container_name: container_name,
				critical_states: critical_states,
			}
		));

	}

}

impl PluginInstance
for CheckLxcContainerInstance {

	fn perform_check (
		& self,
		plugin_provider: & PluginProvider,
	) -> Result <CheckResult, Box <error::Error>> {

		let mut check_result_builder =
			CheckResultBuilder::new ();

		let container_path =
			format! (
				"/var/lib/lxc/{}",
				self.container_name);

		let metadata =
			match fs::metadata (
				container_path,
			) {

			Err (_) => {

				self.check_not_present (
					& mut check_result_builder)

			},

			Ok (metadata) => {

				self.check_present (
					& mut check_result_builder)

			},

		};

		Ok (
			check_result_builder.into_check_result (
				plugin_provider,
			)
		)

	}

}

impl CheckLxcContainerInstance {

	fn check_not_present (
		& self,
		check_result: & mut CheckResultBuilder,
	) {

		if self.critical_states.contains (
			& ContainerState::NotPresent,
		) {

			check_result.critical (
				format! (
					"container {} not present",
					self.container_name));

		} else {

			check_result.ok (
				format! (
					"container {} not present",
					self.container_name));

		}

	}

	fn check_present (
		& self,
		check_result: & mut CheckResultBuilder,
	) {

		if self.critical_states.contains (
			& ContainerState::Present,
		) {

			check_result.critical (
				format! (
					"container {} present",
					self.container_name));

		} else {

			check_result.ok (
				format! (
					"container {} present",
					self.container_name));

		}

	}

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

// ex: noet ts=4 filetype=rust
