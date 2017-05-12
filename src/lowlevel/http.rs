use std::error;
use std::fmt;
use std::io::Read;
use std::time;

use hyper::Client as HyperClient;
use hyper::error::Result as HyperResult;
use hyper::header::Headers as HyperHeaders;
use hyper::http::RawStatus as HyperRawStatus;
use hyper::net::HttpsConnector as HyperHttpsConnector;
use hyper::net::NetworkStream as HyperNetworkStream;
use hyper::net::SslClient as HyperSslClient;
use hyper_native_tls::NativeTlsClient as HyperNativeTlsClient;
use hyper_native_tls::TlsStream as HyperNativeTlsStream;

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

	pub timeout: time::Duration,

}

#[ derive (Debug) ]
pub struct HttpResponse {
	pub status_code: u64,
	pub status_message: String,
	pub headers: Vec <(String, String)>,
	pub body: String,
	pub duration: time::Duration,
}

#[ derive (Debug) ]
pub enum PerformRequestResult {
	Success (HttpResponse),
	Timeout (time::Duration),
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

	let mut hyper_client =
		if http_request.secure {

		let hyper_ssl_client =
			SniSslClient {

			nested_client:
				HyperNativeTlsClient::new ().unwrap (),

			hostname:
				http_request.hostname.to_string (),

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

	for & (ref header_name, ref header_value)
		in http_request.headers.iter () {

		hyper_headers.set_raw (
			header_name.to_string (),
			vec! [ header_value.as_bytes ().to_vec () ]);

	}

	let hyper_request =
		hyper_request.headers (
			hyper_headers);

	// perform request

	let start_time =
		time::Instant::now ();

	let mut hyper_response =
		hyper_request.send () ?;

	let mut response_body =
		String::new ();

	hyper_response.read_to_string (
		& mut response_body,
	) ?;

	let end_time =
		time::Instant::now ();

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

	// return

	Ok (
		PerformRequestResult::Success (

		HttpResponse {
			status_code: response_status_code,
			status_message: response_status_message,
			headers: response_headers,
			body: response_body,
			duration: duration,
		}

	))

}

struct SniSslClient {
	nested_client: HyperNativeTlsClient,
	hostname: String,
}

impl <
	NetworkStream: HyperNetworkStream + Send + Clone + fmt::Debug + Sync
> HyperSslClient <NetworkStream> for SniSslClient {

	type Stream = HyperNativeTlsStream <NetworkStream>;

	fn wrap_client (
		& self,
		stream: NetworkStream,
		_host: & str,
	) -> HyperResult <Self::Stream> {

		self.nested_client.wrap_client (
			stream,
			& self.hostname,
		)

    }

}

// ex: noet ts=4 filetype=rust
