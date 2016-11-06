use sha2::Digest;
use sha2::Sha256;

use std::io::Read;
use std::fs::File;
use std::path::Path;

use misc::*;
use output::*;
use types::*;

fn checksum_for_file <PathRef: AsRef <Path>> (
	path: PathRef,
) -> Result <Hash, String> {

	let mut file =
		try! (
			io_result (
				File::open (
					path)));

	let mut hasher =
		Sha256::new ();

	let mut buffer: [u8; 0x1000] =
		[0u8; 0x1000];

	loop {

		let bytes_read =
			try! (
				io_result (
					file.read (
						& mut buffer)));

		if bytes_read == 0 {
			break;
		}

		hasher.input (
			& buffer [
				0 .. bytes_read]);

	}

	let mut result: Hash =
		[0u8; HASH_SIZE];

	result.copy_from_slice (
		& hasher.result ());

	Ok (result)

}

pub fn split_by_hash (
	output: & mut Output,
	file_metadata_lists: FileMetadataLists,
) -> (HashLists, u64) {

	let mut result =
		HashLists::new ();

	let mut error_count: u64 = 0;

	let mut progress: usize = 0;
	let target = file_metadata_lists.len ();

	output.status (
		"Checksum progress: 0%");

	for (_file_metadata, path_list)
		in file_metadata_lists {

		for path in path_list {

			match checksum_for_file (
				& path,
			) {

				Ok (checksum) => {

					let coinciding_paths =
						result.entry (
							checksum,
						).or_insert (
							Vec::new (),
						);

					coinciding_paths.push (
						path);

				},

				Err (error) => {

					output.message (
						& format! (
							"Error checksumming {:?}: {}",
							& path,
							error));

					error_count += 1;

				},

			}

		}

		progress += 1;

		if progress % 256 == 0 {

			output.status (
				& format! (
					"Checksum progress: {}%",
					progress * 100 / target));

		}

	}

	output.clear_status ();

	(result, error_count)

}

// ex: noet ts=4 filetype=rust
