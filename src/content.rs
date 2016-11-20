use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::rc::Rc;

use output::Output;

use sha2::Digest;
use sha2::Sha256;

use time;

use arguments::*;
use misc::*;
use storage::*;
use types::*;

pub struct ContentHasher {
	num_ignored: u64,
	num_fresh: u64,
	num_updated: u64,
	num_remaining: u64,
	num_errors: u64,
}

impl ContentHasher {

	pub fn new (
	) -> ContentHasher {

		ContentHasher {
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
	) -> FileDatabase {

		let mut new_database: FileDatabase =
			FileDatabase::new ();

		let mut num_ignored: u64 = 0;
		let mut num_fresh: u64 = 0;
		let mut num_remaining: u64 = 0;
		let mut num_updated: u64 = 0;
		let mut num_errors: u64 = 0;

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

			} else if file_data.content_hash != ZERO_HASH {

				new_database.insert_direct (
					file_data);

				num_fresh += 1;

			} else if (
				num_updated > 0
				&& size_hashed + file_data.size > arguments.content_hash_batch_size
			) {

				new_database.insert_direct (
					file_data);

				num_remaining += 1;

			} else {

				output.status (
					& format! (
						"Content hash: {}",
						file_data.path.to_string_lossy ()));

				let content_hash_time =
					time::get_time ();

				if let Ok (content_hash) = (
					calculate_hash_for_file (
						file_data.path.clone ())
				) {

					new_database.insert_update_content_hash (
						& file_data,
						file_data.root_path.as_ref ().unwrap ().clone (),
						content_hash,
						content_hash_time.sec);

					size_hashed += file_data.size;
					num_updated += 1;

				} else {

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

		new_database

	}

	pub fn num_ignored (& self) -> u64 {
		self.num_ignored
	}

	pub fn num_fresh (& self) -> u64 {
		self.num_fresh
	}

	pub fn num_updated (& self) -> u64 {
		self.num_updated
	}

	pub fn num_remaining (& self) -> u64 {
		self.num_remaining
	}

	pub fn num_errors (& self) -> u64 {
		self.num_errors
	}

	pub fn num_processed (& self) -> u64 {
		self.num_updated + self.num_errors
	}

	pub fn num_to_process (& self) -> u64 {
		self.num_updated + self.num_errors + self.num_remaining
	}

}

fn calculate_hash_for_file (
	path: PathRef,
) -> Result <Hash, String> {

	let mut file =
		try! (
			io_result (
				File::open (
					path.as_ref ())));

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

pub fn calculate_content_hashes (
	arguments: & Arguments,
	output: & mut Output,
	file_database: FileDatabase,
) -> Result <FileDatabase, String> {

	let mut file_database =
		file_database;

	let mut content_hasher =
		ContentHasher::new ();

	loop {

		// calculate a batch of hashes

		file_database =
			content_hasher.calculate_hashes (
				arguments,
				output,
				file_database);

		if content_hasher.num_remaining () == 0 {
			break;
		}

		output.message (
			& format! (
				"Hashed contents of {} out of {} files, {} remaining",
				content_hasher.num_processed (),
				content_hasher.num_to_process (),
				content_hasher.num_remaining));

		// write out updated database

		try! (
			write_database (
				& arguments,
				output,
				& file_database));

	}

	output.message (
		& format! (
			"Hashed contents of {} files with {} errors, ignored {} with fresh \
			hashes",
			content_hasher.num_updated (),
			content_hasher.num_errors (),
			content_hasher.num_fresh ()));

	// write out updated database

	if content_hasher.num_updated > 0 {

		try! (
			write_database (
				& arguments,
				output,
				& file_database));

	}

	Ok (file_database)

}

// ex: noet ts=4 filetype=rust
