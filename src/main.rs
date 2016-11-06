#[ macro_use ]
extern crate clap;

extern crate btrfs;
extern crate sha2;
extern crate termion;

#[ doc (hidden) ]
#[ macro_use ]
mod misc;

mod examine;
mod output;
mod scan;
mod types;

use std::path::PathBuf;
use std::process;

use types::*;
use output::Output;

fn main () {

	let arguments =
		parse_arguments ();

	let mut output =
		Output::new ();

	match main_real (
		& arguments,
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

fn parse_arguments (
) -> Arguments {

	let argument_matches = (
		clap::App::new ("Btrfs Dedupe")

		.arg (
			clap::Arg::with_name ("match-filename")
				.long ("match-filename")
				.help ("Match filename as well as checksum")
		)

		.arg (
			clap::Arg::with_name ("root-path")
				.multiple (true)
				.value_name ("PATH")
				.help ("Root path to scan for files")
		)

	).get_matches ();

	let match_filename =
		argument_matches.is_present (
			"match-filename",
		);

	let root_paths = (
		argument_matches.values_of_os (
			"root-path",
		)
	).map (
		|os_values|

		os_values.map (
			|os_value|

			PathBuf::from (os_value)

		).collect ()

	).unwrap_or (
		Vec::new (),
	);

	Arguments {
		match_filename: match_filename,
		root_paths: root_paths,
	}

}

fn main_real (
	arguments: & Arguments,
	output: & mut Output,
) -> Result <(), String> {

	// create a list of all files with the same name and size

	let file_metadata_lists: FileMetadataLists = try! (
		scan::scan_directories (
			& arguments,
			output)
	);

	// discard any with only a single coincidence

	let file_metadata_lists: FileMetadataLists =
		file_metadata_lists.into_iter ().filter (
			|& (_, ref value)|

		value.len () > 1

	).collect ();

	output.message (
		& format! (
			"Found {} {} that coincide",
			file_metadata_lists.len (),
			if arguments.match_filename {
				"filenames and sizes"
			} else {
				"sizes"
			}));

	// perform a checksum on each one

	let (hash_lists, checksum_error_count) =
		examine::split_by_hash (
			output,
			file_metadata_lists);

	output.message (
		& format! (
			"Found {} {} that coincide",
			hash_lists.len (),
			if arguments.match_filename {
				"filenames and hashes"
			} else {
				"checksums"
			}));

	if checksum_error_count > 0 {

		output.message (
			& format! (
				"Encountered {} errors while calculating checksums",
				checksum_error_count));

	}

	if hash_lists.is_empty () {
		return Ok (());
	}

	// deduplicate the matches

	let mut progress: usize = 0;
	let target = hash_lists.len ();

	let mut error_count: u64 = 0;

	output.status (
		"Deduplication progress: 0%");

	for (_, paths)
	in hash_lists.into_iter () {

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
