use std::collections::HashSet;
use std::fs::File;
use std::io::Read;

use output::Output;

use sha2::Digest;
use sha2::Sha256;

use time;

use misc::*;
use database::*;
use types::*;

pub struct ContentHasher <'a> {

	root_paths_set: HashSet <PathRef>,

	batch_size: u64,

	file_database: & 'a mut FileDatabase,

	num_ignored: u64,
	num_fresh: u64,
	num_updated: u64,
	num_remaining: u64,
	num_errors: u64,

}

impl <'a> ContentHasher <'a> {

	pub fn new (
		root_paths: & 'a [PathRef],
		batch_size: u64,
		file_database: & 'a mut FileDatabase,
	) -> ContentHasher <'a> {

		let root_paths_set: HashSet <PathRef> =
			root_paths.iter ().map (
				|root_path|
				root_path.clone ()
			).collect ();

		ContentHasher {

			root_paths_set: root_paths_set,

			batch_size: batch_size,

			file_database: file_database,

			num_ignored: 0,
			num_fresh: 0,
			num_updated: 0,
			num_remaining: 0,
			num_errors: 0,

		}

	}

	pub fn calculate_hashes (
		& mut self,
		output: & Output,
	) {

		let mut num_ignored: u64 = 0;
		let mut num_fresh: u64 = 0;
		let mut num_remaining: u64 = 0;
		let mut num_updated: u64 = 0;
		let mut num_errors: u64 = 0;

		let mut size_hashed: u64 = 0;

		for ref mut file_data
		in self.file_database.iter_mut () {

			if (

				(

					file_data.root_path.is_none ()

				) || (

					file_data.root_path.is_some ()

					&& ! self.root_paths_set.contains (
						& file_data.root_path.as_ref ().unwrap ().clone ())

				)

			) {

				num_ignored += 1;

				continue;

			} else if (
				file_data.content_hash != ZERO_HASH
				|| file_data.size == 0
			) {

				num_fresh += 1;

				continue;

			} else if (
				num_updated > 0
				&& size_hashed + file_data.size > self.batch_size
			) {

				num_remaining += 1;

				continue;

			} else {

				output.status_format (
					format_args! (
						"Content hash: {}",
						file_data.path.to_string_lossy ()));

				let content_hash_time =
					time::get_time ();

				if let Ok (content_hash) = (
					calculate_hash_for_file (
						file_data.path.clone ())
				) {

					if content_hash != file_data.content_hash {

						file_data.content_hash = content_hash;
						file_data.content_hash_time = content_hash_time.sec;

						file_data.extent_hash = ZERO_HASH;
						file_data.extent_hash_time = 0;

						file_data.defragment_time = 0;
						file_data.deduplicate_time = 0;

					}

					num_updated += 1;

				} else {

					num_errors += 1;

				}

				size_hashed += file_data.size;

			}

		}

		self.num_ignored = num_ignored;
		self.num_fresh = num_fresh;
		self.num_remaining = num_remaining;
		self.num_updated += num_updated;
		self.num_errors += num_errors;

		output.clear_status ();

	}

	pub fn file_database (& self) -> & FileDatabase {
		self.file_database
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

// ex: noet ts=4 filetype=rust
