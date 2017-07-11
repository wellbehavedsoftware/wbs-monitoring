use super::http_prelude::*;

pub enum HttpStream {
	Http (TokioTcpStream),
	Https (TokioRustTlsStream <TokioTcpStream, RustTlsClientSession>),
}

#[ derive (Clone) ]
pub struct HttpSharedStream {
	internal: Arc <Mutex <HttpSharedStreamInternal>>,
}

pub struct HttpSharedStreamInternal {
	http_stream: Option <HttpStream>,
	senders: LinkedList <OneshotSender <HttpStream>>,
}

pub struct HttpBorrowedStream {
	shared_stream: HttpSharedStream,
	owned_stream: Option <HttpStream>,
}

impl HttpSharedStream {

	pub fn new (
		http_stream: HttpStream,
	) -> HttpSharedStream {

		HttpSharedStream {

			internal: Arc::new (Mutex::new (
				HttpSharedStreamInternal {

				http_stream: Some (http_stream),
				senders: LinkedList::new (),

			})),

		}

	}

	pub fn borrow (
		& self,
	) -> BoxFuture <HttpBorrowedStream, FuturesCanceled> {

		let mut internal =
			self.internal.lock ().unwrap ();

		if let Some (http_stream) =
			internal.http_stream.take () {

			future_ok (
				HttpBorrowedStream {

					shared_stream:
						self.clone (),

					owned_stream:
						Some (http_stream),

				}
			).boxed ()

		} else {

			let (sender, receiver) =
				oneshot_channel ();

			internal.senders.push_back (
				sender);

			let self_clone =
				self.clone ();

			receiver.map (
				move |http_stream| {

				HttpBorrowedStream {

					shared_stream:
						self_clone,

					owned_stream:
						Some (http_stream),

				}

			}).boxed ()

		}

	}

	fn unborrow (
		& self,
		http_stream: HttpStream,
	) {

		let mut http_stream =
			http_stream;

		loop {

			let mut internal =
				self.internal.lock ().unwrap ();

			if let Some (sender) =
				internal.senders.pop_front () {

				// send it to next in queue

				drop (internal);

				if let Err (returned_http_stream) =
					sender.send (
						http_stream) {

					// not accepted, loop

					http_stream =
						returned_http_stream;

				} else {

					// accepted, done

					break;

				}

			} else {

				// no waiters, reclaim ownership

				internal.http_stream =
					Some (http_stream);

				break;

			}

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

		// do nothing

		Ok (FuturesAsync::Ready (()))

	}

}

impl Drop for HttpBorrowedStream {

	fn drop (
		& mut self,
	) {

		let owned_stream =
			self.owned_stream.take ().unwrap ();

		self.shared_stream.unborrow (
			owned_stream);

	}

}

// ex: noet ts=4 filetype=rust
