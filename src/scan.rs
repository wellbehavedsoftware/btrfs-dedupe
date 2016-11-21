#![ allow (unused_parens) ]

use std::collections::HashSet;
use std::fs;
use std::fs::DirEntry;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::rc::Rc;

use output::Output;

use arguments::*;
use misc::*;
use storage::*;
use types::*;

pub fn scan_directories (
	arguments: & Arguments,
	output: & mut Output,
	previous_database: FileDatabase,
) -> Result <FileDatabase, String> {

	let root_paths: HashSet <PathRef> =
		arguments.root_paths.iter ().map (
			|root_path|

			root_path.clone ()

		).collect ();

	let previous_database_iterator =
		previous_database.into_iter ();

	let previous_database_iterator =
		previous_database_iterator.peekable ();

	let new_database: FileDatabase =
		FileDatabase::new ();

	let directory_scanner =
		DirectoryScanner {

		arguments: arguments,
		output: output,

		in_iterator: previous_database_iterator,
		out_database: new_database,

		root_paths_unordered: root_paths,
		root_paths_scanned: HashSet::new (),

		progress: 0,

	};

	directory_scanner.scan_directories ()

}

struct DirectoryScanner <'a> {

	arguments: & 'a Arguments,
	output: & 'a mut Output,

	in_iterator: FileDatabaseIntoIterator,
	out_database: FileDatabase,

	root_paths_unordered: HashSet <PathRef>,
	root_paths_scanned: HashSet <PathRef>,

	progress: u64,

}

impl <'a> DirectoryScanner <'a> {

	pub fn scan_directories (
		mut self,
	) -> Result <FileDatabase, String> {

		for root_path in & self.arguments.root_paths {

			if self.root_paths_scanned.contains (root_path) {
				continue;
			}

			self.output.message (
				& format! (
					"Scanning {}",
					root_path.to_string_lossy ()));

			let root_path =
				self.out_database.get_path (
					root_path.clone ());

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

					if * existing_file_path >= root_path {
						break;
					}

				}

				self.out_database.insert_direct (
					self.in_iterator.next ().unwrap ());

			}

			try! (
				self.scan_directory_internal (
					root_path.clone (),
					root_path.clone (),
					metadata.dev ()));

		}

		for existing_file_data_ref
		in self.in_iterator {

			self.out_database.insert_direct (
				existing_file_data_ref.clone ());

		}

		self.output.clear_status ();

		self.output.message (
			& format! (
				"Scanned {} files",
				self.progress));

		self.output.message (
			& format! (
				"Total {} files in database",
				self.out_database.len ()));

		Ok (self.out_database)

	}

	fn scan_directory_internal (
		& mut self,
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

			let entry_path =
				Rc::new (
					entry.path ());

			loop {

				let exists = {

					let in_next_option =
						self.in_iterator.peek ();

					if in_next_option.is_none () {
						break;
					}

					let in_next =
						in_next_option.unwrap ();

					let in_next_path =
						in_next.path.clone ();

					if in_next_path >= entry_path {
						break;
					}

					in_next_path.exists ()

				};

				if exists {

					self.out_database.insert_direct (
						self.in_iterator.next ().unwrap ());

				} else {

					self.in_iterator.next ();

				}

			}

			let metadata =
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

			let file_type =
				metadata.file_type ();

			if (

				file_type.is_symlink ()

				|| metadata.dev () != device_id

			) {

				// ignore

			} else if file_type.is_dir () {

				try! (
					self.scan_directory_internal (
						entry_path,
						root_path.clone (),
						device_id));

			} else if file_type.is_file () {

				let exists = {

					let in_next_option =
						self.in_iterator.peek ();

					if in_next_option.is_some () {

						let in_next =
							in_next_option.unwrap ();

						let in_next_path =
							in_next.path.clone ();

						in_next_path == entry_path

					} else {

						false

					}

				};

				if exists {

					let existing_file_data =
						self.in_iterator.next ().unwrap ();

					let changed = (

						metadata.len () !=
							existing_file_data.size

					||

						metadata.mtime () !=
							existing_file_data.mtime

					);

					if changed {

						self.out_database.insert_update_metadata (
							& existing_file_data,
							root_path.clone (),
							& metadata,
						);

					} else {

						self.out_database.insert_direct (
							existing_file_data)

					}

				} else {

					self.out_database.insert_new (
						Rc::new (
							entry.path ()),
						root_path.clone (),
						& metadata,
					);

				}

			};

			if self.progress % 0x1000 == 0 {

				self.output.status (
					& format! (
						"Scanning filesystem: {}",
						entry.path ().to_string_lossy ()));

			}

			self.progress += 1;

		}

		Ok (())

	}

}

// ex: noet ts=4 filetype=rust
