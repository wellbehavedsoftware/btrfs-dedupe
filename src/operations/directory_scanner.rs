use std::collections::HashSet;
use std::fs;
use std::fs::DirEntry;
use std::io;
use std::iter::Peekable;
use std::os::unix::fs::MetadataExt;
use std::rc::Rc;
use std::vec;

use output::Output;

use misc::*;
use database::*;
use types::*;

pub struct DirectoryScanner <'a> {

	root_paths: & 'a [PathRef],

	in_iterator: Peekable <vec::IntoIter <FileData>>,
	out_builder: FileDatabaseBuilder,

	root_paths_unordered: HashSet <PathRef>,
	root_paths_scanned: HashSet <PathRef>,

	progress: u64,

}

impl <'a> DirectoryScanner <'a> {

	pub fn new (
		root_paths: & [PathRef],
		file_database: FileDatabase,
	) -> DirectoryScanner {

		let root_paths_set: HashSet <PathRef> =
			root_paths.iter ().map (
				|root_path|

				root_path.clone ()

			).collect ();

		let previous_database_iterator =
			file_database.into_iter ();

		let previous_database_iterator =
			previous_database_iterator.peekable ();

		let new_database_builder: FileDatabaseBuilder =
			FileDatabaseBuilder::new ();

		DirectoryScanner {

			root_paths: root_paths,

			in_iterator: previous_database_iterator,
			out_builder: new_database_builder,

			root_paths_unordered: root_paths_set,
			root_paths_scanned: HashSet::new (),

			progress: 0,

		}

	}

	pub fn scan_directories (
		mut self,
		output: & Output,
		recursive_path_database: & mut RecursivePathDatabase,
	) -> Result <FileDatabase, String> {

		for root_path in self.root_paths.iter () {

			if self.root_paths_scanned.contains (root_path) {
				continue;
			}

			output.message_format (
				format_args! (
					"Scanning {}",
					root_path.to_string_lossy ()));

			let root_recursive_path =
				recursive_path_database.for_path (
					root_path.as_ref (),
				).unwrap ();

			let metadata =
				try! (

				fs::symlink_metadata (
					root_path.as_ref (),
				).map_err (
					|error|

					format! (
						"Error reading metadata for: {}: {}",
						root_path.to_string_lossy (),
						error)

				)

			);

			loop {

				{

					let existing_file_data_option =
						self.in_iterator.peek ();

					if existing_file_data_option.is_none () {
						break;
					}

					let existing_file_data =
						existing_file_data_option.unwrap ();

					let existing_file_path =
						& existing_file_data.path;

					if * existing_file_path >= root_recursive_path {
						break;
					}

				}

				self.out_builder.insert (
					self.in_iterator.next ().unwrap ());

			}

			self.scan_directory_internal (
				output,
				recursive_path_database,
				root_path.clone (),
				root_path.clone (),
				metadata.dev (),
			) ?;

		}

		for existing_file_data_ref
		in self.in_iterator {

			self.out_builder.insert (
				existing_file_data_ref);

		}

		output.clear_status ();

		output.message_format (
			format_args! (
				"Scanned {} files",
				self.progress));

		output.message_format (
			format_args! (
				"Total {} files in database",
				self.out_builder.len ()));

		Ok (self.out_builder.build ())

	}

	fn scan_directory_internal (
		& mut self,
		output: & Output,
		recursive_path_database: & mut RecursivePathDatabase,
		directory: PathRef,
		root_path: PathRef,
		device_id: u64,
	) -> Result <(), String> {

		if (
			self.root_paths_unordered.contains (
				& directory)
		) {
			self.root_paths_scanned.insert (
				directory.clone ());
		}

		let directory =
			directory.as_ref ();

		let entry_results: Vec <io::Result <DirEntry>> =
			try! (

			io_result (
				fs::read_dir (
					directory),
			).map_err (
				|error|

				format! (
					"Error reading directory: {}: {}",
					directory.to_string_lossy (),
					error)

			)

		).collect ();

		let mut entries: Vec <DirEntry> =
			Vec::new ();

		for entry_result in entry_results.into_iter () {

			entries.push (
				try! (

				io_result (
					entry_result,
				).map_err (
					|error|

					format! (
						"Error reading entry: {}",
						error)

				)

			));

		}

		entries.sort_by_key (
			|entry|

			entry.file_name ()

		);

		let mut entry_iterator =
			entries.into_iter ().peekable ();

		loop {

			{

				let entry_next =
					entry_iterator.peek ();

				if entry_next.is_none () {
					break;
				}

			}

			let entry =
				entry_iterator.next ().unwrap ();

			let entry_recursive_path =
				recursive_path_database.for_path (
					entry.path (),
				).unwrap ();

			loop {

				let exists = {

					let in_next_option =
						self.in_iterator.peek ();

					if in_next_option.is_none () {
						break;
					}

					let in_next =
						in_next_option.unwrap ();

					if in_next.path >= entry_recursive_path {
						break;
					}

					in_next.path.to_path ().exists ()

				};

				if exists {

					self.out_builder.insert (
						self.in_iterator.next ().unwrap ());

				} else {

					self.in_iterator.next ();

				}

			}

			let entry_metadata =
				try! (

				fs::symlink_metadata (
					entry.path (),
				).map_err (
					|error|

					format! (
						"Error reading metadata for: {}: {}",
						entry.path ().to_string_lossy (),
						error)

				)

			);

			let entry_file_type =
				entry_metadata.file_type ();

			let (temp_root_path, temp_device_id) =
				if self.root_paths_unordered.contains (
					& entry.path ()) {

				(
					Rc::new (entry.path ()),
					entry_metadata.dev (),
				)

			} else {

				(
					root_path.clone (),
					device_id,
				)

			};

			if (
				entry_file_type.is_symlink ()
				|| entry_metadata.dev () != temp_device_id
			) {

				// ignore

			} else if entry_file_type.is_dir () {

				self.scan_directory_internal (
					output,
					recursive_path_database,
					Rc::new (entry.path ()),
					temp_root_path,
					temp_device_id,
				) ?;

			} else if entry_file_type.is_file () {

				let exists = {

					let in_next_option =
						self.in_iterator.peek ();

					if in_next_option.is_some () {

						let in_next =
							in_next_option.unwrap ();

						let in_next_path =
							in_next.path.clone ();

						in_next_path == entry_recursive_path

					} else {

						false

					}

				};

				if exists {

					let mut file_data =
						self.in_iterator.next ().unwrap ();

					let changed = (

						entry_metadata.len () !=
							file_data.size

					||

						entry_metadata.mtime () !=
							file_data.mtime

					);

					if changed {

						file_data.size = entry_metadata.len ();

						file_data.content_hash = ZERO_HASH;
						file_data.content_hash_time = 0;

						file_data.extent_hash = ZERO_HASH;
						file_data.extent_hash_time = 0;

						file_data.defragment_time = 0;
						file_data.deduplicate_time = 0;

						file_data.mtime = entry_metadata.mtime ();
						file_data.ctime = entry_metadata.ctime ();

						file_data.mode = entry_metadata.mode ();
						file_data.uid = entry_metadata.uid ();
						file_data.gid = entry_metadata.gid ();

					}

					self.out_builder.insert (
						file_data);

				} else {

					self.out_builder.insert (
						FileData {

						path: entry_recursive_path,
						root_path: Some (root_path.clone ()),

						size: entry_metadata.len (),

						content_hash: ZERO_HASH,
						content_hash_time: 0,

						extent_hash: ZERO_HASH,
						extent_hash_time: 0,

						defragment_time: 0,
						deduplicate_time: 0,

						mtime: entry_metadata.mtime (),
						ctime: entry_metadata.ctime (),

						mode: entry_metadata.mode (),
						uid: entry_metadata.uid (),
						gid: entry_metadata.gid (),

					});

				}

			};

			if self.progress % 0x1000 == 0 {

				output.status_format (
					format_args! (
						"Scanning filesystem: {}",
						entry.path ().to_string_lossy ()));

			}

			self.progress += 1;

		}

		Ok (())

	}

}

// ex: noet ts=4 filetype=rust
