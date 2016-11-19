use std::rc::Rc;
use std::path::PathBuf;

#[ derive (Debug, Deserialize, Serialize) ]
pub struct FileDataRecord {

	pub path: Rc <PathBuf>,

    pub size: u64,

    #[ serde (skip_serializing_if = "Option::is_none") ]
    pub content_hash: Option <String>,

    #[ serde (skip_serializing_if = "Option::is_none") ]
	pub content_hash_time: Option <i64>,

    #[ serde (skip_serializing_if = "Option::is_none") ]
	pub extent_hash: Option <String>,

    #[ serde (skip_serializing_if = "Option::is_none") ]
	pub extent_hash_time: Option <i64>,

	pub mtime: i64,
	pub ctime: i64,

	pub mode: u32,
    pub uid: u32,
    pub gid: u32,

}

// ex: noet ts=4 filetype=rust
