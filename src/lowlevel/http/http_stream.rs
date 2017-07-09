use std::error::Error;
use std::io::Error as IoError;
use std::io::ErrorKind as IoErrorKind;
use std::io::Read;
use std::io::Result as IoResult;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;

use futures::Async as FuturesAsync;
use futures::Future;
use futures::IntoFuture;
use futures::Poll as FuturesPoll;
use futures::Stream;
use futures::future;
use futures::future::Either as FutureEither;
use futures::future::FutureResult;

use hyper::Client as HyperClient;
use hyper::Method as HyperMethod;
use hyper::Uri as HyperUri;
use hyper::client::HttpConnector as HyperHttpConnector;
use hyper::client::Request as HyperRequest;

use rustls::Certificate as RustTlsCertificate;
use rustls::ClientConfig as RustTlsClientConfig;
use rustls::ClientSession as RustTlsClientSession;
use rustls::Session as RustTlsSession;

use tokio_core::net::TcpStream as TokioTcpStream;
use tokio_io::AsyncRead;
use tokio_io::AsyncWrite;
use tokio_rustls::TlsStream as TokioRustTlsStream;

pub enum HttpStream {
	Http (TokioTcpStream),
	Https (TokioRustTlsStream <TokioTcpStream, RustTlsClientSession>),
}

#[ derive (Clone) ]
pub struct HttpSharedStream {
	http_stream: Arc <Mutex <Option <HttpStream>>>,
}

pub struct HttpBorrowedStream {
	owned_stream: Option <HttpStream>,
	shared_stream: Arc <Mutex <Option <HttpStream>>>,
}

impl HttpSharedStream {

	pub fn new (
		http_stream: HttpStream,
	) -> HttpSharedStream {

println! ("CREATE");

		HttpSharedStream {

			http_stream:
				Arc::new (Mutex::new (
					Some (http_stream))),

		}

	}

	pub fn borrow (
		& self,
	) -> HttpBorrowedStream {

		let mut http_stream_lock =
			self.http_stream.lock ().unwrap ();

		let http_stream =
			http_stream_lock.take ().unwrap ();

println! ("BORROW");

		HttpBorrowedStream {

			owned_stream:
				Some (http_stream),

			shared_stream:
				self.http_stream.clone (),

		}

	}

}

impl Read for HttpBorrowedStream {

	#[inline]
	fn read (
		& mut self,
		buffer: & mut [u8],
	) -> IoResult <usize> {

		match self.owned_stream {

			Some (HttpStream::Http (ref mut stream)) =>
				stream.read (buffer),

			Some (HttpStream::Https (ref mut stream)) =>
				stream.read (buffer),

			None =>
				panic! (),

		}

	}

}

impl Write for HttpBorrowedStream {

	#[inline]
	fn write (
		& mut self,
		buffer: & [u8],
	) -> IoResult <usize> {

		match self.owned_stream {

	        Some (HttpStream::Http (ref mut stream)) =>
	        	stream.write (buffer),

	        Some (HttpStream::Https (ref mut stream)) =>
	        	stream.write (buffer),

			None =>
				panic! (),

	    }

	}

	#[inline]
	fn flush (
		& mut self,
	) -> IoResult <()> {

		match self.owned_stream {

			Some (HttpStream::Http (ref mut stream)) =>
				stream.flush (),

			Some (HttpStream::Https (ref mut stream)) =>
				stream.flush (),

			None =>
				panic! (),

		}

	}

}

impl AsyncRead for HttpBorrowedStream {

	unsafe fn prepare_uninitialized_buffer (
		& self,
		buf: & mut [u8],
	) -> bool {

		match self.owned_stream {

			Some (HttpStream::Http (ref stream)) =>
				stream.prepare_uninitialized_buffer (buf),

			Some (HttpStream::Https (ref stream)) =>
				stream.prepare_uninitialized_buffer (buf),

			None =>
				panic! (),

		}

	}

}

impl AsyncWrite for HttpBorrowedStream {

	fn shutdown (
		& mut self,
	) -> FuturesPoll <(), IoError> {

		let owned_stream =
			self.owned_stream.take ().unwrap ();

		let mut shared_stream_lock =
			self.shared_stream.lock ().unwrap ();

		assert! (shared_stream_lock.is_none ());

		* shared_stream_lock =
			Some (owned_stream);

println! ("SHARE");

		Ok (FuturesAsync::Ready (()))

	}

}

// ex: noet ts=4 filetype=rust
