use std::collections::HashMap;
use std::collections::HashSet;

use btrfs;

use output::Output;

use time;

use arguments::*;
use database::*;
use types::*;

pub struct FileDeduper {
	num_ignored: u64,
	num_fresh: u64,
	num_updated: u64,
	num_remaining: u64,
	num_errors: u64,
}

impl FileDeduper {

	pub fn new (
	) -> FileDeduper {

		FileDeduper {
			num_ignored: 0,
			num_fresh: 0,
			num_updated: 0,
			num_remaining: 0,
			num_errors: 0,
		}

	}

	pub fn dedupe_files (
		& mut self,
		output: & Output,
		arguments: & Arguments,
		file_database: & mut FileDatabase,
		dedupe_map: & mut HashMap <RecursivePathRef, RecursivePathRef>,
	) -> Result <(), String> {

		let mut num_ignored = 0;
		let mut num_fresh = 0;
		let mut num_remaining = 0;
		let mut num_updated = 0;
		let mut num_errors = 0;

		let mut size_deduped: u64 = 0;

		let root_set: HashSet <PathRef> =
			arguments.root_paths.iter ().map (
				|root_path|
				root_path.clone ()
			).collect ();

		for ref mut file_data
		in file_database.iter_mut () {

			if (

				(

					file_data.root_path.is_none ()

				) || (

					file_data.root_path.is_some ()

					&& ! root_set.contains (
						& file_data.root_path.as_ref ().unwrap ().clone ())

				)

			) {

				num_ignored += 1;

				continue;

			} else if ! dedupe_map.contains_key (
				file_data.path.as_ref ()) {

				num_fresh += 1;

				continue;

			} else if (
				num_updated > 0
				&& size_deduped + file_data.size > arguments.dedupe_batch_size
			) {

				num_remaining += 1;

				continue;

			} else {

				let deduplicate_time =
					time::get_time ();

				let target_path =
					dedupe_map.get (
						file_data.path.as_ref (),
					).unwrap ().clone ();

				let success =
					if * target_path == * file_data.path {

					output.status_format (
						format_args! (
							"Defragment: {}",
							file_data.path.to_string_lossy ()));

					btrfs::defragment_file (
						file_data.path.to_path (),
						1,
						btrfs::CompressionType::Lzo,
						true,
					).is_ok ()

				} else {

					output.status_format (
						format_args! (
							"Deduplicate: {} -> {}",
							file_data.path.to_string_lossy (),
							target_path.to_string_lossy ()));

					btrfs::deduplicate_files_with_source (
						target_path.to_path (),
						& vec! [ file_data.path.to_path () ],
					).is_ok ()

				};

				dedupe_map.remove (
					file_data.path.as_ref ());

				file_data.extent_hash = ZERO_HASH;
				file_data.extent_hash_time = 0;

				file_data.defragment_time = 0;
				file_data.deduplicate_time = deduplicate_time.sec;

				size_deduped += file_data.size;

				if success {
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

		Ok (())

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

}

// ex: noet ts=4 filetype=rust
