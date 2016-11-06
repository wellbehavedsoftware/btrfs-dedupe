use std::collections::HashMap;
use std::path::PathBuf;

pub struct Arguments {
	pub match_filename: bool,
	pub root_paths: Vec <PathBuf>,
}

#[ derive (Eq, Hash, PartialEq) ]
pub struct FileMetadata {
	pub filename: Option <PathBuf>,
	pub size: u64,
}

pub type FileMetadataLists =
	HashMap <FileMetadata, Vec <PathBuf>>;

pub const HASH_SIZE: usize = 32;

pub type Hash = [u8; HASH_SIZE];

pub type HashLists =
	HashMap <Hash, Vec <PathBuf>>;

// ex: noet ts=4 filetype=rust
