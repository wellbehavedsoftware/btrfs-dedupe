#![ allow (unused_parens) ]

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use misc::*;
use output::Output;
use types::*;

fn scan_directory_internal <AsPath: AsRef <Path>> (
	arguments: & Arguments,
	output: & mut Output,
	directory: AsPath,
	file_metadata_lists: & mut FileMetadataLists,
	progress: & mut u64,
) -> Result <(), String> {

	let directory =
		directory.as_ref ();

	for entry_result
	in try! (

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

	) {

		let entry =
			try! (

			io_result (
				entry_result,
			).map_err (
				|error|

				format! (
					"Error reading entry: {}",
					error)

			)

		);

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
					entry.path (),
					file_metadata_lists,
					progress));

		} else if file_type.is_file () {

			let paths =
				file_metadata_lists.entry (
					FileMetadata {

						filename: if arguments.match_filename {
							Some (
								PathBuf::from (
									entry.file_name ())
							)
						} else { None },

						size: metadata.len (),

					},
				).or_insert (
					Vec::new (),
				);

			paths.push (
				entry.path ());

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

/*
pub fn scan_directory <AsPath: AsRef <Path>> (
	directory: AsPath,
) -> Result <FilenameAndSizeCounts, String> {

	let mut result: FilenameAndSizeCounts =
		FilenameAndSizeCounts::new ();

	try! (
		scan_directory_internal (
			directory,
			& mut result));

	Ok (result)

}
*/

pub fn scan_directories (
	arguments: & Arguments,
	output: & mut Output,
) -> Result <FileMetadataLists, String> {

	let mut result: FileMetadataLists =
		FileMetadataLists::new ();

	let mut progress: u64 = 0;

	for directory in & arguments.root_paths {

		try! (
			scan_directory_internal (
				arguments,
				output,
				directory,
				& mut result,
				& mut progress));

	}

	output.clear_status ();

	output.message (
		& format! (
			"Found {} unique {}",
			result.len (),
			if arguments.match_filename {
				"filenames and sizes"
			} else {
				"file sizes"
			}));

	Ok (result)

}

// ex: noet ts=4 filetype=rust
