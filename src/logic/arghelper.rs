use getopts;

use std::error;
use std::marker;
use std::time;

use logic::simpleerror::*;

// ==================== boolean arguments

pub fn check_if_present (
	option_matches: & getopts::Matches,
	option_name: & str,
) -> Result <bool, Box <error::Error>> {

	Ok (

		option_matches.opt_present (
			option_name)

	)

}

// ==================== string arguments

pub fn parse_string (
	option_matches: & getopts::Matches,
	option_name: & str,
) -> Result <Option <String>, Box <error::Error>> {

	Ok (

		option_matches.opt_str (
			option_name)

	)

}

pub fn parse_string_or_default (
	option_matches: & getopts::Matches,
	option_name: & str,
	default_value: & str,
) -> Result <String, Box <error::Error>> {

	Ok (

		option_matches.opt_str (
			option_name,
		).unwrap_or_else (
			|| default_value.to_owned (),
		)

	)

}

pub fn parse_string_required (
	option_matches: & getopts::Matches,
	option_name: & str,
) -> Result <String, Box <error::Error>> {

	match option_matches.opt_str (
		option_name,
	) {

		None => Err (

			Box::new (
				SimpleError::from (
					format! (
						"Required argument '{}' not present",
						option_name)))

		),

		Some (value) =>
			Ok (value),

	}

}

pub fn parse_string_multiple (
	option_matches: & getopts::Matches,
	option_name: & str,
) -> Result <Vec <String>, Box <error::Error>> {

	Ok (

		option_matches.opt_strs (
			option_name)

	)

}

// ==================== enum arguments

pub trait EnumArg where Self: marker::Sized {

	fn from_string (
		string_value: & str,
	) -> Option <Self>;

}

pub fn parse_enum <EnumType: EnumArg> (
	options_matches: & getopts::Matches,
	option_name: & str,
) -> Result <Option <EnumType>, Box <error::Error>> {

	match options_matches.opt_str (
		option_name) {

		None =>
			Ok (None),

		Some (option_string) =>
			match EnumType::from_string (
				& option_string) {

			Some (option_enum) =>
				Ok (

				Some (
					option_enum)

			),

			None =>
				Err (

				Box::new (
					SimpleError::from (
						format! (
							"Invalid value for {}",
								option_name)))

			),

		},

	}

}

// ==================== fractional arguments

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

// ==================== integer arguments

pub fn parse_positive_integer (
	options_matches: & getopts::Matches,
	option_name: & str,
) -> Result <Option <u64>, Box <error::Error>> {

	match options_matches.opt_str (
		option_name) {

		None =>
			Ok (None),

		Some (option_string) => {

			let value =
				try! (
					option_string.parse::<u64> (
					).map_err (
						|_|
						format! (
							"Parameter {} must be a positive integer",
							option_name),
					));

			if value < 1 {

				return Err (
					Box::new (
						SimpleError::from (
							format! (
								"Parameter {} must be a positive integer, but \
								got {}",
								option_name,
								value)))
				);

			}

			Ok (Some (
				value
			))

		},

	}

}

pub fn parse_positive_integer_or_default (
	options_matches: & getopts::Matches,
	option_name: & str,
	default_value: u64,
) -> Result <u64, Box <error::Error>> {

	Ok (

		try! (

			parse_positive_integer (
				options_matches,
				option_name)

		).unwrap_or (
			default_value,
		)

	)

}

pub fn parse_positive_integer_multiple (
	option_matches: & getopts::Matches,
	option_name: & str,
) -> Result <Vec <u64>, Box <error::Error>> {

	let mut return_values: Vec <u64> =
		vec! [];

	for option_value in option_matches.opt_strs (
		option_name) {

		let integer_value =
			try! (
				option_value.parse::<u64> (
				).map_err (
					|_|
					format! (
						"Parameter {} must be a positive integer",
						option_name),
				));

		if integer_value < 1 {

			return Err (
				Box::new (
					SimpleError::from (
						format! (
							"Parameter {} must be a positive integer, but got \
							{}",
							option_name,
							integer_value)))
			);

		}

		return_values.push (
			integer_value);

	}

	Ok (
		return_values
	)

}

pub fn parse_positive_integer_multiple_or_default (
	option_matches: & getopts::Matches,
	option_name: & str,
	default_value: & [u64],
) -> Result <Vec <u64>, Box <error::Error>> {

	let return_values =
		try! (
			parse_positive_integer_multiple (
				option_matches,
				option_name));

	Ok (

		if return_values.is_empty () {
			default_value.to_owned ()
		} else {
			return_values
		}

	)

}

// ==================== duration arguments

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

pub fn parse_duration_or_default (
	options_matches: & getopts::Matches,
	option_name: & str,
	default_value: & time::Duration,
) -> Result <time::Duration, Box <error::Error>> {

	Ok (

		try! (
			parse_duration (
				options_matches,
				option_name)
		).unwrap_or (
			default_value.to_owned (),
		)

	)

}

// ex: noet ts=4 filetype=rust
