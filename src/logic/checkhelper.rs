use std::error;

use logic::*;

pub fn check_ratio_greater_than (
	check_result_builder: & mut CheckResultBuilder,
	warning_limit: Option <f64>,
	critical_limit: Option <f64>,
	message: & str,
	value: f64,
) -> Result <(), Box <error::Error>> {

	if critical_limit.is_some ()
		&& value < critical_limit.unwrap () {

		check_result_builder.critical (
			format! (
				"{} or {}% (critical is {}%)",
				message,
				(value * 100.0) as u64,
				(critical_limit.unwrap () * 100.0) as u64));

	} else if warning_limit.is_some ()
		&& value < warning_limit.unwrap () {

		check_result_builder.warning (
			format! (
				"{} or {}% (warning is {}%)",
				message,
				(value * 100.0) as u64,
				(warning_limit.unwrap () * 100.0) as u64));

	} else {

		check_result_builder.ok (
			format! (
				"{} or {}%",
				message,
				(value * 100.0) as u64));

	}

	Ok (())

}

pub fn display_data_size (
	size_in_bytes: u64,
) -> String {

	let scale = 4;

	if size_in_bytes == 0 {

		"0".to_string ()

	} else if size_in_bytes < scale * 1024 {

		format! (
			"{} B",
			size_in_bytes)

	} else if size_in_bytes < scale * 1024 * 1024 {

		format! (
			"{} KiB",
			size_in_bytes / 1024)

	} else if size_in_bytes < scale * 1024 * 1024 * 1024 {

		format! (
			"{} MiB",
			size_in_bytes / 1024 / 1024)

	} else if size_in_bytes < scale * 1024 * 1024 * 1024 * 1024 {

		format! (
			"{} GiB",
			size_in_bytes / 1024 / 1024 / 1024)

	} else {

		format! (
			"{} TiB",
			size_in_bytes / 1024 / 1024 / 1024 / 1024)

	}

}

pub fn display_data_size_ratio (
	numerator_in_bytes: u64,
	denominator_in_bytes: u64,
) -> String {

	let scale = 4;

	if denominator_in_bytes == 0 {

		"0".to_string ()

	} else if denominator_in_bytes < scale * 1024 {

		format! (
			"{} of {} B",
			numerator_in_bytes,
			denominator_in_bytes)

	} else if denominator_in_bytes < scale * 1024 * 1024 {

		format! (
			"{} of {} KiB",
			numerator_in_bytes / 1024,
			denominator_in_bytes / 1024)

	} else if denominator_in_bytes < scale * 1024 * 1024 * 1024 {

		format! (
			"{} of {} MiB",
			numerator_in_bytes / 1024 / 1024,
			denominator_in_bytes / 1024 / 1024)

	} else if denominator_in_bytes < scale * 1024 * 1024 * 1024 * 1024 {

		format! (
			"{} of {} GiB",
			numerator_in_bytes / 1024 / 1024 / 1024,
			denominator_in_bytes / 1024 / 1024 / 1024)

	} else {

		format! (
			"{} of {} TiB",
			numerator_in_bytes / 1024 / 1024 / 1024 / 1024,
			denominator_in_bytes / 1024 / 1024 / 1024 / 1024)

	}

}

// ex: noet ts=4 filetype=rust
