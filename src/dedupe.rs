use std::collections::HashMap;
use std::collections::HashSet;
use std::thread;
use std::time::Duration;

use btrfs;

use output::Output;

use arguments::*;
use content::*;
use extent::*;
use scan::*;
use storage::*;
use types::*;

pub struct FileDeduper {
	pub num_ignored: u64,
	pub num_fresh: u64,
	pub num_updated: u64,
	pub num_remaining: u64,
	pub num_errors: u64,
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
		arguments: & Arguments,
		output: & mut Output,
		previous_database: FileDatabase,
		dedupe_map: & mut HashMap <PathRef, PathRef>,
	) -> Result <FileDatabase, String> {

		let mut new_database: FileDatabase =
			FileDatabase::new ();

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

			} else if ! dedupe_map.contains_key (file_data.path.as_ref ()) {

				new_database.insert_direct (
					file_data);

				num_fresh += 1;

			} else if (
				num_updated > 0
				&& size_deduped + file_data.size > arguments.dedupe_batch_size
			) {

				new_database.insert_direct (
					file_data);

				num_remaining += 1;

			} else {

				let target_path =
					dedupe_map.get (
						file_data.path.as_ref (),
					).unwrap ().clone ();

				let success =
					if * target_path == * file_data.path {

					output.status (
						& format! (
							"Defragment: {:?} (TODO)",
							file_data.path));

					true

				} else {

					output.status (
						& format! (
							"Deduplicate: {:?} -> {:?}",
							file_data.path,
							target_path));

					btrfs::deduplicate_files_with_source (
						target_path.as_ref (),
						& vec! [ file_data.path.as_ref () ],
					).is_ok ()

				};

				dedupe_map.remove (
					file_data.path.as_ref ());

				new_database.insert_without_extent_hash (
					file_data.as_ref (),
					file_data.root_path.as_ref ().unwrap ().clone ());

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

		Ok (new_database)

	}

}

fn build_dedupe_map (
	arguments: & Arguments,
	output: & mut Output,
	previous_database: & FileDatabase,
) -> HashMap <PathRef, PathRef> {

	// find all unique files

	let deduplication_candidates =
		group_deduplication_candidates (
			arguments,
			& previous_database);

	let unique_hash_count =
		deduplication_candidates.len ();

	output.message (
		& format! (
			"Found {} unique hashes",
			unique_hash_count));

	// filter to duplicated files

	let deduplication_candidates: HashMap <Hash, Vec <FileDataRef>> =
		deduplication_candidates.into_iter ().filter (
			|& (ref _hash, ref file_datas)|

			file_datas.len () > 1

		).collect ();

	let duplicated_file_count =
		deduplication_candidates.len ();

	output.message (
		& format! (
			"Found {} unique hashes with multiple instances",
			duplicated_file_count));

	// filter to files with physical extents

	let deduplication_candidates: HashMap <Hash, Vec <FileDataRef>> =
		deduplication_candidates.into_iter ().map (
			|(content_hash, file_datas)|

		(
			content_hash,

			file_datas.into_iter ().filter (
				|file_data|

				file_data.extent_hash != ZERO_HASH

			).collect ()
		)

	).filter (
		|& (ref _hash, ref file_datas): & (Hash, Vec <FileDataRef>)|

		file_datas.len () > 1

	).collect ();

	let physical_duplicated_file_count =
		deduplication_candidates.len ();

	output.message (
		& format! (
			"Found {} unique hashes which can be deduplicated",
			physical_duplicated_file_count));

	// filter to files which are not deduplicated

	let deduplication_candidates: HashMap <Hash, Vec <FileDataRef>> =
		deduplication_candidates.into_iter ().filter (
			|& (ref _hash, ref file_datas)| {

		let first_file_data =
			file_datas [0].clone ();

		let first_extent_hash =
			first_file_data.extent_hash;

		file_datas.iter ().any (
			|file_data|

			file_data.extent_hash != first_extent_hash

		)

	}).collect ();

	let physical_not_deduplicated_file_count =
		deduplication_candidates.len ();

	output.message (
		& format! (
			"Found {} unique hashes which need deduplication",
			physical_not_deduplicated_file_count));

	// work out what to deduplicate

	let dedupe_map: HashMap <PathRef, PathRef> =
		deduplication_candidates.into_iter ().flat_map (
			|(_hash, file_datas)| {

		let first_file_data =
			file_datas [0].clone ();

		let first_file_path =
			first_file_data.path.clone ();

		file_datas.into_iter ().map (
			move |file_data|

			(
				file_data.path.clone (),
				first_file_path.clone (),
			)

		)

	}).collect ();

	output.message (
		& format! (
			"Found {} files to deduplicate",
			dedupe_map.len ()));

	// return

	dedupe_map

}

fn group_deduplication_candidates (
	arguments: & Arguments,
	file_database: & FileDatabase,
) -> HashMap <Hash, Vec <FileDataRef>> {

	let mut identical_files =
		HashMap::new ();

	let root_set: HashSet <PathRef> =
		arguments.root_paths.iter ().map (
			|root_path|
			root_path.clone ()
		).collect ();

	for file_data in file_database.iter () {

		if file_data.size < arguments.minimum_file_size {
			continue;
		}

		if (

			(

				file_data.root_path.is_none ()

			) || (

				file_data.root_path.is_some ()

				&& ! root_set.contains (
					& file_data.root_path.as_ref ().unwrap ().clone ())

			)

		) {
			continue;
		}

		let mut file_datas =
			identical_files.entry (
				file_data.content_hash,
			).or_insert_with (
				||

				Vec::new (),

			);

		file_datas.push (
			file_data.clone ());

	}

	identical_files

}

fn perform_deduplication (
	arguments: & Arguments,
	output: & mut Output,
	file_database: FileDatabase,
) -> Result <FileDatabase, String> {

	let mut dedupe_map =
		build_dedupe_map (
			arguments,
			output,
			& file_database);

	let mut file_database =
		file_database;

	let mut file_deduper =
		FileDeduper::new ();

	loop {

		// calculate a batch of hashes

		file_database =
			try! (
				file_deduper.dedupe_files (
					arguments,
					output,
					file_database,
					& mut dedupe_map));

		if file_deduper.num_remaining == 0 {
			break;
		}

		output.message (
			& format! (
				"Deduped {} out of {} files, {} remaining",
				file_deduper.num_updated
					+ file_deduper.num_errors,
				file_deduper.num_updated
					+ file_deduper.num_errors
					+ file_deduper.num_remaining,
				file_deduper.num_remaining));

		output.status (
			& format! (
				"Sleeping for {} seconds",
				arguments.dedupe_sleep_time));

		thread::sleep (
			Duration::from_secs (
				arguments.dedupe_sleep_time));

		output.clear_status ();

		// write out updated database

		try! (
			write_database (
				& arguments,
				output,
				& file_database));

	}

	output.message (
		& format! (
			"Deduped {} files with {} errors",
			file_deduper.num_updated,
			file_deduper.num_errors));

	// write out updated database

	if file_deduper.num_updated > 0 {

		try! (
			write_database (
				& arguments,
				output,
				& file_database));

	}

	Ok (file_database)

}

pub fn dedupe_command (
	arguments: & Arguments,
	output: & mut Output,
) -> Result <(), String> {

	// load existing database

	let mut file_database =
		try! (
			init_database (
				arguments,
				output));

	// create a list of all files with the same name and size

	file_database =
		try! (
			scan_directories (
				arguments,
				output,
				file_database));

	// write out updated database

	try! (
		write_database (
			& arguments,
			output,
			& file_database));

	// calculate content hashes

	file_database =
		try! (
			calculate_content_hashes (
				arguments,
				output,
				file_database));

	// calculate extent hashes

	file_database =
		try! (
			calculate_extent_hashes (
				arguments,
				output,
				file_database));

	// perform deduplication

	try! (
		perform_deduplication (
			arguments,
			output,
			file_database));

	// return

	Ok (())

}

// ex: noet ts=4 filetype=rust
