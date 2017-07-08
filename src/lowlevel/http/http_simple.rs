use std::error;
use std::str;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;

use chrono::NaiveDateTime;

use encoding::DecoderTrap;
use encoding::label::encoding_from_whatwg_label;

use futures::Future;
use futures::IntoFuture;
use futures::Stream;
use futures::future::Either;

use hyper::Client as HyperClient;
use hyper::Method as HyperMethod;
use hyper::client::Request as HyperRequest;
use hyper::header::ContentType as HyperContentTypeHeader;
use hyper::header::Host as HyperHostHeader;

use rustls::Certificate as RustTlsCertificate;

use tokio_core::reactor::Core as TokioCore;
use tokio_core::reactor::Timeout as TokioTimeout;

use logic::*;

use super::http_data::HttpMethod;
use super::http_certificate::get_certificate_validity;
use super::http_sni_connector::SniConnector;

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

	duration: Duration,
	certificate_expiry: Option <NaiveDateTime>,

}

#[ derive (Debug) ]
pub enum HttpSimpleResult {
	Success (HttpSimpleResponse),
	Timeout (Duration),
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

	pub fn duration (& self) -> Duration {
		self.duration
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
				DecoderTrap::Strict,
			) ?.to_string ()
		)

	}

}

pub fn http_simple_perform (
	http_request: & HttpSimpleRequest,
) -> HttpSimpleResult {

	http_simple_perform_real (
		http_request,
	).unwrap_or_else (
		|error|

		HttpSimpleResult::Failure (
			error.description ().to_owned (),
		)

	)

}

fn http_simple_perform_real (
	http_request: & HttpSimpleRequest,
) -> Result <HttpSimpleResult, Box <error::Error>> {

	let url =
		format! (
			"{}://{}:{}{}",
			if http_request.secure { "https" } else { "http" },
			http_request.address,
			http_request.port,
			http_request.path);

	if http_request.method != HttpMethod::Get {

		return Err (

			Box::new (
				SimpleError::from (
					"TODO: only supports GET method so far"))

		);

	}

	// setup client

	let mut tokio_core =
		TokioCore::new () ?;

	// setup client

	let peer_certificates: Arc <Mutex <Option <Vec <RustTlsCertificate>>>> =
		Arc::new (Mutex::new (None));

	let sni_connector =
		SniConnector::new (
			1,
			& tokio_core.handle (),
			http_request.hostname.to_string (),
			peer_certificates.clone (),
		);

	let hyper_client =
		HyperClient::configure (
		).connector (
			sni_connector,
		).build (
			& tokio_core.handle (),
		);

	// setup request

	let mut hyper_request =
		HyperRequest::new (
			HyperMethod::Get,
			url.parse () ?);

	{

		let hyper_headers =
			hyper_request.headers_mut ();

		let mut got_host = false;

		for & (ref header_name, ref header_value)
			in http_request.headers.iter () {

			let header_name =
				header_name.to_lowercase ();

			if header_name == "host" {
				got_host = true;
			}

			hyper_headers.set_raw (
				header_name.to_string (),
				vec! [ header_value.as_bytes ().to_vec () ]);

		}

		if ! got_host {

			if (
				! http_request.secure
				&& http_request.port == 80
			) || (
				http_request.secure
				&& http_request.port == 443
			) {

				hyper_headers.set (
					HyperHostHeader::new (
						http_request.hostname.to_string (),
						None));

			} else {

				hyper_headers.set (
					HyperHostHeader::new (
						http_request.hostname.to_string (),
						http_request.port as u16));

			}

		}

	}

	// perform request

	let start_time =
		Instant::now ();

	let timeout_time =
		start_time + http_request.timeout;

	let timeout =
		TokioTimeout::new_at (
			timeout_time,
			& tokio_core.handle (),
		).into_future ().flatten ();

	let hyper_response =
		match tokio_core.run (

		hyper_client.request (
			hyper_request,
		).select2 (timeout)

	) {

		Ok (Either::A ((hyper_response, _))) =>
			hyper_response,

		Err (Either::A (_)) =>
			return Ok (HttpSimpleResult::Failure (
				format! (
					"Unknown error performing request"))),

		_ =>
			return Ok (HttpSimpleResult::Timeout (
				Instant::now () - start_time)),

	};

	let certificate_expiry =
		get_certificate_validity (
			peer_certificates,
		).map (
			|(_start, end)| end,
		);

	// process response headers

	let response_status_code =
		hyper_response.status_raw ().0 as u64;

	let response_status_message =
		hyper_response.status_raw ().1.to_string ();

	let response_headers: Vec <(String, String)> =
		hyper_response.headers ().iter ().map (
			|header|
			(
				header.name ().to_string (),
				header.value_string (),
			)
		).collect ();

	let response_encoding =
		if let Some (response_content_type) =
			hyper_response.headers ().get::<HyperContentTypeHeader> () {

		if let Some (response_charset) =
			response_content_type.get_param (
				"charset",
			) {

			Some (response_charset.to_string ())

		} else {
			None
		}

	} else {
		None
	};

	// process response body

	let mut response_body: Vec <u8> =
		Vec::new ();

	let timeout =
		TokioTimeout::new_at (
			timeout_time,
			& tokio_core.handle (),
		).into_future ().flatten ();

	match tokio_core.run (

		hyper_response.body ().for_each (
			|chunk| {

			response_body.extend_from_slice (
				& chunk);

			Ok (())

		}).select2 (timeout)

	) {

		Ok (Either::A (_)) =>
			(),

		Err (Either::A (_)) =>
			return Ok (HttpSimpleResult::Failure (
				format! (
					"Unknown error performing request"))),

		_ =>
			return Ok (HttpSimpleResult::Timeout (
				Instant::now () - start_time)),

	};

	let end_time =
		Instant::now ();

	let duration =
		end_time.duration_since (
			start_time);

	// return

	Ok (
		HttpSimpleResult::Success (

		HttpSimpleResponse {
			status_code: response_status_code,
			status_message: response_status_message,
			headers: response_headers,
			body: response_body,
			body_encoding: response_encoding,
			duration: duration,
			certificate_expiry: certificate_expiry,
		}

	))

}

// ex: noet ts=4 filetype=rust
