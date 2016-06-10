use getopts;

use std::error;
use std::time;

use logic::simpleerror::*;

pub fn parse_decimal_fraction (
	options_matches: & getopts::Matches,
	option_name: & str,
) -> Result <Option <f64>, Box <error::Error>> {

	match options_matches.opt_str (
		option_name) {

		None =>
			Ok (None),

		Some (option_string) => {

			Ok (Some (
				try! (
					option_string.parse::<f64> (
					).map_err (
						|_|
						format! (
							"Invalid value for {}",
							option_name),
					))))

		},

	}

}

pub fn parse_duration (
	options_matches: & getopts::Matches,
	option_name: & str,
) -> Result <Option <time::Duration>, Box <error::Error>> {

	match options_matches.opt_str (
		option_name) {

		None =>
			Ok (None),

		Some (option_string) => {

			let (multiplier, suffix_length) =
				if option_string.ends_with ("ms") {
					(0, 2)
				} else if option_string.ends_with ("s") {
					(1000, 1)
				} else if option_string.ends_with ("m") {
					(1000 * 60, 1)
				} else if option_string.ends_with ("h") {
					(1000 * 60 * 60, 1)
				} else if option_string.ends_with ("d") {
					(1000 * 60 * 60 * 24, 1)
				} else {
					return Err (Box::new (
						SimpleError::from (
							format! (
								"units not specified or recognised for --{}",
								option_name))));
				};

			let quantity_string =
				& option_string [
					0 ..
					option_string.len () - suffix_length];

			let quantity_integer =
				try! (
					quantity_string.parse::<u64> (
					).map_err (
						|_|
						format! (
							"unable to parse value for --{}",
							option_name)
					));

			Ok (Some (
				time::Duration::from_millis (
					multiplier * quantity_integer)))

		}

	}

}

// ex: noet ts=4 filetype=rust
