#![ allow (unused_parens) ]

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use misc::*;
use output::Output;
use types::*;

fn scan_directory_internal <AsPath: AsRef <Path>> (
	output: & mut Output,
	directory: AsPath,
	filename_and_size_counts: & mut FilenameAndSizeLists,
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
					output,
					entry.path (),
					filename_and_size_counts,
					progress));

		} else if file_type.is_file () {

			let paths =
				filename_and_size_counts.entry (
					FilenameAndSize {
						filename: PathBuf::from (
							entry.file_name ()),
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

pub fn scan_directories <AsPath: AsRef <Path>> (
	output: & mut Output,
	directories: & [AsPath],
) -> Result <FilenameAndSizeLists, String> {

	let mut result: FilenameAndSizeLists =
		FilenameAndSizeLists::new ();

	let mut progress: u64 = 0;

	for directory in directories {

		try! (
			scan_directory_internal (
				output,
				directory,
				& mut result,
				& mut progress));

	}

	output.clear_status ();

	output.message (
		& format! (
			"Found {} unique filenames and sizes",
			result.len ()));

	Ok (result)

}

// ex: noet ts=4 filetype=rust
