extern crate libc;

use std::error;
use std::ffi;
use std::mem;

// ---------- public interface

pub struct BtrfsSpaceInfo {
	total_bytes: u64,
	used_bytes: u64,
}

// ---------- c ffi structs

#[ repr (C) ]
pub struct BtrfsIoctlSpaceArgs {
	space_slots: u64,
	total_spaces: u64,
}

#[ repr (C) ]
pub struct BtrfsIoctlSpaceInfo {
	flags: u64,
	total_bytes: u64,
	used_bytes: u64,
}

ioctl! (
	readwrite btrfs_ioc_space_info
	with 0x94, 20; BtrfsIoctlSpaceArgs);

// ---------- c ffi wrapper

pub fn get_space_info (
	path: & str,
) -> Result <Vec <BtrfsSpaceInfo>, Box <error::Error>> {

	// open directory fd

	let path_c =
		try! (
			ffi::CString::new (
				path.to_owned ()));

	let dir_fd =
		unsafe {
			libc::open (
				path_c.as_ptr (),
				libc::O_DIRECTORY)
		};

	if dir_fd < 0 {

		return Err (
			Box::from (
				format! (
					"error opening directory: {}",
					path,
				)));

	}

	println! (
		"open: {:?}",
		dir_fd );

	// get temporary info

	let mut space_args_temp =
		BtrfsIoctlSpaceArgs {
			space_slots: 0,
			total_spaces: 0,
		};

	let get_space_args_temp_result =
		unsafe {
			btrfs_ioc_space_info (
				dir_fd,
				& mut space_args_temp as * mut BtrfsIoctlSpaceArgs)
		};

	if get_space_args_temp_result != 0 {

		return Err (
			Box::from (
				format! (
					"error getting btrfs space information: {}",
					path)));

	}

	println! (
		"btrfs_ioc_space_info: {}, {}",
		space_args_temp.space_slots,
		space_args_temp.total_spaces);

	// allocate buffer for real info

	let space_args_real_buffer_size =
		mem::size_of::<BtrfsIoctlSpaceArgs> ()
		+ space_args_temp.total_spaces as usize
			* mem::size_of::<BtrfsIoctlSpaceInfo> ();

	let space_args_real_buffer: * mut libc::c_void =
		unsafe {
			libc::malloc (
				space_args_real_buffer_size)
		};

	let space_args_real: & mut BtrfsIoctlSpaceArgs =
		unsafe {
			mem::transmute (
				space_args_real_buffer)
		};

	// get real info

	space_args_real.space_slots =
		space_args_temp.total_spaces;

	let get_space_args_real_result =
		unsafe {
			btrfs_ioc_space_info (
				dir_fd,
				space_args_real as * mut BtrfsIoctlSpaceArgs)
		};

	if get_space_args_real_result != 0 {

		return Err (
			Box::from (
				format! (
					"error getting btrfs space information: {}",
					path)));

	}

	println! (
		"btrfs_ioc_space_info: {}, {}",
		space_args_real.space_slots,
		space_args_real.total_spaces);

	for index in 0 .. space_args_real.space_slots {

		let space_info: & BtrfsIoctlSpaceInfo =
			unsafe {
				mem::transmute (
					space_args_real_buffer.offset (
						(
							mem::size_of::<BtrfsIoctlSpaceArgs> ()
							+ index as usize
								* mem::size_of::<BtrfsIoctlSpaceInfo> ()
						) as isize))
			};

		println! (
			"space slot: {}, {}, {}",
			space_info.flags,
			space_info.total_bytes,
			space_info.used_bytes);

	}

	// create return value

	let mut space_infos: Vec <BtrfsSpaceInfo> =
		vec! [];

	for index in 0 .. space_args_real.space_slots {

		let space_info: & BtrfsIoctlSpaceInfo =
			unsafe {
				mem::transmute (
					space_args_real_buffer.offset (
						(
							mem::size_of::<BtrfsIoctlSpaceArgs> ()
							+ index as usize
								* mem::size_of::<BtrfsIoctlSpaceInfo> ()
						) as isize))
			};

		space_infos.push (
			BtrfsSpaceInfo {
				used_bytes: space_info.used_bytes,
				total_bytes: space_info.total_bytes,
			}
		);

	}

	// free buffer and close file

	unsafe {

		libc::free (
			space_args_real_buffer);

		libc::close (
			dir_fd);

	}

	Ok (space_infos)

}
