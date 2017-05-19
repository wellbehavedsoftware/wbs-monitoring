use std::error;
use std::io::Read;
use std::str;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;

use chrono::NaiveDateTime;

use encoding::DecoderTrap;
use encoding::label::encoding_from_whatwg_label;

use hyper::Client as HyperClient;
use hyper::error::Result as HyperResult;
use hyper::header::ContentType as HyperContentTypeHeader;
use hyper::header::Headers as HyperHeaders;
use hyper::header::Host as HyperHostHeader;
use hyper::http::RawStatus as HyperRawStatus;
use hyper::mime::Attr as HyperAttr;
use hyper::net::HttpStream as HyperHttpStream;
use hyper::net::HttpsConnector as HyperHttpsConnector;
use hyper::net::SslClient as HyperSslClient;
use hyper_rustls::TlsClient as HyperRustTlsClient;
use hyper_rustls::WrappedStream as HyperRustTlsWrappedStream;

use nom;

use der_parser;
use der_parser::DerObject;
use der_parser::DerObjectContent;

use rustls::Certificate as RustTlsCertificate;

use logic::*;

#[ derive (Clone, Copy, Debug, PartialEq) ]
pub enum HttpMethod {
	Get,
	Post,
}

#[ derive (Clone, Copy, Debug) ]
pub struct HttpRequest <'a> {

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
pub struct HttpResponse {

	status_code: u64,
	status_message: String,

	headers: Vec <(String, String)>,

	body: Vec <u8>,
	body_encoding: Option <String>,

	duration: Duration,
	certificate_expiry: Option <NaiveDateTime>,

}

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

#[ derive (Debug) ]
pub enum PerformRequestResult {
	Success (HttpResponse),
	Timeout (Duration),
	Failure (String),
}

pub fn perform_request (
	http_request: & HttpRequest,
) -> PerformRequestResult {

	perform_request_real (
		http_request,
	).unwrap_or_else (
		|error|

		PerformRequestResult::Failure (
			error.description ().to_owned (),
		)

	)

}

pub fn perform_request_real (
	http_request: & HttpRequest,
) -> Result <PerformRequestResult, Box <error::Error>> {

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

	let wrapped_stream: Arc <Mutex <Option <HyperRustTlsWrappedStream>>> =
		Arc::new (Mutex::new (None));

	let mut hyper_client =
		if http_request.secure {

		let hyper_ssl_client =
			SniSslClient {

			nested_client:
				HyperRustTlsClient::new (),

			hostname:
				http_request.hostname.to_string (),

			wrapped_stream:
				wrapped_stream.clone (),

		};

		let hyper_connector =
			HyperHttpsConnector::new (
				hyper_ssl_client);

		HyperClient::with_connector (
			hyper_connector)

	} else {

		HyperClient::new ()

	};

	hyper_client.set_read_timeout (
		Some (http_request.timeout));

	hyper_client.set_write_timeout (
		Some (http_request.timeout));

	// setup request

	let hyper_request =
		hyper_client.get (
			& url);

	let mut hyper_headers =
		HyperHeaders::new ();

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
				HyperHostHeader {
					hostname: http_request.hostname.to_string (),
					port: None,
				}
			)

		} else {

			hyper_headers.set (
				HyperHostHeader {
					hostname: http_request.hostname.to_string (),
					port: Some (http_request.port as u16),
				}
			)

		}

	}

	let hyper_request =
		hyper_request.headers (
			hyper_headers);

	// perform request

	let start_time =
		Instant::now ();

	let mut hyper_response =
		hyper_request.send () ?;

	let mut response_body: Vec <u8> =
		Vec::new ();

	hyper_response.read_to_end (
		& mut response_body,
	) ?;

	let certificate_expiry =
		get_certificate_validity_from_wrapped_stream (
			wrapped_stream,
		).map (
			|(_start, end)| end,
		);

	let end_time =
		Instant::now ();

	let duration =
		end_time.duration_since (
			start_time);

	// process response

	let HyperRawStatus (
		ref response_status_code,
		ref response_status_message,
	) = * hyper_response.status_raw ();

	let response_status_code =
		* response_status_code as u64;

	let response_status_message =
		response_status_message.to_string ();

	let response_headers: Vec <(String, String)> =
		hyper_response.headers.iter ().map (
			|header|
			(
				header.name ().to_string (),
				header.value_string (),
			)
		).collect ();

	let response_encoding =
		if let Some (response_content_type) =
			hyper_response.headers.get::<HyperContentTypeHeader> () {

		if let Some (response_charset) =
			response_content_type.get_param (
				HyperAttr::Charset,
			) {

			Some (response_charset.to_string ())

		} else {
			None
		}

	} else {
		None
	};

	// return

	Ok (
		PerformRequestResult::Success (

		HttpResponse {
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

struct SniSslClient {
	nested_client: HyperRustTlsClient,
	hostname: String,
	wrapped_stream: Arc <Mutex <Option <HyperRustTlsWrappedStream>>>,
}

impl HyperSslClient for SniSslClient {

	type Stream = HyperRustTlsWrappedStream;

	fn wrap_client (
		& self,
		stream: HyperHttpStream,
		_host: & str,
	) -> HyperResult <Self::Stream> {

		let result =
			self.nested_client.wrap_client (
				stream,
				& self.hostname,
			);

		if let Ok (ref wrapped_stream) = result {

			let mut wrapped_stream_lock =
				self.wrapped_stream.lock ().unwrap ();

			* wrapped_stream_lock =
				Some (wrapped_stream.clone ());

		}

		result

    }

}

fn get_certificate_validity_from_wrapped_stream (
	wrapped_stream: Arc <Mutex <Option <HyperRustTlsWrappedStream>>>,
) -> Option <(NaiveDateTime, NaiveDateTime)> {

	let wrapped_stream =
		wrapped_stream.lock ().unwrap ();

	if let Some (ref wrapped_stream) =
		* wrapped_stream {

		let tls_stream =
			wrapped_stream.to_tls_stream ();

		let tls_session =
			tls_stream.get_session ();

		if let Some (peer_certificates) =
			tls_session.get_peer_certificates () {

			if let Some (ref peer_certificate) =
				peer_certificates.iter ().next () {

				let & RustTlsCertificate (ref peer_certificate) =
					* peer_certificate;

				return get_certificate_validity (
					& peer_certificate,
				).ok ();

			}

		}

	}

	None

}


fn get_certificate_validity (
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
