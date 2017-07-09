use std::str;
use std::sync::Arc;
use std::sync::Mutex;

use chrono::NaiveDateTime;

use nom;

use der_parser;
use der_parser::DerObject;
use der_parser::DerObjectContent;

use rustls::Certificate as RustTlsCertificate;

pub fn get_certificate_validity (
	peer_certificates: & Option <Vec <RustTlsCertificate>>,
) -> Option <(NaiveDateTime, NaiveDateTime)> {

	if let Some (ref peer_certificates) =
		* peer_certificates {

		if let Some (ref peer_certificate) =
			peer_certificates.iter ().next () {

			let & RustTlsCertificate (ref peer_certificate) =
				* peer_certificate;

			return get_certificate_validity_real (
				& peer_certificate,
			).ok ();

		}

	}

	None

}

pub fn get_certificate_validity_real (
	bytes: & [u8],
) -> Result <(NaiveDateTime, NaiveDateTime), ()> {

	let raw =
		match der_parser::parse_der (
			& bytes,
		) {

		nom::IResult::Done (_remain, value) =>
			value,

		_ =>
			return Err (()),

	};

	let certificate =
		der_sequence (
			& raw,
		) ?;

	let certificate_main =
		der_sequence (
			& certificate [0],
		) ?;

	let certificate_validity =
		der_sequence (
			& certificate_main [4],
		) ?;

	let valid_from =
		der_utctime (
			& certificate_validity [0],
		) ?;

	let valid_to =
		der_utctime (
			& certificate_validity [1],
		) ?;

	Ok ((
		valid_from,
		valid_to,
	))

}

fn der_sequence <'a> (
	der_object: & 'a DerObject,
) -> Result <& 'a [DerObject <'a>], ()> {

	match der_object.content {

		DerObjectContent::Sequence (ref value) =>
			Ok (& value),

		_ =>
			Err (()),

	}

}

fn der_utctime <'a> (
	der_object: & 'a DerObject,
) -> Result <NaiveDateTime, ()> {

	match der_object.content {

		DerObjectContent::UTCTime (ref value) =>
			Ok (parse_utc_time (
				str::from_utf8 (
					& value,
				).map_err (|_| ()) ?,
			).map_err (|_| ()) ?),

		_ =>
			Err (()),

	}

}

fn parse_utc_time (
	time_string: & str,
) -> Result <NaiveDateTime, ()> {

	if time_string.len () == 11 {

		Ok (NaiveDateTime::parse_from_str (
			& format! (
				"20{}",
				& time_string [0 .. 10]),
			"%Y%m%d%H%M",
		).map_err (|_| ()) ?)

	} else if time_string.len () == 13 {

		Ok (NaiveDateTime::parse_from_str (
			& format! (
				"20{}",
				& time_string [0 .. 12]),
			"%Y%m%d%H%M%S",
		).map_err (|_| ()) ?)

	} else {

		Err (())

	}

}

// ex: noet ts=4 filetype=rust
