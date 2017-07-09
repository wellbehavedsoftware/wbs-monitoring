use std::error::Error;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FormatResult;
use std::time::Duration;

#[ derive (Clone, Copy, Debug, PartialEq) ]
pub enum HttpMethod {

	Get,
	Post,

}

pub struct HttpRequest {

	pub method: HttpMethod,
	pub path: HttpMethod,
	pub headers: Vec <(String, String)>,

}

pub struct HttpResponse {

	pub status_code: u64,
	pub status_message: String,

	pub headers: Vec <(String, String)>,

	pub body: Vec <u8>,
	pub body_encoding: Option <String>,

	pub request_duration: Duration,
	pub response_duration: Duration,

}

pub enum HttpError {
	Timeout,
	InvalidUri,
	Unknown (Box <Error>),
}

pub type HttpResult <Ok> =
	Result <Ok, HttpError>;

impl Error for HttpError {

	fn description (& self) -> & str {

		match self {

			& HttpError::Timeout =>
				"Timeout",

			& HttpError::InvalidUri =>
				"Invalid URI",

			& HttpError::Unknown (ref error) =>
				error.description (),

		}

	}

}

impl Debug for HttpError {

	fn fmt (
		& self,
		formatter: & mut Formatter,
	) -> FormatResult {

		formatter.write_str (
			"HttpError (..)",
		) ?;

		Ok (())

	}

}

impl Display for HttpError {

	fn fmt (
		& self,
		formatter: & mut Formatter,
	) -> FormatResult {

		formatter.write_str (
			"HttpError (..)",
		) ?;

		Ok (())

	}

}

// ex: noet ts=4 filetype=rust
