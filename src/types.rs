use std::path::PathBuf;
use std::rc::Rc;

pub type PathRef = Rc <PathBuf>;

pub const HASH_SIZE: usize = 32;

pub type Hash = [u8; HASH_SIZE];

pub const ZERO_HASH: Hash = [0u8; HASH_SIZE];

#[ derive (Eq, Hash, PartialEq) ]
pub struct CompareFileMetadata {
	pub filename: Option <PathBuf>,
	pub size: u64,
}

// ex: noet ts=4 filetype=rust
