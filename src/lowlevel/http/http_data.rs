use super::http_prelude::*;

#[ derive (Clone, Copy, Debug, PartialEq) ]
pub enum HttpMethod {

	Get,
	Post,

}

pub struct HttpRequest {

	pub method: HttpMethod,
	pub path: String,
	pub headers: Vec <(String, String)>,

}

pub struct HttpResponse {

	pub (in super) status_code: u64,
	pub (in super) status_message: String,

	pub (in super) headers: Vec <(String, String)>,

	pub (in super) body: Vec <u8>,
	pub (in super) body_encoding: Option <String>,

	pub (in super) request_duration: Duration,
	pub (in super) response_duration: Duration,

}

pub enum HttpError {
	Timeout,
	InvalidUri,
	Unknown (Box <Error>),
}

pub type HttpResult <Ok> =
	Result <Ok, HttpError>;

impl HttpResponse {

	pub fn status_code (& self) -> u64 {
		self.status_code
	}

	pub fn status_message (& self) -> & str {
		& self.status_message
	}

	pub fn headers (& self) -> & [(String, String)] {
		& self.headers
	}

	pub fn body (& self) -> & [u8] {
		& self.body
	}

	pub fn body_encoding (& self) -> & Option <String> {
		& self.body_encoding
	}

	pub fn request_duration (& self) -> Duration {
		self.request_duration
	}

	pub fn response_duration (& self) -> Duration {
		self.response_duration
	}

	pub fn duration (& self) -> Duration {
		self.request_duration + self.response_duration
	}

	pub fn body_string (
		& self,
	) -> Result <String, String> {

		let body_encoding =
			self.body_encoding.as_ref ().ok_or_else (
				|| "response encoding not specified".to_string (),
			) ?;

		let encoding =
			encoding_from_whatwg_label (
				& body_encoding,
			).ok_or_else (||

				format! (
					"response encoding not recognised: {}",
					body_encoding)

			) ?;

		Ok (
			encoding.decode (
				& self.body,
				EncodingDecoderTrap::Strict,
			) ?.to_string ()
		)

	}

}

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
