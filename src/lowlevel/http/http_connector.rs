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

use futures::Async;
use futures::Future;
use futures::Poll;
use futures::sync::oneshot;

use rustls::Certificate as RustTlsCertificate;
use rustls::ClientConfig as RustTlsClientConfig;
use rustls::ClientSession as RustTlsClientSession;
use rustls::Session as RustTlsSession;

use tokio_core::net::TcpStream as TokioTcpStream;
use tokio_core::reactor::Handle as TokioHandle;
use tokio_io::AsyncRead;
use tokio_io::AsyncWrite;
use tokio_rustls::ClientConfigExt;
use tokio_rustls::TlsStream as TokioRustTlsStream;
use tokio_service::Service;

use webpki_roots;

pub struct HttpsConnecting {
	future: Box <Future <
		Item = MaybeHttpsStream,
		Error = IoError,
	>>,
	peer_certificates: Arc <Mutex <Option <Vec <RustTlsCertificate>>>>,
}

impl Future for HttpsConnecting {

	type Item = MaybeHttpsStream;
	type Error = IoError;

	fn poll (
		& mut self,
	) -> Poll <Self::Item, Self::Error> {

		match self.future.poll () {

			Ok (Async::Ready (maybe_stream)) => {

				if let MaybeHttpsStream::Https (ref tls_stream) =
					maybe_stream {

					let mut peer_certificates =
						self.peer_certificates.lock ().unwrap ();

					let (_tls_io, tls_session) =
						tls_stream.get_ref ();

					* peer_certificates =
						tls_session.get_peer_certificates ();

				}

				Ok (Async::Ready (maybe_stream))

			},

			other =>
				other,

		}

	}

}

impl fmt::Debug for HttpsConnecting {

	fn fmt (
		& self,
		formatter: & mut fmt::Formatter,
	) -> fmt::Result {

		formatter.pad (
			"HttpsConnecting",
		)

	}

}

pub enum MaybeHttpsStream {
	Http (TokioTcpStream),
	Https (TokioRustTlsStream <TokioTcpStream, ClientSession>),
}

impl fmt::Debug for MaybeHttpsStream {

	fn fmt (
		& self,
		formatter: & mut fmt::Formatter,
	) -> fmt::Result {

		match * self {

			MaybeHttpsStream::Http (..) =>
				formatter.pad ("Http (..)"),

			MaybeHttpsStream::Https (..) =>
				formatter.pad ("Https (..)"),

		}

	}

}

impl Read for MaybeHttpsStream {

	#[inline]
	fn read (
		& mut self,
		buffer: & mut [u8],
	) -> IoResult <usize> {

		match * self {

			MaybeHttpsStream::Http (ref mut stream) =>
				stream.read (buffer),

			MaybeHttpsStream::Https (ref mut stream) =>
				stream.read (buffer),

		}

	}

}

impl Write for MaybeHttpsStream {

	#[inline]
	fn write (
		& mut self,
		buffer: & [u8],
	) -> IoResult <usize> {

	    match * self {

	        MaybeHttpsStream::Http (ref mut stream) =>
	        	stream.write (buffer),

	        MaybeHttpsStream::Https (ref mut stream) =>
	        	stream.write (buffer),

	    }

	}

	#[inline]
	fn flush (
		& mut self,
	) -> IoResult <()> {

		match * self {

			MaybeHttpsStream::Http (ref mut stream) =>
				stream.flush (),

			MaybeHttpsStream::Https (ref mut stream) =>
				stream.flush (),

		}

	}

}

impl AsyncRead for MaybeHttpsStream {

	unsafe fn prepare_uninitialized_buffer (
		& self,
		buf: & mut [u8],
	) -> bool {

		match * self {

			MaybeHttpsStream::Http (ref stream) =>
				stream.prepare_uninitialized_buffer (buf),

			MaybeHttpsStream::Https (ref stream) =>
				stream.prepare_uninitialized_buffer (buf),

		}

	}

}

impl AsyncWrite for MaybeHttpsStream {

	fn shutdown (
		& mut self,
	) -> Poll <(), IoError> {

		match * self {

			MaybeHttpsStream::Http (ref mut stream) =>
				stream.shutdown (),

			MaybeHttpsStream::Https (ref mut stream) =>
				stream.shutdown (),

		}

	}

}

// ex: noet ts=4 filetype=rust
