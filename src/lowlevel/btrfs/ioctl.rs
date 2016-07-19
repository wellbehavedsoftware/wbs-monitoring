use lowlevel::btrfs::ctypes;

ioctl! (
	readwrite space_info
	with 0x94, 20; ctypes::IoctlSpaceArgs);

ioctl! (
	read fs_info
	with 0x94, 31; ctypes::IoctlFsInfoArgs);

// ex: noet ts=4 filetype=rust
