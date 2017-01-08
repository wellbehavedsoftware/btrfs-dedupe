use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;

use output::Output;

use arguments::*;
use database::*;
use operations::*;
use types::*;

pub fn dedupe_command (
	output: & Output,
	arguments: & Arguments,
) -> Result <(), String> {

	let mut recursive_path_database =
		RecursivePathDatabase::new ();

	// load existing database

	let mut file_database =
		read_database (
			output,
			arguments,
			& mut recursive_path_database,
		) ?;

	// scan filesystem

	file_database =
		scan_directories (
			output,
			arguments,
			& mut recursive_path_database,
			file_database,
		) ?;

	// write out updated database

	write_database (
		output,
		arguments,
		& mut file_database,
	) ?;

	// calculate content hashes

	calculate_content_hashes (
		output,
		arguments,
		& mut file_database,
	) ?;

	// calculate extent hashes

	calculate_extent_hashes (
		output,
		arguments,
		& mut file_database,
	) ?;

	// perform deduplication

	perform_deduplication (
		output,
		arguments,
		& mut file_database,
	) ?;

	// return

	Ok (())

}

fn read_database (
	output: & Output,
	arguments: & Arguments,
	recursive_path_database: & mut RecursivePathDatabase,
) -> Result <FileDatabase, String> {

	// if it doesn't exist just call new

	if ! (
		arguments.database_path.is_some ()
		&& arguments.database_path.as_ref ().unwrap ().exists ()
	) {

		return Ok (
			FileDatabaseBuilder::new ().build ()
		);

	}

	// read existing database

	output.message_format (
		format_args! (
			"Reading database from {}",
			arguments.database_path
				.as_ref ()
				.unwrap ()
				.to_string_lossy ()));

	let database_file = try! (

		File::open (
			arguments.database_path
				.as_ref ()
				.unwrap (),
		).map_err (
			|io_error|

			format! (
				"Error reading database: {}",
				io_error.description ())

		)

	);

	let mut database_reader = try! (

		GzDecoder::new (
			database_file,
		).map_err (
			|io_error|

			format! (
				"Error reading database: {}",
				io_error.description ())

		)

	);

	FileDatabase::read (
		recursive_path_database,
		& arguments.root_paths,
		& mut database_reader,
	).map_err (
		|error_string|

		format! (
			"Error reading database: {}",
			error_string)

	)

}

fn write_database (
	output: & Output,
	arguments: & Arguments,
	file_database: & FileDatabase,
) -> Result <(), String> {

	if arguments.database_path.is_none () {
		return Ok (());
	}

	let database_path =
		arguments.database_path.as_ref ().unwrap ();

	output.status_format (
		format_args! (
			"Writing database to {}",
			database_path.to_string_lossy ()));

	let database_path_temp_bytes: Vec <u8> =
		database_path.as_os_str ().as_bytes ().iter ().chain (
			b".temp".iter (),
		).map (
			|byte_ref|
			* byte_ref
		).collect ();

	let database_path_temp =
		PathBuf::from (
			OsStr::from_bytes (
				& database_path_temp_bytes));

	let database_file = try! (

		File::create (
			& database_path_temp,
		).map_err (
			|io_error|

			format! (
				"Error writing database: {}",
				io_error.description ())

		)

	);

	let mut database_writer =
		GzEncoder::new (
			database_file,
			Compression::Fast);

	try! (
		file_database.write (
			& mut database_writer));

	let database_file = try! (

		database_writer.finish (
		).map_err (
			|io_error|

			format! (
				"Error writing database: {}",
				io_error.description ())

		)

	);

	try! (

		database_file.sync_data (
		).map_err (
			|io_error|

			format! (
				"Error writing database: {}",
				io_error.description ())

		)

	);

	try! (
		fs::rename (
			& database_path_temp,
			database_path,
		).map_err (
			|io_error|

			format! (
				"Error writing database: {}",
				io_error.description ())

		)
	);

	output.clear_status ();

	Ok (())

}

fn scan_directories (
	output: & Output,
	arguments: & Arguments,
	recursive_path_database: & mut RecursivePathDatabase,
	file_database: FileDatabase,
) -> Result <FileDatabase, String> {

	let directory_scanner =
		DirectoryScanner::new (
			& arguments.root_paths,
			file_database,
		);

	Ok (
		directory_scanner.scan_directories (
			output,
			recursive_path_database,
		) ?
	)

}

fn calculate_content_hashes (
	output: & Output,
	arguments: & Arguments,
	file_database: & mut FileDatabase,
) -> Result <(), String> {

	let mut content_hasher =
		ContentHasher::new (
			& arguments.root_paths,
			arguments.content_hash_batch_size,
			file_database,
		);

	loop {

		// calculate a batch of hashes

		content_hasher.calculate_hashes (
			output,
		);

		if content_hasher.num_remaining () == 0 {
			break;
		}

		output.message_format (
			format_args! (
				"Hashed contents of {} out of {} files, {} remaining",
				content_hasher.num_processed (),
				content_hasher.num_to_process (),
				content_hasher.num_remaining ()));

		// write out updated database

		write_database (
			output,
			arguments,
			content_hasher.file_database (),
		) ?;

	}

	output.message_format (
		format_args! (
			"Hashed contents of {} files with {} errors, ignored {} with fresh \
			hashes",
			content_hasher.num_updated (),
			content_hasher.num_errors (),
			content_hasher.num_fresh ()));

	// write out updated database

	if content_hasher.num_updated () > 0 {

		write_database (
			output,
			arguments,
			& content_hasher.file_database (),
		) ?;

	}

	Ok (())

}

fn calculate_extent_hashes (
	output: & Output,
	arguments: & Arguments,
	file_database: & mut FileDatabase,
) -> Result <(), String> {

	let mut extent_hasher =
		ExtentHasher::new (
			& arguments.root_paths,
			arguments.extent_hash_batch_size,
			file_database);

	loop {

		// calculate a batch of hashes

		extent_hasher.calculate_hashes (
			output,
		);

		if extent_hasher.num_remaining () == 0 {
			break;
		}

		output.message_format (
			format_args! (
				"Hashed extents of {} out of {} files, {} remaining",
				extent_hasher.num_updated ()
					+ extent_hasher.num_errors (),
				extent_hasher.num_updated ()
					+ extent_hasher.num_errors ()
					+ extent_hasher.num_remaining (),
				extent_hasher.num_remaining ()));

		// write out updated database

		write_database (
			output,
			arguments,
			& extent_hasher.file_database (),
		) ?;

	}

	output.message_format (
		format_args! (
			"Hashed extents of {} files, {} errors, skipped {}",
			extent_hasher.num_updated (),
			extent_hasher.num_errors (),
			extent_hasher.num_fresh ()));

	// write out updated database

	if extent_hasher.num_updated () > 0 {

		write_database (
			output,
			arguments,
			extent_hasher.file_database (),
		) ?;

	}

	Ok (())

}

pub fn build_dedupe_map (
	output: & Output,
	arguments: & Arguments,
	file_database: & FileDatabase,
) -> HashMap <RecursivePathRef, RecursivePathRef> {

	// find all unique files

	let deduplication_candidates =
		group_deduplication_candidates (
			arguments,
			& file_database);

	let unique_hash_count =
		deduplication_candidates.len ();

	output.message_format (
		format_args! (
			"Found {} unique hashes",
			unique_hash_count));

	// filter to duplicated files

	let deduplication_candidates: HashMap <Hash, Vec <usize>> =
		deduplication_candidates.into_iter ().filter (
			|& (ref _hash, ref file_data_indices)|

			file_data_indices.len () > 1

		).collect ();

	let duplicated_file_count =
		deduplication_candidates.len ();

	output.message_format (
		format_args! (
			"Found {} unique hashes with multiple instances",
			duplicated_file_count));

	// filter to files with physical extents

	let deduplication_candidates: HashMap <Hash, Vec <usize>> =
		deduplication_candidates.into_iter ().map (
			|(content_hash, file_data_indices)|

		(
			content_hash,

			file_data_indices.into_iter ().filter (
				|& file_data_index|

				file_database [file_data_index].extent_hash
					!= ZERO_HASH

			).collect ()
		)

	).filter (
		|& (ref _hash, ref file_data_indices): & (Hash, Vec <usize>)|

		file_data_indices.len () > 1

	).collect ();

	let physical_duplicated_file_count =
		deduplication_candidates.len ();

	output.message_format (
		format_args! (
			"Found {} unique hashes which can be deduplicated",
			physical_duplicated_file_count));

	// filter to files which are not deduplicated

	let deduplication_candidates: HashMap <Hash, Vec <usize>> =
		deduplication_candidates.into_iter ().filter (
			|& (ref _hash, ref file_indices)| {

		let first_file_index =
			file_indices [0];

		let ref first_file_data =
			file_database [first_file_index];

		let first_extent_hash =
			first_file_data.extent_hash;

		file_indices.iter ().any (
			|file_index|

			file_database [* file_index].extent_hash
				!= first_extent_hash

		)

	}).collect ();

	let physical_not_deduplicated_file_count =
		deduplication_candidates.len ();

	output.message_format (
		format_args! (
			"Found {} unique hashes which need deduplication",
			physical_not_deduplicated_file_count));

	// work out what to deduplicate

	let dedupe_map: HashMap <RecursivePathRef, RecursivePathRef> =
		deduplication_candidates.into_iter ().flat_map (
			|(_hash, file_data_indices)| {

		let ref first_file_data =
			file_database [
				file_data_indices [0]];

		let first_file_path =
			first_file_data.path.clone ();

		file_data_indices.into_iter ().map (
			move |file_data_index|

			(
				file_database [file_data_index].path.clone (),
				first_file_path.clone (),
			)

		)

	}).collect ();

	output.message_format (
		format_args! (
			"Found {} files to deduplicate",
			dedupe_map.len ()));

	// return

	dedupe_map

}

fn group_deduplication_candidates (
	arguments: & Arguments,
	file_database: & FileDatabase,
) -> HashMap <Hash, Vec <usize>> {

	let mut identical_files_map =
		HashMap::new ();

	let root_set: HashSet <PathRef> =
		arguments.root_paths.iter ().map (
			|root_path|
			root_path.clone ()
		).collect ();

	for (file_index, file_data)
	in file_database.iter ().enumerate () {

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

		identical_files_map.entry (
			file_data.content_hash,
		).or_insert_with (
			|| Vec::new (),
		).push (
			file_index,
		);

	}

	identical_files_map

}

fn perform_deduplication (
	output: & Output,
	arguments: & Arguments,
	file_database: & mut FileDatabase,
) -> Result <(), String> {

	let mut dedupe_map =
		build_dedupe_map (
			output,
			arguments,
			& file_database);

	let mut file_database =
		file_database;

	let mut file_deduper =
		FileDeduper::new ();

	loop {

		// calculate a batch of hashes

		file_deduper.dedupe_files (
			output,
			arguments,
			file_database,
			& mut dedupe_map,
		) ?;

		if file_deduper.num_remaining () == 0 {
			break;
		}

		output.message_format (
			format_args! (
				"Deduped {} out of {} files, {} remaining",
				file_deduper.num_updated ()
					+ file_deduper.num_errors (),
				file_deduper.num_updated ()
					+ file_deduper.num_errors ()
					+ file_deduper.num_remaining (),
				file_deduper.num_remaining ()));

		// write out updated database

		write_database (
			output,
			arguments,
			& file_database,
		) ?;

	}

	output.message_format (
		format_args! (
			"Deduped {} files with {} errors, ignored {} already deduped",
			file_deduper.num_updated (),
			file_deduper.num_errors (),
			file_deduper.num_fresh ()));

	// write out updated database

	if file_deduper.num_updated () > 0 {

		write_database (
			output,
			arguments,
			& file_database,
		) ?;

	}

	Ok (())

}

// ex: noet ts=4 filetype=rust
