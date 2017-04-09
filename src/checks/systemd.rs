extern crate getopts;
extern crate systemd_jp;

use std::cell::RefCell;
use std::error;
use std::rc::Rc;

use self::systemd_jp::*;

use logic::*;

check! {

	new = new,
	name = "systemd",
	prefix = "SYSTEMD",

	provider = CheckSystemdProvider,

	instance = CheckSystemdInstance {
	},

	options_spec = |options_spec| {
	},

	options_parse = |_options_matches| {

		CheckSystemdInstance {
		}

	},

	perform = |self, plugin_provider, check_result_builder| {

		let systemd_connection =
			Rc::new (RefCell::new (
				SystemdConnection::new () ?,
			));

		let systemd_manager =
			SystemdManager::new (
				systemd_connection,
			);

		let mut systemd_units =
			systemd_manager.list_units () ?;

		systemd_units.sort_by (
			|ref left, ref right| left.name ().cmp (right.name ()));

		for unit_state in vec! [

			SystemdActiveState::Active,
			SystemdActiveState::Inactive,

			SystemdActiveState::Reloading,
			SystemdActiveState::Activating,
			SystemdActiveState::Deactivating,

			SystemdActiveState::Failed,

		] {

			let num_units =
				systemd_units.iter ().filter (
					|systemd_unit|
					* systemd_unit.active_state () == unit_state
				).count ();

			if unit_state == SystemdActiveState::Active {

				check_result_builder.ok (
					format! (
						"{} active",
						num_units));

			} else if num_units > 0 {

				match unit_state {

					SystemdActiveState::Inactive =>
						check_result_builder.ok (
							format! (
								"{} inactive",
								num_units)),

					SystemdActiveState::Reloading =>
						check_result_builder.warning (
							format! (
								"{} reloading",
								num_units)),

					SystemdActiveState::Activating =>
						check_result_builder.warning (
							format! (
								"{} activating",
								num_units)),

					SystemdActiveState::Deactivating =>
						check_result_builder.warning (
							format! (
								"{} deactivating",
								num_units)),

					SystemdActiveState::Failed =>
						check_result_builder.critical (
							format! (
								"{} failed",
								num_units)),

					_ => (),

				};

			}

			let num_units_other =
				systemd_units.iter ().filter (
					|systemd_unit|

					match * systemd_unit.active_state () {
						SystemdActiveState::Other (_) => true,
						_ => false,
					}

				).count ();

			if num_units_other > 0 {

				check_result_builder.warning (
					format! (
						"{} in unknown state",
						num_units));

			}

		}

		for systemd_unit in systemd_units {

			match * systemd_unit.active_state () {

				SystemdActiveState::Reloading
				| SystemdActiveState::Activating
				| SystemdActiveState::Deactivating
				| SystemdActiveState::Failed
				| SystemdActiveState::Other (_) =>

					check_result_builder.extra_information (
						format! (
							"{}: {} {} {}",
							systemd_unit.name (),
							systemd_unit.load_state ().as_str (),
							systemd_unit.active_state ().as_str (),
							systemd_unit.sub_state ().as_str ())),

				_ => (),

			}

		}

	},

}

impl CheckSystemdInstance {

}

// ex: noet ts=4 filetype=rust
