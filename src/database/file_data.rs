use database::*;
use types::*;

#[ derive (Debug, Eq, Hash, PartialEq) ]
pub struct FileData {

	pub path: RecursivePathRef,
	pub root_path: Option <PathRef>,

    pub size: u64,

	pub content_hash: Hash,
	pub content_hash_time: i64,

	pub extent_hash: Hash,
	pub extent_hash_time: i64,

	pub defragment_time: i64,
	pub deduplicate_time: i64,

	pub mtime: i64,
	pub ctime: i64,

	pub mode: u32,
    pub uid: u32,
    pub gid: u32,

}

// ex: noet ts=4 filetype=rust
