use std::io::Error as IoError;
use std::sync::Arc;
use std::sync::Mutex;

use hyper::Uri as HyperUri;

use futures::future;
use futures::future::FutureResult;

use tokio_service::Service as TokioService;

use super::*;

pub struct HttpConnector {
	http_stream: Arc <Mutex <HttpSharedStream>>,
}

impl HttpConnector {

	pub fn new (
		http_stream: HttpSharedStream,
	) -> HttpConnector {

		HttpConnector {

			http_stream:
				Arc::new (Mutex::new (
					http_stream)),

		}

	}

}

impl TokioService for HttpConnector {

	type Request = HyperUri;
	type Response = HttpBorrowedStream;
	type Error = IoError;
	type Future = FutureResult <HttpBorrowedStream, IoError>;

	fn call (
		& self,
		_uri: HyperUri,
	) -> Self::Future {

		let http_stream_lock =
			self.http_stream.lock ().unwrap ();

		future::ok (
			http_stream_lock.borrow ())

	}

}

// ex: noet ts=4 filetype=rust
