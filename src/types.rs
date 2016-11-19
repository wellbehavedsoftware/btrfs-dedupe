use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

pub type PathRef = Rc <PathBuf>;

pub const HASH_SIZE: usize = 32;

pub type Hash = [u8; HASH_SIZE];

pub const ZERO_HASH: Hash = [0u8; HASH_SIZE];

pub type HashLists =
	HashMap <Hash, Vec <PathBuf>>;

#[ derive (Eq, Hash, PartialEq) ]
pub struct CompareFileMetadata {
	pub filename: Option <PathBuf>,
	pub size: u64,
}

pub type CompareFileMetadataLists =
	HashMap <CompareFileMetadata, Vec <PathBuf>>;

// ex: noet ts=4 filetype=rust
