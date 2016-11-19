use std::collections::HashSet;
use std::mem;
use std::path::PathBuf;
use std::rc::Rc;
use std::slice;

use btrfs;
use btrfs::FileExtent;

use output::Output;

use sha2::Digest;
use sha2::Sha256;

use time;

use arguments::*;
use storage::*;
use types::*;

pub struct ExtentHasher {
	pub num_ignored: u64,
	pub num_fresh: u64,
	pub num_updated: u64,
	pub num_remaining: u64,
	pub num_errors: u64,
}

impl ExtentHasher {

	pub fn new (
	) -> ExtentHasher {

		ExtentHasher {
			num_ignored: 0,
			num_fresh: 0,
			num_updated: 0,
			num_remaining: 0,
			num_errors: 0,
		}

	}

	pub fn calculate_hashes (
		& mut self,
		arguments: & Arguments,
		output: & mut Output,
		previous_database: FileDatabase,
	) -> Result <FileDatabase, String> {

		let mut new_database: FileDatabase =
			FileDatabase::new ();

		let mut num_ignored = 0;
		let mut num_fresh = 0;
		let mut num_remaining = 0;
		let mut num_updated = 0;
		let mut num_errors = 0;

		let mut size_hashed: u64 = 0;

		let root_set: HashSet <Rc <PathBuf>> =
			arguments.root_paths.iter ().map (
				|root_path|
				root_path.clone ()
			).collect ();

		for file_data in previous_database.into_iter () {

			if (

				(

					file_data.root_path.is_none ()

				) || (

					file_data.root_path.is_some ()

					&& ! root_set.contains (
						& file_data.root_path.as_ref ().unwrap ().clone ())

				)

			) {

				new_database.insert_direct (
					file_data);

				num_ignored += 1;

			} else if file_data.extent_hash_time != 0 {

				new_database.insert_direct (
					file_data);

				num_fresh += 1;

			} else if (
				num_updated > 0
				&& size_hashed + file_data.size > arguments.extent_hash_batch_size
			) {

				new_database.insert_direct (
					file_data);

				num_remaining += 1;

			} else {

				output.status (
					& format! (
						"Extent hash: {:?}",
						file_data.path));

				let extent_hash_time =
					time::get_time ();

				if let Ok (extent_hash) = (
					calculate_extent_hash_for_file (
						file_data.path.clone ())
				) {

					new_database.insert_update_fiemap_hash (
						& file_data,
						file_data.root_path.as_ref ().unwrap ().clone (),
						extent_hash,
						extent_hash_time.sec);

					size_hashed += file_data.size;
					num_updated += 1;

				} else {

					size_hashed += file_data.size;
					num_errors += 1;

				}

			}

		}

		self.num_ignored = num_ignored;
		self.num_fresh = num_fresh;
		self.num_remaining = num_remaining;
		self.num_updated += num_updated;
		self.num_errors += num_errors;

		output.clear_status ();

		Ok (new_database)

	}

}

pub fn calculate_extent_hash_for_file (
	path: PathRef,
) -> Result <Option <Hash>, String> {

	let file_extents =
		try! (
			btrfs::get_file_extent_map_for_path (
				path.as_ref ()));

	let mut hasher =
		Sha256::new ();

	let mut physical_extents: u64 = 0;

	for file_extent in file_extents.iter () {

		if file_extent.physical == 0 {
			continue;
		}

		let file_extent_pointer =
			file_extent as * const FileExtent;

		hasher.input (
			unsafe {
				slice::from_raw_parts (
					file_extent_pointer as * const u8,
					mem::size_of::<FileExtent> ())
			}
		);

		physical_extents += 1;

	}

	if physical_extents > 0 {

		let mut result: Hash =
			[0u8; HASH_SIZE];

		result.copy_from_slice (
			& hasher.result ());

		Ok (Some (result))

	} else {

		Ok (None)

	}

}

pub fn calculate_extent_hashes (
	arguments: & Arguments,
	output: & mut Output,
	file_database: FileDatabase,
) -> Result <FileDatabase, String> {

	let mut file_database =
		file_database;

	let mut extent_hasher =
		ExtentHasher::new ();

	loop {

		// calculate a batch of hashes

		file_database =
			try! (
				extent_hasher.calculate_hashes (
					arguments,
					output,
					file_database));

		if extent_hasher.num_remaining == 0 {
			break;
		}

		output.message (
			& format! (
				"Hashed extents of {} out of {} files, {} remaining",
				extent_hasher.num_updated
					+ extent_hasher.num_errors,
				extent_hasher.num_updated
					+ extent_hasher.num_errors
					+ extent_hasher.num_remaining,
				extent_hasher.num_remaining));

		// write out updated database

		try! (
			write_database (
				& arguments,
				output,
				& file_database));

	}

	output.message (
		& format! (
			"Hashed extents of {} files, {} errors, skipped {}",
			extent_hasher.num_updated,
			extent_hasher.num_errors,
			extent_hasher.num_fresh));

	// write out updated database

	if extent_hasher.num_updated > 0 {

		try! (
			write_database (
				& arguments,
				output,
				& file_database));

	}

	Ok (file_database)

}

pub fn print_extents_command (
	arguments: & Arguments,
	output: & mut Output,
) -> Result <(), String> {

	for path in arguments.root_paths.iter () {

		output.message (
			& format! (
				"Extents for {:?}",
				path));

		let file_extents =
			try! (
				btrfs::get_file_extent_map_for_path (
					path.as_ref ()));

		for file_extent in file_extents {

			output.message (
				& format! (
					"  {:?}",
					file_extent));

		}

	}

	Ok (())

}

// ex: noet ts=4 filetype=rust
