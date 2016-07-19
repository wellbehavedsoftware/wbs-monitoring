pub const AVAIL_ALLOC_BIT_SINGLE: u64 = 1 << 48;

pub const BLOCK_GROUP_DATA: u64 = 1 << 0;
pub const BLOCK_GROUP_SYSTEM: u64 = 1 << 1;
pub const BLOCK_GROUP_METADATA: u64 = 1 << 2;

pub const BLOCK_GROUP_RAID0: u64 = 1 << 3;
pub const BLOCK_GROUP_RAID1: u64 = 1 << 4;
pub const BLOCK_GROUP_DUP: u64 = 1 << 5;
pub const BLOCK_GROUP_RAID10: u64 = 1 << 6;
pub const BLOCK_GROUP_RAID5: u64 = 1 << 7;
pub const BLOCK_GROUP_RAID6: u64 = 1 << 8;

pub const BLOCK_GROUP_RESERVED: u64 = AVAIL_ALLOC_BIT_SINGLE;

pub const BLOCK_GROUP_DATA_AND_METADATA: u64 = (
	BLOCK_GROUP_DATA
	| BLOCK_GROUP_METADATA
);

pub const BLOCK_GROUP_TYPE_MASK: u64 = (
	BLOCK_GROUP_DATA
	| BLOCK_GROUP_SYSTEM
	| BLOCK_GROUP_METADATA
);

pub const BLOCK_GROUP_TYPE_AND_RESERVED_MASK: u64 = (
	BLOCK_GROUP_TYPE_MASK
	| BLOCK_GROUP_RESERVED
);

pub const BLOCK_GROUP_PROFILE_MASK: u64 = (
	BLOCK_GROUP_RAID0
	| BLOCK_GROUP_RAID1
	| BLOCK_GROUP_RAID5
	| BLOCK_GROUP_RAID6
	| BLOCK_GROUP_DUP
	| BLOCK_GROUP_RAID10
);

#[ repr (C) ]
#[ derive (Copy, Clone) ]
pub struct IoctlSpaceArgs {
	pub space_slots: u64,
	pub total_spaces: u64,
}

#[ repr (C) ]
#[ derive (Copy, Clone) ]
pub struct IoctlSpaceInfo {
	pub flags: u64,
	pub total_bytes: u64,
	pub used_bytes: u64,
}

#[ repr (C) ]
#[ derive (Copy, Clone) ]
pub struct IoctlFsInfoArgs {
	pub max_id: u64,
	pub num_devices: u64,
	pub filesystem_id: [u8; 16],
	pub reserved0: [u64; 32],
	pub reserved1: [u64; 32],
	pub reserved2: [u64; 32],
	pub reserved3: [u64; 28],
}

// ex: noet ts=4 filetype=rust
