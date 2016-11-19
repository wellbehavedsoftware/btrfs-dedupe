use serde_json;

use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::LinkedList;
use std::collections::linked_list;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::fs::Metadata;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::iter::Peekable;
use std::ops::Deref;
use std::os::unix::fs::MetadataExt;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::rc::Rc;

use output::Output;

use rustc_serialize::hex::ToHex;

use arguments::*;
use misc::*;
use types::*;

pub use serde_types::FileDataRecord;

#[ derive (Debug, Eq, Hash, PartialEq) ]
pub struct FileData {

	pub path: Rc <PathBuf>,
	pub filename: Rc <PathBuf>,
	pub root_path: Option <Rc <PathBuf>>,

    pub size: u64,

	pub content_hash: Hash,
	pub content_hash_time: i64,

	pub extent_hash: Hash,
	pub extent_hash_time: i64,

	pub mtime: i64,
	pub ctime: i64,

	pub mode: u32,
    pub uid: u32,
    pub gid: u32,

}
pub type FileDataRef =
	Rc <FileData>;

pub type FileDataListRef =
	Rc <RefCell <Vec <FileDataRef>>>;

pub type FileDataByPath =
	HashMap <Rc <PathBuf>, FileDataRef>;

pub type FileDataByParent =
	HashMap <Rc <PathBuf>, FileDataListRef>;

pub type FileDataOrdered =
	LinkedList <FileDataRef>;

pub struct FileDatabase {
	path_pool: HashSet <Rc <PathBuf>>,
	file_data_by_parent: FileDataByParent,
	file_data_ordered: FileDataOrdered,
}

pub type FileDatabaseIntoIterator =
	Peekable <linked_list::IntoIter <FileDataRef>>;

impl FileDatabase {

	pub fn new (
	) -> FileDatabase {

		FileDatabase {

			path_pool:
				HashSet::new (),

			file_data_by_parent:
				FileDataByParent::new (),

			file_data_ordered:
				FileDataOrdered::new (),

		}

	}

	pub fn get_path (
		& mut self,
		path: Rc <PathBuf>,
	) -> Rc <PathBuf> {

		{

			let path_pool =
				& mut self.path_pool;

			if ! path_pool.contains (
				& path) {

				path_pool.insert (
					path.clone ());

			}

		}

		self.path_pool.get (
			& path,
		).unwrap ().clone ()

	}

	pub fn insert_direct (
		& mut self,
		file_data: FileDataRef,
	) {

		let parent =
			Rc::new (
				PathBuf::from (
					file_data.path.parent ().unwrap ()));

		let parent_list_entry = (
			self.file_data_by_parent.entry (
				parent,
			).or_insert (
				Rc::new (
					RefCell::new (
						Vec::new ())),
			)
		).clone ();

		let mut parent_list =
			parent_list_entry.deref ().borrow_mut ();

		parent_list.push (
			file_data.clone ());

		self.file_data_ordered.push_back (
			file_data.clone ());

	}

	pub fn insert_new (
		& mut self,
		path: Rc <PathBuf>,
		root_path: Rc <PathBuf>,
		metadata: & Metadata,
	) -> FileDataRef {

		let path =
			self.get_path (
				path);

		let filename =
			self.get_path (
				Rc::new (
					PathBuf::from (
						path.file_name ().unwrap ())));

		let file_data =
			Rc::new (

			FileData {

				path: path.clone (),
				filename: filename.clone (),
				root_path: Some (root_path.clone ()),

			    size: metadata.len (),

				content_hash: ZERO_HASH,
				content_hash_time: 0,

				extent_hash: ZERO_HASH,
				extent_hash_time: 0,

				mtime: metadata.mtime (),
				ctime: metadata.ctime (),

				mode: metadata.mode (),
				uid: metadata.uid (),
				gid: metadata.gid (),

			}

		);

		self.insert_direct (
			file_data.clone ());

		file_data

	}

	pub fn insert_update_metadata (
		& mut self,
		existing_file_data: & FileData,
		root_path: Rc <PathBuf>,
		new_metadata: & Metadata,
	) -> FileDataRef {

		let file_data =
			Rc::new (

			FileData {

				path: existing_file_data.path.clone (),
				filename: existing_file_data.filename.clone (),
				root_path: Some (root_path),

			    size: new_metadata.len (),

				content_hash: ZERO_HASH,
				content_hash_time: 0,

				extent_hash: ZERO_HASH,
				extent_hash_time: 0,

				mtime: new_metadata.mtime (),
				ctime: new_metadata.ctime (),

				mode: new_metadata.mode (),
				uid: new_metadata.uid (),
				gid: new_metadata.gid (),

			}

		);

		self.insert_direct (
			file_data.clone ());

		file_data

	}

	pub fn insert_update_content_hash (
		& mut self,
		existing_file_data: & FileData,
		root_path: Rc <PathBuf>,
		new_content_hash: Hash,
		new_content_hash_time: i64,
	) -> FileDataRef {

		let file_data =
			Rc::new (

			FileData {

				path: existing_file_data.path.clone (),
				filename: existing_file_data.filename.clone (),
				root_path: Some (root_path),

			    size: existing_file_data.size,

				content_hash: new_content_hash,
				content_hash_time: new_content_hash_time,

				extent_hash: ZERO_HASH,
				extent_hash_time: 0,

				mtime: existing_file_data.mtime,
				ctime: existing_file_data.ctime,

				mode: existing_file_data.mode,
				uid: existing_file_data.uid,
				gid: existing_file_data.gid,

			}

		);

		self.insert_direct (
			file_data.clone ());

		file_data

	}

	pub fn insert_update_fiemap_hash (
		& mut self,
		existing_file_data: & FileData,
		root_path: Rc <PathBuf>,
		new_extent_hash: Option <Hash>,
		new_extent_hash_time: i64,
	) -> FileDataRef {

		let file_data =
			Rc::new (

			FileData {

				path: existing_file_data.path.clone (),
				filename: existing_file_data.filename.clone (),
				root_path: Some (root_path),

			    size: existing_file_data.size,

				content_hash: existing_file_data.content_hash,
				content_hash_time: existing_file_data.content_hash_time,

				extent_hash: new_extent_hash.unwrap_or (ZERO_HASH),
				extent_hash_time: new_extent_hash_time,

				mtime: existing_file_data.mtime,
				ctime: existing_file_data.ctime,

				mode: existing_file_data.mode,
				uid: existing_file_data.uid,
				gid: existing_file_data.gid,

			}

		);

		self.insert_direct (
			file_data.clone ());

		file_data

	}

	pub fn insert_without_extent_hash (
		& mut self,
		existing_file_data: & FileData,
		root_path: Rc <PathBuf>,
	) -> FileDataRef {

		let file_data =
			Rc::new (

			FileData {

				path: existing_file_data.path.clone (),
				filename: existing_file_data.filename.clone (),
				root_path: Some (root_path),

			    size: existing_file_data.size,

				content_hash: existing_file_data.content_hash,
				content_hash_time: existing_file_data.content_hash_time,

				extent_hash: ZERO_HASH,
				extent_hash_time: 0,

				mtime: existing_file_data.mtime,
				ctime: existing_file_data.ctime,

				mode: existing_file_data.mode,
				uid: existing_file_data.uid,
				gid: existing_file_data.gid,

			}

		);

		self.insert_direct (
			file_data.clone ());

		file_data

	}

	pub fn write (
		& self,
		database_output: & mut Write,
	) -> Result <(), String> {

		for file_data
		in self.file_data_ordered.iter () {

			let file_data_record =
				FileDataRecord {

				path: file_data.path.clone (),
				size: file_data.size.clone (),

				content_hash: if file_data.content_hash == ZERO_HASH {
					None
				} else {
					Some (file_data.content_hash.to_hex ())
				},

				content_hash_time: if file_data.content_hash_time == 0 {
					None
				} else {
					Some (file_data.content_hash_time)
				},

				extent_hash: if file_data.extent_hash == ZERO_HASH {
					None
				} else {
					Some (file_data.extent_hash.to_hex ())
				},

				extent_hash_time: if file_data.extent_hash_time == 0 {
					None
				} else {
					Some (file_data.extent_hash_time)
				},

				mtime: file_data.mtime,
				ctime: file_data.ctime,

				mode: file_data.mode,
				uid: file_data.uid,
				gid: file_data.gid,

			};

			let file_data_json = try! (

				serde_json::to_string (
					& file_data_record,
				).map_err (
					|serde_error|

					format! (
						"Serialization error: {}",
						serde_error)

				)

			);

			try! (

				database_output.write (
					file_data_json.as_bytes (),
				).map_err (
					|io_error|

					format! (
						"IO error: {}",
						io_error.description ())

				)

			);

			try! (

				database_output.write (
					b"\n",
				).map_err (
					|io_error|

					format! (
						"IO error: {}",
						io_error.description ())

				)

			);

		}

		Ok (())

	}

	pub fn read (
		arguments: & Arguments,
		database_input: & mut Read,
	) -> Result <FileDatabase, String> {

		let database_input =
			BufReader::new (
				database_input);

		let mut database_output =
			FileDatabase::new ();

		let mut root_map: HashMap <Rc <PathBuf>, Option <Rc <PathBuf>>> =
			arguments.root_paths.iter ().map (
				|root_path|

				(
					root_path.clone (),
					Some (root_path.clone ()),
				)

			).collect ();

		for input_line_result in database_input.lines () {

			let input_line = try! (
				input_line_result.map_err (
					|io_error|

				format! (
					"IO error: {}",
					io_error.description ())

			));

			let file_data_record: FileDataRecord =
				try! (

				serde_json::from_str (
					& input_line,
				).map_err (
					|serde_error|

					format! (
						"Deserialization error: {}",
						serde_error)

				)

			);

			let file_path =
				database_output.get_path (
					file_data_record.path.clone ());

			let root_path =
				find_root (
					& mut database_output,
					& mut root_map,
					file_path.clone ());

			let file_data =
				Rc::new (
					FileData {

				path:
					database_output.get_path (
						file_data_record.path.clone ()),

				filename:
					database_output.get_path (
						Rc::new (
							PathBuf::from (
								file_data_record.path.file_name ().unwrap ()))),

				root_path:
					root_path,

				size: file_data_record.size,

				content_hash:
					decode_hash (
						& file_data_record.content_hash),

				content_hash_time:
					file_data_record.content_hash_time.unwrap_or (0),

				extent_hash:
					decode_hash (
						& file_data_record.extent_hash),

				extent_hash_time:
					file_data_record.extent_hash_time.unwrap_or (0),

				mtime: file_data_record.mtime,
				ctime: file_data_record.ctime,

				mode: file_data_record.mode,
				uid: file_data_record.uid,
				gid: file_data_record.gid,

			});

			database_output.insert_direct (
				file_data);

		}

		Ok (database_output)

	}

	pub fn len (
		& self,
	) -> usize {
		self.file_data_ordered.len ()
	}

	pub fn iter (
		& self,
	) -> linked_list::Iter <FileDataRef> {

		self.file_data_ordered.iter ()

	}

	pub fn into_iter (
		self,
	) -> linked_list::IntoIter <FileDataRef> {

		self.file_data_ordered.into_iter ()

	}

}

fn find_root (
	file_database: & mut FileDatabase,
	root_map: & mut HashMap <PathRef, Option <PathRef>>,
	file_path: PathRef,
) -> Option <PathRef> {

	let mut search_path =
		Some (file_path.clone ());

	let mut new_paths: Vec <PathRef> =
		Vec::new ();

	while (
		search_path.is_some ()
		&& ! root_map.contains_key (
			search_path.as_ref ().unwrap ())
	) {

		new_paths.push (
			search_path.as_ref ().unwrap ().clone ());

		let search_parent =
			search_path.unwrap ().parent ().map (
				|search_parent|

				file_database.get_path (
					Rc::new (
						PathBuf::from (
							search_parent)))

			);

		if search_parent.is_none () {

			search_path = None;

			break;

		}

		search_path =
			search_parent;

	}

	let root_path =
		search_path.and_then (
			|search_path|

		root_map.get (
			& search_path,
		).unwrap ().clone ()

	);

	for new_path in new_paths.into_iter () {

		root_map.insert (
			new_path,
			root_path.clone ());

	}

	root_path

}

pub fn init_database (
	arguments: & Arguments,
	output: & mut Output,
) -> Result <FileDatabase, String> {

	if (

		arguments.database_path.is_some ()

		&& arguments.database_path.as_ref ().unwrap ().exists ()

	) {

		output.message (
			& format! (
				"Reading database from {}",
				arguments.database_path
					.as_ref ()
					.unwrap ()
					.to_string_lossy ()));

		let mut database_file = try! (

			File::open (
				arguments.database_path
					.as_ref ()
					.unwrap (),
			).map_err (
				|io_error|

				format! (
					"Error reading database: {}",
					io_error.description ())

			)

		);

		FileDatabase::read (
			arguments,
			& mut database_file)

	} else {

		Ok (
			FileDatabase::new ()
		)

	}

}

pub fn write_database (
	arguments: & Arguments,
	output: & mut Output,
	file_database: & FileDatabase,
) -> Result <(), String> {

	if arguments.database_path.is_none () {
		return Ok (());
	}

	let database_path =
		arguments.database_path.as_ref ().unwrap ();

	output.message (
		& format! (
			"Writing database to {}",
			database_path.to_string_lossy ()));

	let database_path_temp_bytes: Vec <u8> =
		database_path.as_os_str ().as_bytes ().iter ().chain (
			b".temp".iter (),
		).map (
			|byte_ref|
			* byte_ref
		).collect ();

	let database_path_temp =
		PathBuf::from (
			OsStr::from_bytes (
				& database_path_temp_bytes));

	let mut database_file = try! (

		File::create (
			& database_path_temp,
		).map_err (
			|io_error|

			format! (
				"Error writing database: {}",
				io_error.description ())

		)

	);

	try! (
		file_database.write (
			& mut database_file));

	try! (
		database_file.sync_data (
		).map_err (
			|io_error|

			format! (
				"Error writing database: {}",
				io_error.description ())

		)
	);

	try! (
		fs::rename (
			& database_path_temp,
			database_path,
		).map_err (
			|io_error|

			format! (
				"Error writing database: {}",
				io_error.description ())

		)
	);

	Ok (())

}

// ex: noet ts=4 filetype=rust
