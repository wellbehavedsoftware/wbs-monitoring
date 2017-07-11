use super::http_prelude::*;

pub struct HttpConnection {

	address: String,
	hostname: String,
	port: u64,
	secure: bool,

	tokio_core: TokioCore,
	hyper_client: HyperClient <HttpConnector>,

	connect_duration: Duration,

	peer_certificates: Option <Vec <RustTlsCertificate>>,
	certificate_expiry: Option <NaiveDateTime>,

}

impl HttpConnection {

	pub fn connect (
		address: String,
		port: Option <u64>,
		secure: bool,
		hostname: String,
	) -> HttpResult <HttpConnection> {

		let port =
			port.unwrap_or (
				if secure { 443 } else { 80 });

		// setup tokio

		let mut tokio_core =
			TokioCore::new ().map_err (
				|error|

			HttpError::Unknown (
				Box::new (error))

		) ?;

		// setup stream

		let hyper_uri: HyperUri =
			format! (
				"{}://{}:{}/",
				if secure { "http" } else { "http" },
				address,
				port,
			).parse ().map_err (
				|_| HttpError::InvalidUri,
			) ?;

		let mut hyper_connector =
			HyperHttpConnector::new (
				1,
				& tokio_core.handle ());

		hyper_connector.enforce_http (false);

		let start_time = Instant::now ();

		let (http_stream, peer_certificates) =
			tokio_core.run ({

			let hostname = hostname.to_string ();

			hyper_connector.call (
				hyper_uri,
			).and_then (
				move |tcp_stream| {

				if secure {

					let mut rust_tls_client_config =
						RustTlsClientConfig::new ();

					rust_tls_client_config.root_store.add_trust_anchors (
						& webpki_roots::ROOTS,
					);

					Arc::new (
						rust_tls_client_config,
					).connect_async (
						& hostname,
						tcp_stream,
					).map_err (
						|error|

						IoError::new (
							IoErrorKind::Other,
							error,
						)

					).map (
						|rust_tls_stream| {

						let peer_certificates = {

							let (_tls_io, tls_session) =
								rust_tls_stream.get_ref ();

							tls_session.get_peer_certificates ()

						};

						(
							HttpStream::Https (
								rust_tls_stream),
							peer_certificates,
						)

					}).map_err (
						|error|

						IoError::new (
							IoErrorKind::Other,
							error,
						)

					).boxed ()

				} else {

					future::ok ((
						HttpStream::Http (
							tcp_stream),
						None,
					)).boxed ()

				}

			})

		}).map_err (
			|error|

			HttpError::Unknown (
				Box::new (error)),

		) ?;

		let http_shared_stream =
			HttpSharedStream::new (
				http_stream);

		let end_time = Instant::now ();

		// create client

		let connector =
			HttpConnector::new (
				http_shared_stream);

		let hyper_client =
			HyperClient::configure (
			).connector (
				connector,
			).build (
				& tokio_core.handle (),
			);

		// create conection

		let certificate_expiry =
			get_certificate_validity (
				& peer_certificates,
			).map (
				|(_start, end)| end,
			);

		Ok (HttpConnection {

			address: address,
			hostname: hostname,
			port: port,
			secure: secure,

			tokio_core: tokio_core,
			hyper_client: hyper_client,

			connect_duration: end_time - start_time,

			peer_certificates: peer_certificates,
			certificate_expiry: certificate_expiry,

		})

	}

	pub fn perform (
		& mut self,
		request: HttpRequest,
		timeout: Duration,
	) -> HttpResult <HttpResponse> {

		// create uri

		let hyper_uri = if self.port == self.default_port () {

			format! (
				"{}://{}:{}{}",
				if self.secure { "https" } else { "http" },
				self.address,
				self.port,
				request.path)

		} else {

			format! (
				"{}://{}{}",
				if self.secure { "https" } else { "http" },
				self.address,
				request.path)

		}.parse ().map_err (|_| HttpError::InvalidUri) ?;

		// create request

		let mut hyper_request =
			HyperRequest::new (
				HyperMethod::Get,
				hyper_uri);

		{

			let hyper_headers =
				hyper_request.headers_mut ();

			let mut got_host = false;

			for & (ref header_name, ref header_value)
				in request.headers.iter () {

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

				if self.port == self.default_port () {

					hyper_headers.set (
						HyperHostHeader::new (
							self.hostname.to_string (),
							None));

				} else {

					hyper_headers.set (
						HyperHostHeader::new (
							self.hostname.to_string (),
							self.port as u16));

				}

			}

		}

		// perform request

		let request_start_time =
			Instant::now ();

		let timeout_time =
			request_start_time + timeout;

		let timeout =
			TokioTimeout::new_at (
				timeout_time,
				& self.tokio_core.handle (),
			).into_future ().flatten ();

		let hyper_response =
			match self.tokio_core.run (

			self.hyper_client.request (
				hyper_request,
			).select2 (timeout)

		) {

			Ok (FutureEither::A ((hyper_response, _))) =>
				hyper_response,

			Err (FutureEither::A ((hyper_error, _))) =>
				return Err (HttpError::Unknown (
					Box::new (hyper_error))),

			_ =>
				return Err (HttpError::Timeout),

		};

		let request_end_time =
			Instant::now ();

		let request_duration =
			request_end_time - request_start_time;

		// process response

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

		let response_start_time =
			Instant::now ();

		let mut response_body: Vec <u8> =
			Vec::new ();

		let timeout =
			TokioTimeout::new_at (
				timeout_time,
				& self.tokio_core.handle (),
			).into_future ().flatten ();

		match self.tokio_core.run (

			hyper_response.body ().for_each (
				|chunk| {

				response_body.extend_from_slice (
					& chunk);

				Ok (())

			}).select2 (timeout)

		) {

			Ok (FutureEither::A (_)) =>
				(),

			Err (FutureEither::A ((hyper_error, _))) =>
				return Err (
					HttpError::Unknown (
						Box::new (hyper_error))),

			_ =>
				return Err (
					HttpError::Timeout),

		};

		let response_end_time =
			Instant::now ();

		let response_duration =
			response_end_time - response_start_time;

		// return

		Ok (HttpResponse {

			status_code: response_status_code,
			status_message: response_status_message,

			headers: response_headers,

			body: response_body,
			body_encoding: response_encoding,

			request_duration: request_duration,
			response_duration: response_duration,

		})

	}

	pub fn connect_duration (& self) -> Duration {
		self.connect_duration
	}

	pub fn peer_certificates (& self) -> & Option <Vec <RustTlsCertificate>> {
		& self.peer_certificates
	}

	pub fn certificate_expiry (& self) -> Option <NaiveDateTime> {
		self.certificate_expiry
	}

	fn default_port (
		& self,
	) -> u64 {

		if self.secure {
			443
		} else {
			80
		}

	}

}

// ex: noet ts=4 filetype=rust
