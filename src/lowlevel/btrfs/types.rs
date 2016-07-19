use lowlevel::btrfs::ctypes;

#[ derive (Debug, Eq, PartialEq) ]
pub enum GroupType {
	Data,
	System,
	MetaData,
	DataAndMetaData,
	GlobalReserve,
	Unknown,
}

#[ derive (Debug, Eq, PartialEq) ]
pub enum GroupProfile {
	Single,
	Raid0,
	Raid1,
	Raid5,
	Raid6,
	Dup,
	Raid10,
	Unknown,
}

#[ derive (Debug, Eq, PartialEq) ]
pub struct SpaceInfo {
	pub group_type: GroupType,
	pub group_profile: GroupProfile,
	pub total_bytes: u64,
	pub used_bytes: u64,
}

impl From <u64> for GroupType {

	fn from (
		flags: u64,
	) -> GroupType {

		match flags & ctypes::BLOCK_GROUP_TYPE_AND_RESERVED_MASK {

			ctypes::BLOCK_GROUP_DATA =>
				GroupType::Data,

			ctypes::BLOCK_GROUP_SYSTEM =>
				GroupType::System,

			ctypes::BLOCK_GROUP_METADATA =>
				GroupType::MetaData,

			ctypes::BLOCK_GROUP_DATA_AND_METADATA =>
				GroupType::DataAndMetaData,

			ctypes::BLOCK_GROUP_RESERVED =>
				GroupType::GlobalReserve,

			_ =>
				GroupType::Unknown,

		}

	}

}

impl From <u64> for GroupProfile {

	fn from (
		flags: u64,
	) -> GroupProfile {

		match flags & ctypes::BLOCK_GROUP_PROFILE_MASK {

			0 =>
				GroupProfile::Single,

			ctypes::BLOCK_GROUP_RAID0 =>
				GroupProfile::Raid0,

			ctypes::BLOCK_GROUP_RAID1 =>
				GroupProfile::Raid1,

			ctypes::BLOCK_GROUP_RAID5 =>
				GroupProfile::Raid5,

			ctypes::BLOCK_GROUP_RAID6 =>
				GroupProfile::Raid6,

			ctypes::BLOCK_GROUP_DUP =>
				GroupProfile::Dup,

			ctypes::BLOCK_GROUP_RAID10 =>
				GroupProfile::Raid10,

			_ =>
				GroupProfile::Unknown,

		}

	}

}

// ex: noet ts=4 filetype=rust
