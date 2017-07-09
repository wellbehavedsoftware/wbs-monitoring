use std::error::Error;
use std::io::Error as IoError;
use std::io::ErrorKind as IoErrorKind;
use std::io::Read;
use std::io::Result as IoResult;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

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
use hyper::header::ContentType as HyperContentTypeHeader;
use hyper::header::Host as HyperHostHeader;

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

pub enum HttpStream {
	Http (TokioTcpStream),
	Https (TokioRustTlsStream <TokioTcpStream, RustTlsClientSession>),
}

impl Read for HttpStream {

	#[inline]
	fn read (
		& mut self,
		buffer: & mut [u8],
	) -> IoResult <usize> {

		match * self {

			HttpStream::Http (ref mut stream) =>
				stream.read (buffer),

			HttpStream::Https (ref mut stream) =>
				stream.read (buffer),

		}

	}

}

impl Write for HttpStream {

	#[inline]
	fn write (
		& mut self,
		buffer: & [u8],
	) -> IoResult <usize> {

	    match * self {

	        HttpStream::Http (ref mut stream) =>
	        	stream.write (buffer),

	        HttpStream::Https (ref mut stream) =>
	        	stream.write (buffer),

	    }

	}

	#[inline]
	fn flush (
		& mut self,
	) -> IoResult <()> {

		match * self {

			HttpStream::Http (ref mut stream) =>
				stream.flush (),

			HttpStream::Https (ref mut stream) =>
				stream.flush (),

		}

	}

}

impl AsyncRead for HttpStream {

	unsafe fn prepare_uninitialized_buffer (
		& self,
		buf: & mut [u8],
	) -> bool {

		match * self {

			HttpStream::Http (ref stream) =>
				stream.prepare_uninitialized_buffer (buf),

			HttpStream::Https (ref stream) =>
				stream.prepare_uninitialized_buffer (buf),

		}

	}

}

impl AsyncWrite for HttpStream {

	fn shutdown (
		& mut self,
	) -> FuturesPoll <(), IoError> {

		match * self {

			HttpStream::Http (ref mut stream) =>
				stream.shutdown (),

			HttpStream::Https (ref mut stream) =>
				stream.shutdown (),

		}

	}

}

// ex: noet ts=4 filetype=rust
