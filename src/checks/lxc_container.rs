extern crate getopts;

use std::error;
use std::fs;

use logic::*;

check! {

	new = new,
	name = "check-lxc-container",
	prefix = "LXC",

	provider = CheckLxcContainerProvider,

	instance = CheckLxcContainerInstance {

		container_name: String,
		critical_states: Vec <ContainerState>,

	},

	options_spec = |options_spec| {

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

	},

	options_parse = |options_matches| {

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

		CheckLxcContainerInstance {

			container_name: container_name,
			critical_states: critical_states,

		}

	},

	perform = |self, plugin_provider, check_result_builder| {

		let container_path =
			format! (
				"/var/lib/lxc/{}",
				self.container_name);

		match fs::metadata (
			container_path) {

			Err (_) => {

				self.check_not_present (
					& mut check_result_builder)

			},

			Ok (_metadata) => {

				self.check_present (
					& mut check_result_builder)

			},

		};

	},

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
