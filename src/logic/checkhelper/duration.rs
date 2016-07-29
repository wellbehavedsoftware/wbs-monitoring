use std::error;
use std::time;

use logic::*;

struct DurationFormat <'a> {

	seconds_singular: & 'a str,
	seconds_plural: & 'a str,

	milliseconds_singular: & 'a str,
	milliseconds_plural: & 'a str,

	microseconds_singular: & 'a str,
	microseconds_plural: & 'a str,

}

const DURATION_FORMAT_LONG: DurationFormat <'static> =
	DurationFormat {

		seconds_singular: " second",
		seconds_plural: " seconds",

		milliseconds_singular: " millisecond",
		milliseconds_plural: " milliseconds",

		microseconds_singular: " microsecond",
		microseconds_plural: " microseconds",

	};

const DURATION_FORMAT_SHORT: DurationFormat <'static> =
	DurationFormat {

		seconds_singular: "s",
		seconds_plural: "s",

		milliseconds_singular: "ms",
		milliseconds_plural: "ms",

		microseconds_singular: "µs",
		microseconds_plural: "µs",

	};

pub fn display_duration_long (
	duration: & time::Duration,
) -> String {

	display_duration_generic (
		duration,
		& DURATION_FORMAT_LONG)

}

pub fn display_duration_short (
	duration: & time::Duration,
) -> String {

	display_duration_generic (
		duration,
		& DURATION_FORMAT_SHORT)

}

fn display_duration_generic (
	duration: & time::Duration,
	duration_format: & DurationFormat,
) -> String {

	let seconds: u64 =
		duration.as_secs ();

	let microseconds: u64 =
		duration.subsec_nanos () as u64 / 1000;

	// TODO higher durations

	if seconds >= 100 {

		format! (
			"{}{}",
			seconds,
			duration_format.seconds_plural)

	} else if seconds >= 10 {

		format! (
			"{}.{:1}{}",
			seconds,
			microseconds / 100_000,
			duration_format.seconds_plural)

	} else if seconds >= 1 {

		format! (
			"{}.{:2}{}",
			seconds,
			microseconds / 10_000_000,
			if seconds == 1 {
				duration_format.seconds_singular
			} else {
				duration_format.seconds_plural
			})

	} else if microseconds >= 100_000 {

		format! (
			"{}{}",
			microseconds / 1000,
			duration_format.milliseconds_plural)

	} else if microseconds >= 10_000 {

		format! (
			"{}.{:1}{}",
			microseconds / 1000,
			(microseconds % 1000) / 100,
			duration_format.milliseconds_plural)

	} else if microseconds >= 1_000 {

		format! (
			"{}.{:2}{}",
			microseconds / 1000,
			(microseconds % 1000) / 10,
			if microseconds == 1_000 {
				duration_format.milliseconds_singular
			} else {
				duration_format.milliseconds_plural
			})

	} else {

		format! (
			"{}{}",
			microseconds,
			if microseconds == 1 {
				duration_format.microseconds_singular
			} else {
				duration_format.microseconds_plural
			})

	}

}

pub fn check_duration_less_than (
	check_result_builder: & mut CheckResultBuilder,
	warning_limit: & Option <time::Duration>,
	critical_limit: & Option <time::Duration>,
	message: & str,
	value: & time::Duration,
) -> Result <(), Box <error::Error>> {

	if critical_limit.is_some ()
		&& * value > critical_limit.unwrap () {

		check_result_builder.critical (
			format! (
				"{} (critical is {})",
				message,
				display_duration_short (
					& critical_limit.unwrap ())));

	} else if warning_limit.is_some ()
		&& * value > warning_limit.unwrap () {

		check_result_builder.warning (
			format! (
				"{} (warning is {})",
				message,
				display_duration_short (
					& warning_limit.unwrap ())));

	} else {

		check_result_builder.ok (
			format! (
				"{}",
				message));

	}

	Ok (())

}

// ex: noet ts=4 filetype=rust
