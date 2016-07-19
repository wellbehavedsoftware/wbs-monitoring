extern crate libc;

use std::error;
use std::ffi;

// ---------- file descriptor with destructor

pub struct FileDescriptor {
	value: libc::c_int,
}

impl FileDescriptor {

	pub fn open (
		path: & str,
		flags: libc::c_int,
	) -> Result <FileDescriptor, Box <error::Error>> {

		let path_c =
			try! (
				ffi::CString::new (
					path.to_owned ()));

		let fd =
			unsafe {
				libc::open (
					path_c.as_ptr (),
					flags)
			};

		if fd >= 0 {

			Ok (
				FileDescriptor {
					value: fd,
				}
			)

		} else {

			Err (
				Box::from (
					format! (
						"error opening {}",
						path,
					))
			)

		}

	}

	pub fn get_value (
		& self,
	) -> libc::c_int {
		self.value
	}

}

impl Drop for FileDescriptor {

	fn drop (
		& mut self,
	) {

		unsafe {

			libc::close (
				self.value);

		}

	}

}

impl <'a> From <& 'a FileDescriptor> for libc::c_int {

	fn from (
		file_descriptor: & 'a FileDescriptor,
	) -> libc::c_int {

		file_descriptor.value

	}

}

// ex: noet ts=4 filetype=rust
