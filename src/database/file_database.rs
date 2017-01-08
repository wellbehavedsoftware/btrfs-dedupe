use std::collections::HashMap;
use std::error::Error;
use std::io::Read;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::ops::Index;
use std::slice;
use std::vec;

use rustc_serialize::hex::ToHex;
use serde_json;

use database::*;
use misc::*;
use types::*;

pub struct FileDatabase {
	file_data_ordered: Vec <FileData>,
}

impl FileDatabase {

	#[ doc (hidden) ]
	pub fn new (
		file_data_ordered: Vec <FileData>,
	) -> FileDatabase {

		FileDatabase {
			file_data_ordered: file_data_ordered,
		}

	}

	pub fn read (
		recursive_path_database: & mut RecursivePathDatabase,
		root_paths: & [PathRef],
		source: & mut Read,
	) -> Result <FileDatabase, String> {

		let source =
			BufReader::new (
				source);

		let mut database_builder =
			FileDatabaseBuilder::new ();

		let mut root_map: HashMap <RecursivePathRef, Option <PathRef>> =
			root_paths.iter ().map (
				|root_path|

				(

					recursive_path_database.for_path (
						root_path.as_ref (),
					).unwrap (),

					Some (
						root_path.clone ()),

				)

			).collect ();

		for input_line_result in source.lines () {

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
				recursive_path_database.for_path (
					file_data_record.path,
				).unwrap ();

			let root_path =
				database_builder.find_root (
					& mut root_map,
					file_path.clone ());

			let file_data =
				FileData {

				path: file_path,
				root_path: root_path,
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

				defragment_time:
					file_data_record.defragment_time.unwrap_or (0),

				deduplicate_time:
					file_data_record.deduplicate_time.unwrap_or (0),

				mtime: file_data_record.mtime,
				ctime: file_data_record.ctime,

				mode: file_data_record.mode,
				uid: file_data_record.uid,
				gid: file_data_record.gid,

			};

			database_builder.insert (
				file_data);

		}

		Ok (database_builder.build ())

	}

	pub fn write (
		& self,
		database_output: & mut Write,
	) -> Result <(), String> {

		for file_data
		in self.file_data_ordered.iter () {

			let file_data_record =
				FileDataRecord {

				path: file_data.path.to_path (),
				size: file_data.size,

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

				defragment_time: if file_data.defragment_time == 0 {
					None
				} else {
					Some (file_data.defragment_time)
				},

				deduplicate_time: if file_data.deduplicate_time == 0 {
					None
				} else {
					Some (file_data.deduplicate_time)
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

				database_output.write_all (
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

	pub fn iter (
		& self,
	) -> slice::Iter <FileData> {
		self.file_data_ordered.iter ()
	}

	pub fn iter_mut (
		& mut self,
	) -> slice::IterMut <FileData> {
		self.file_data_ordered.iter_mut ()
	}

	pub fn into_iter (
		self,
	) -> vec::IntoIter <FileData> {
		self.file_data_ordered.into_iter ()
	}

}

impl Index <usize> for FileDatabase {

	type Output = FileData;

	fn index (
		& self,
		index: usize,
	) -> & FileData {

		& self.file_data_ordered [index]

	}

}

// ex: noet ts=4 filetype=rust
