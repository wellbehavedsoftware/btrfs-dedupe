extern crate btrfs;
extern crate termion;

#[ doc (hidden) ]
#[ macro_use ]
mod misc;

mod examine;
mod output;
mod scan;
mod types;

use std::env;
use std::path::PathBuf;
use std::process;

use types::*;
use output::Output;

fn main () {

	let mut output =
		Output::new ();

	match main_real (
		& mut output,
	) {

		Ok (_) => (),

		Err (error_message) => {

			output.clear_status ();

			output.message (
				& format! (
					"Error: {}",
					error_message));

			process::exit (1);

		},

	}

}

fn main_real (
	output: & mut Output,
) -> Result <(), String> {

	// create a list of all files with the same name and size

	let directories: Vec <PathBuf> =
		env::args_os ().skip (1).map (
			|argument|

		PathBuf::from (
			argument)

	).collect ();

	let filename_and_size_lists: FilenameAndSizeLists =
		try! (
			scan::scan_directories (
				output,
				& directories));

	// discard any with only a single coincidence

	let filename_and_size_lists: FilenameAndSizeLists =
		filename_and_size_lists.into_iter ().filter (
			|& (_, ref value)|

		value.len () > 1

	).collect ();

	output.message (
		& format! (
			"Found {} filenames and sizes that coincide",
			filename_and_size_lists.len ()));

	// perform a checksum on each one

	let (filename_and_checksum_lists, checksum_error_count) =
		examine::split_by_hash (
			output,
			filename_and_size_lists);

	output.message (
		& format! (
			"Found {} filenames and checksums that coincide",
			filename_and_checksum_lists.len ()));

	if checksum_error_count > 0 {

		output.message (
			& format! (
				"Encountered {} errors while calculating checksums",
				checksum_error_count));

	}

	if filename_and_checksum_lists.is_empty () {
		return Ok (());
	}

	// deduplicate the matches

	let mut progress: usize = 0;
	let target = filename_and_checksum_lists.len ();

	let mut error_count: u64 = 0;

	output.status (
		"Deduplication progress: 0%");

	for (_, paths)
	in filename_and_checksum_lists.into_iter () {

		let (first_path_slice, other_paths) =
			paths.split_at (1);

		let first_path =
			first_path_slice.into_iter ().next ().unwrap ();

		for paths_chunk in other_paths.chunks (512) {

			match btrfs::deduplicate_files_with_source (
				first_path,
				paths_chunk,
			) {

				Ok (_) => (),
				Err (_) => error_count += 1,

			}

		}

		progress += 1;

		if progress % 256 == 0 {

			output.status (
				& format! (
					"Deduplication progress: {}%",
					progress * 100 / target));

		}

	};

	output.clear_status ();

	if error_count > 0 {

		output.message (
			& format! (
				"Encountered {} errors during deduplication",
				error_count));

	} else {

		output.message (
			& format! (
				"Deduplicated all files successfully"));

	}

	// return

	Ok (())

}

// ex: noet ts=4 filetype=rust
