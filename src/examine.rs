use std::hash::Hasher;
use std::hash::SipHasher;
use std::io::Read;
use std::fs::File;
use std::path::Path;

use misc::*;
use types::*;

fn checksum_for_file <PathRef: AsRef <Path>> (
	path: PathRef,
) -> Result <u64, String> {

	let mut file =
		try! (
			io_result (
				File::open (
					path)));

	let mut hasher =
		SipHasher::new ();

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

		hasher.write (
			& buffer [
				0 .. bytes_read]);

	}

	Ok (hasher.finish ())

}

pub fn split_by_hash (
	filename_and_size_lists: FilenameAndSizeLists,
) -> Result <FilenameAndChecksumLists, String> {

	let mut result =
		FilenameAndChecksumLists::new ();

	let mut progress: usize = 0;
	let target = filename_and_size_lists.len ();

	stderrln! (
		"Checksum progress: 0%");

	for (filename_and_size, path_list)
		in filename_and_size_lists {

		for path in path_list {

			let checksum =
				try! (
					checksum_for_file (
						& path));

			let coinciding_paths =
				result.entry (
					FilenameAndChecksum {
						filename: filename_and_size.filename.clone (),
						checksum: checksum,
					}
				).or_insert (
					Vec::new (),
				);

			coinciding_paths.push (
				path);

		}

		progress += 1;

		if progress % 256 == 0 {

			stderrln! (
				"{}Checksum progress: {}%",
				"\x1b[A",
				progress * 100 / target);

		}

	}

	Ok (result)

}

// ex: noet ts=4 filetype=rust
