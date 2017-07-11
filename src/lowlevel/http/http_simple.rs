use super::http_prelude::*;

#[ derive (Clone, Copy, Debug) ]
pub struct HttpSimpleRequest <'a> {

	pub address: & 'a str,
	pub hostname: & 'a str,
	pub port: u64,
	pub secure: bool,

	pub method: HttpMethod,
	pub path: & 'a str,
	pub headers: & 'a Vec <(String, String)>,

	pub timeout: Duration,

}

#[ derive (Debug) ]
pub struct HttpSimpleResponse {

	status_code: u64,
	status_message: String,

	headers: Vec <(String, String)>,

	body: Vec <u8>,
	body_encoding: Option <String>,

	connect_duration: Duration,
	request_duration: Duration,
	response_duration: Duration,

	certificate_expiry: Option <NaiveDateTime>,

}

#[ derive (Debug) ]
pub enum HttpSimpleResult {
	Success (HttpSimpleResponse),
	Timeout,
	Failure (String),
}

impl HttpSimpleResponse {

	pub fn status_code (& self) -> u64 {
		self.status_code
	}

	pub fn status_message (& self) -> & str {
		& self.status_message
	}

	pub fn headers (& self) -> & [(String, String)] {
		& self.headers
	}

	pub fn body_bytes (& self) -> & [u8] {
		& self.body
	}

	pub fn body_encoding (& self) -> & Option <String> {
		& self.body_encoding
	}

	pub fn connect_duration (& self) -> Duration {
		self.connect_duration
	}

	pub fn request_duration (& self) -> Duration {
		self.request_duration
	}

	pub fn response_duration (& self) -> Duration {
		self.response_duration
	}

	pub fn duration (& self) -> Duration {
		self.connect_duration
		+ self.request_duration
		+ self.response_duration
	}

	pub fn certificate_expiry (& self) -> Option <NaiveDateTime> {
		self.certificate_expiry
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

pub fn http_simple_perform (
	http_request: & HttpSimpleRequest,
) -> HttpResult <HttpSimpleResponse> {

	if http_request.method != HttpMethod::Get {

		return Err (HttpError::InvalidUri);

	}

	// connect

	let mut http_connection =
		HttpConnection::connect (
			http_request.address.to_string (),
			Some (http_request.port),
			http_request.secure,
			http_request.hostname.to_string (),
		) ?;

	// perform request

	let http_response =
		http_connection.perform (
			HttpRequest {
				method: HttpMethod::Get,
				path: http_request.path.to_string (),
				headers: http_request.headers.clone (),
			},
			http_request.timeout,
		) ?;

	// return

	Ok (HttpSimpleResponse {

		status_code: http_response.status_code (),
		status_message: http_response.status_message ().to_string (),

		headers: http_response.headers ().to_vec (),

		body: http_response.body ().to_vec (),
		body_encoding: http_response.body_encoding ().clone (),

		connect_duration: http_connection.connect_duration (),
		request_duration: http_response.request_duration (),
		response_duration: http_response.response_duration (),

		certificate_expiry: http_connection.certificate_expiry (),

	})

}

// ex: noet ts=4 filetype=rust
