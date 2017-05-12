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

pub fn check_ratio_lesser_than (
	check_result_builder: & mut CheckResultBuilder,
	warning_limit: Option <f64>,
	critical_limit: Option <f64>,
	message: & str,
	value: f64,
) -> Result <(), Box <error::Error>> {

	if critical_limit.is_some ()
		&& value > critical_limit.unwrap () {

		check_result_builder.critical (
			format! (
				"{} or {}% (critical is {}%)",
				message,
				(value * 100.0) as u64,
				(critical_limit.unwrap () * 100.0) as u64));

	} else if warning_limit.is_some ()
		&& value > warning_limit.unwrap () {

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

// ex: noet ts=4 filetype=rust
