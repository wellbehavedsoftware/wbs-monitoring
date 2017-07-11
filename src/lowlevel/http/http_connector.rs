use super::http_prelude::*;

pub struct HttpConnector {
	http_shared_stream: HttpSharedStream,
}

impl HttpConnector {

	pub fn new (
		http_shared_stream: HttpSharedStream,
	) -> HttpConnector {

		HttpConnector {
			http_shared_stream: http_shared_stream,
		}

	}

}

impl TokioService for HttpConnector {

	type Request = HyperUri;
	type Response = HttpBorrowedStream;
	type Error = IoError;
	type Future = BoxFuture <HttpBorrowedStream, IoError>;

	fn call (
		& self,
		_uri: HyperUri,
	) -> Self::Future {

println! ("CONNECT");

		self.http_shared_stream.borrow ().map_err (
			|canceled|

			IoError::new (
				IoErrorKind::Other,
				canceled,
			)

		).boxed ()

	}

}

// ex: noet ts=4 filetype=rust
