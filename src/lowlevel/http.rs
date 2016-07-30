extern crate curl;

use std::error;
use std::time;

use logic::*;

#[ derive (Clone, Copy, Debug, PartialEq) ]
pub enum HttpMethod {
	Get,
	Post,
}

#[ derive (Clone, Copy, Debug) ]
pub struct HttpRequest <'a> {

	pub address: & 'a str,
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
	//status_message: String,
	pub body: String,
	pub duration: time::Duration,
}

#[ derive (Debug) ]
pub enum PerformRequestResult {
	Success (HttpResponse),
	Timeout (time::Duration),
}

pub fn perform_request (
	http_request: & HttpRequest,
) -> Result <PerformRequestResult, Box <error::Error>> {

	// setup request

	let url =
		format! (
			"{}://{}:{}{}",
			if http_request.secure { "https" } else { "http" },
			http_request.address,
			http_request.port,
			http_request.path);

   	let mut curl_easy =
   		curl::easy::Easy::new ();

	if http_request.method != HttpMethod::Get {

		return Err (

			Box::new (
				SimpleError::from (
					"TODO: only supports GET method so far"))

		);

	}

	try! (
		curl_easy.get (
			true));

	try! (
		curl_easy.url (
			url.as_str ()));

	try! (
		curl_easy.timeout (
			http_request.timeout));

	// setup request headers

	let mut curl_headers =
		curl::easy::List::new ();

	for & (ref header_name, ref header_value)
		in http_request.headers.iter () {

		try! (
			curl_headers.append (
				& format! (
					"{}: {}",
					header_name,
					header_value)));

	}

	try! (
		curl_easy.http_headers (
			curl_headers));

	// perform request

	let mut response_buffer: Vec <u8> =
		vec! [];

	let start_time =
		time::Instant::now ();

	{

		let mut curl_transfer =
			curl_easy.transfer ();

		try! (
			curl_transfer.write_function (
				|data| {

			response_buffer.extend_from_slice (
				data);

			Ok (data.len ())

		}));

		try! (
			curl_transfer.perform ());

	}

	let end_time =
		time::Instant::now ();

	let duration =
		end_time.duration_since (
			start_time);

	// process response

	let response_status_code =
		try! (
			curl_easy.response_code ()
		) as u64;

	let response_body =
		try! (
			String::from_utf8 (
				response_buffer));

	Ok (
		PerformRequestResult::Success (

		HttpResponse {
			status_code: response_status_code,
			body: response_body,
			duration: duration,
		}

	))

}

