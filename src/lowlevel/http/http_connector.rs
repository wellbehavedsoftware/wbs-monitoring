use std::error::Error;
use std::fmt;
use std::io::Error as IoError;
use std::io::ErrorKind as IoErrorKind;
use std::io::Read;
use std::io::Result as IoResult;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;

use hyper::Uri as HyperUri;
use hyper::client::HttpConnector as HyperHttpConnector;

use futures::Future;
use futures::IntoFuture;
use futures::Poll as FuturesPoll;
use futures::Stream;
use futures::future;
use futures::future::Either as FutureEither;
use futures::future::FutureResult;

use rustls::Certificate as RustTlsCertificate;
use rustls::ClientConfig as RustTlsClientConfig;
use rustls::ClientSession as RustTlsClientSession;
use rustls::Session as RustTlsSession;

use tokio_core::net::TcpStream as TokioTcpStream;
use tokio_core::reactor::Core as TokioCore;
use tokio_core::reactor::Handle as TokioHandle;
use tokio_core::reactor::Timeout as TokioTimeout;
use tokio_io::AsyncRead;
use tokio_io::AsyncWrite;
use tokio_rustls::ClientConfigExt;
use tokio_rustls::TlsStream as TokioRustTlsStream;
use tokio_service::Service as TokioService;

use webpki_roots;

use super::*;

pub struct HttpConnector {
	http_stream: HttpSharedStream,
}

impl HttpConnector {

	pub fn new (
		http_stream: HttpSharedStream,
	) -> HttpConnector {

		HttpConnector {
			http_stream: http_stream,
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
		uri: HyperUri,
	) -> Self::Future {

		future::ok (
			self.http_stream.borrow ())

	}

}

// ex: noet ts=4 filetype=rust
