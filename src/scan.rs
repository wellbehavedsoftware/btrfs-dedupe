#![ allow (unused_parens) ]

use std::fs;
use std::fs::DirEntry;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::rc::Rc;

use output::Output;

use arguments::*;
use misc::*;
use storage::*;

fn scan_directory_internal (
	arguments: & Arguments,
	output: & mut Output,
	directory: Rc <PathBuf>,
	root_path: Rc <PathBuf>,
	in_iterator: & mut FileDatabaseIntoIterator,
	out_database: & mut FileDatabase,
	progress: & mut u64,
) -> Result <(), String> {

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
				"Error reading directory: {:?}: {}",
				directory,
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
					in_iterator.peek ();

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

				out_database.insert_direct (
					in_iterator.next ().unwrap ());

			} else {

				in_iterator.next ();

			}

		}

		let metadata =
			try! (

			fs::symlink_metadata (
				entry.path (),
			).map_err (
				|error|

				format! (
					"Error reading metadata for: {:?}: {}",
					entry.path (),
					error)

			)

		);

		let file_type =
			metadata.file_type ();

		if (

			file_type.is_symlink ()

			|| metadata.len () == 0

		) {

			// ignore

		} else if file_type.is_dir () {

			try! (
				scan_directory_internal (
					arguments,
					output,
					entry_path,
					root_path.clone (),
					in_iterator,
					out_database,
					progress));

		} else if file_type.is_file () {

			let exists = {

				let in_next_option =
					in_iterator.peek ();

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
					in_iterator.next ().unwrap ();

				let changed = (

					metadata.len () !=
						existing_file_data.size

				||

					metadata.mtime () !=
						existing_file_data.mtime

				);

				if changed {

					out_database.insert_update_metadata (
						& existing_file_data,
						root_path.clone (),
						& metadata,
					);

				} else {

					out_database.insert_direct (
						existing_file_data)

				}

			} else {

				out_database.insert_new (
					Rc::new (
						entry.path ()),
					root_path.clone (),
					& metadata,
				);

			}

		} else {

			stderrln! (
				"Ignoring unknown filetype: {:?}: {:?}",
				file_type,
				entry.path ());

		};

		if * progress % 0x1000 == 0 {

			output.status (
				& format! (
					"Scanning filesystem... {:?}",
					entry.path ()));

		}

		* progress += 1;

	}

	Ok (())

}

pub fn scan_directories (
	arguments: & Arguments,
	output: & mut Output,
	previous_database: FileDatabase,
) -> Result <FileDatabase, String> {

	let mut new_database: FileDatabase =
		FileDatabase::new ();

	let mut progress: u64 = 0;

	let previous_database_iterator =
		previous_database.into_iter ();

	let mut previous_database_iterator =
		previous_database_iterator.peekable ();

	for root_path in & arguments.root_paths {

		let root_path =
			new_database.get_path (
				root_path.clone ());

		loop {

			{

				let existing_file_data_option =
					previous_database_iterator.peek ();

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

			new_database.insert_direct (
				previous_database_iterator.next ().unwrap ());

		}

		try! (
			scan_directory_internal (
				arguments,
				output,
				root_path.clone (),
				root_path.clone (),
				& mut previous_database_iterator,
				& mut new_database,
				& mut progress));

	}

	for existing_file_data_ref in previous_database_iterator {

		new_database.insert_direct (
			existing_file_data_ref.clone ());

	}

	output.clear_status ();

	output.message (
		& format! (
			"Scanned {} files",
			progress));

	output.message (
		& format! (
			"Total {} files in database",
			new_database.len ()));

	Ok (new_database)

}

// ex: noet ts=4 filetype=rust
