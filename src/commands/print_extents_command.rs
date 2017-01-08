use btrfs;

use output::Output;

use arguments::*;

pub fn print_extents_command (
	output: & Output,
	arguments: & Arguments,
) -> Result <(), String> {

	for path in arguments.root_paths.iter () {

		output.message_format (
			format_args! (
				"Extents for {}",
				path.to_string_lossy ()));

		let file_extents =
			try! (
				btrfs::get_file_extent_map_for_path (
					path.as_ref ()));

		for file_extent in file_extents {

			output.message_format (
				format_args! (
					"  {:?}",
					file_extent));

		}

	}

	Ok (())

}

// ex: noet ts=4 filetype=rust
