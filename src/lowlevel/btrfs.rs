extern crate libc;

use std::error;
use std::ffi;
use std::iter;
use std::iter::FromIterator;
use std::mem;
use std::slice;

// ---------- public interface

pub struct BtrfsSpaceInfo {
	total_bytes: u64,
	used_bytes: u64,
}

// ---------- c ffi structs

#[ repr (C) ]
#[ derive (Copy, Clone) ]
pub struct BtrfsIoctlSpaceArgs {
	space_slots: u64,
	total_spaces: u64,
}

#[ repr (C) ]
#[ derive (Copy, Clone) ]
pub struct BtrfsIoctlSpaceInfo {
	flags: u64,
	total_bytes: u64,
	used_bytes: u64,
}

ioctl! (
	readwrite btrfs_ioc_space_info
	with 0x94, 20; BtrfsIoctlSpaceArgs);

// ---------- file descriptor with destructor

struct FileDescriptor {
	value: libc::c_int,
}

impl FileDescriptor {

	fn open (
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

// ---------- high level wrapper

pub fn get_space_info (
	path: & str,
) -> Result <Vec <BtrfsSpaceInfo>, Box <error::Error>> {

	// open directory

	let file_descriptor =
		try! (
			FileDescriptor::open (
				path,
				libc::O_DIRECTORY));

	let mut num_spaces = 0;
	let mut c_space_info;

	loop {

		c_space_info =
			try! (
				get_c_space_info (
					file_descriptor.value,
					num_spaces));

		if c_space_info.args.total_spaces
			<= c_space_info.args.space_slots {

			break;

		}

		num_spaces =
			c_space_info.args.total_spaces;

	}

	for c_space_info in c_space_info.infos.iter () {

		println! (
			"space slot: {}, {}, {}",
			c_space_info.flags,
			c_space_info.total_bytes,
			c_space_info.used_bytes);

	}

	// create return value

	let mut space_infos: Vec <BtrfsSpaceInfo> =
		vec! [];

	for c_space_info in c_space_info.infos.iter () {

		space_infos.push (
			BtrfsSpaceInfo {
				used_bytes: c_space_info.used_bytes,
				total_bytes: c_space_info.total_bytes,
			}
		);

	}

	Ok (space_infos)

}

// low level wrapper

struct CSpaceInfoResult {
	args: BtrfsIoctlSpaceArgs,
	infos: Vec <BtrfsIoctlSpaceInfo>,
}

fn get_c_space_info (
	file_descriptor: libc::c_int,
	num_spaces: u64,
) -> Result <CSpaceInfoResult, Box <error::Error>> {

	// allocate buffer

	let c_space_buffer_size =
		mem::size_of::<BtrfsIoctlSpaceArgs> ()
		+ num_spaces as usize
			* mem::size_of::<BtrfsIoctlSpaceInfo> ();

	let mut c_space_buffer: Vec <u8> =
		Vec::from_iter (
			iter::repeat (0u8).take (
				c_space_buffer_size));

	let (c_space_args_buffer, c_space_infos_buffer) =
		c_space_buffer.split_at_mut (
			mem::size_of::<BtrfsIoctlSpaceArgs> ());

	// split buffer

	let c_space_args_slice: & mut [BtrfsIoctlSpaceArgs] =
		unsafe {
			slice::from_raw_parts_mut (
				mem::transmute (
					c_space_args_buffer.as_mut_ptr ()),
				1)
		};

	let c_space_args =
		& mut c_space_args_slice [0];

	let c_space_infos: & mut [BtrfsIoctlSpaceInfo] =
		unsafe {
			slice::from_raw_parts_mut (
				mem::transmute (
					c_space_infos_buffer.as_mut_ptr ()),
				num_spaces as usize)
		};

	// get info

	c_space_args.space_slots =
		num_spaces;

	let get_space_args_real_result =
		unsafe {
			btrfs_ioc_space_info (
				file_descriptor,
				c_space_args as * mut BtrfsIoctlSpaceArgs)
		};

	if get_space_args_real_result != 0 {

		return Err (
			Box::from (
				"error getting btrfs space information")
		);

	}

	// return

	Ok (
		CSpaceInfoResult {
			args: * c_space_args,
			infos: c_space_infos.to_vec (),
		}
	)

}
