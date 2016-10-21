use std::collections::HashMap;
use std::path::PathBuf;

#[ derive (Eq, Hash, PartialEq) ]
pub struct FilenameAndSize {
	pub filename: PathBuf,
	pub size: u64,
}

pub type FilenameAndSizeLists =
	HashMap <FilenameAndSize, Vec <PathBuf>>;

#[ derive (Eq, Hash, PartialEq) ]
pub struct FilenameAndChecksum {
	pub filename: PathBuf,
	pub checksum: u64,
}

pub type FilenameAndChecksumLists =
	HashMap <FilenameAndChecksum, Vec <PathBuf>>;

// ex: noet ts=4 filetype=rust
