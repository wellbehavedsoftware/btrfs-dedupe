use clap;

use std::fs;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::process;
use std::rc::Rc;

pub enum Command {
	Dedupe,
	PrintExtents,
}

pub struct Arguments {
	pub command: Command,
	pub database_path: Option <PathBuf>,
	pub minimum_file_size: u64,
	pub content_hash_batch_size: u64,
	pub extent_hash_batch_size: u64,
	pub dedupe_batch_size: u64,
	pub dedupe_sleep_time: u64,
	pub root_paths: Vec <Rc <PathBuf>>,
}

pub fn parse_arguments (
) -> Arguments {

	let application = (
		clap::App::new ("Btrfs Dedupe")

		.about (
			"Deduplicates identical files on BTRFS file systems.\n\nPlease \
			visit https://btrfs-dedupe.com for more information.")

		.author (
			"James Pharaoh <james@pharaoh.uk>")

		.subcommand (

			clap::SubCommand::with_name ("dedupe")
				.about ("Automatically runs all deduplication steps (default)")

			.arg (
				clap::Arg::with_name ("database")
					.long ("database")
					.value_name ("PATH")
					.help ("Database path to store metadata and hashes")
			)

			.arg (
				clap::Arg::with_name ("minimum-file-size")
					.long ("minimum-file-size")
					.value_name ("SIZE")
					.default_value ("1KiB")
					.help ("Minimum file size to consider for deduplication")
			)

			.arg (
				clap::Arg::with_name ("content-hash-batch-size")
					.long ("content-hash-batch-size")
					.value_name ("SIZE")
					.default_value ("2GiB")
					.help ("Amount of file contents data to hash before \
						writing database")
			)

			.arg (
				clap::Arg::with_name ("extent-hash-batch-size")
					.long ("extent-hash-batch-size")
					.value_name ("SIZE")
					.default_value ("8GiB")
					.help ("Amount of file extent data to hash before writing \
						database")
			)

			.arg (
				clap::Arg::with_name ("dedupe-batch-size")
					.long ("dedupe-batch-size")
					.value_name ("SIZE")
					.default_value ("64MiB")
					.help ("Amount of file data to deduplicate before writing \
						database and sleeping")
			)

			.arg (
				clap::Arg::with_name ("dedupe-sleep-time")
					.long ("dedupe-sleep-time")
					.value_name ("SECONDS")
					.default_value ("5")
					.help ("Amount of time to sleep between deduplication \
						batches")
			)

			.arg (
				clap::Arg::with_name ("root-path")
					.multiple (true)
					.value_name ("PATH")
					.help ("Root path to scan for files")
			)

		)

		.subcommand (

			clap::SubCommand::with_name ("print-extents")
				.about ("Prints file extent information for a given file")

			.arg (
				clap::Arg::with_name ("file-path")
					.multiple (true)
					.value_name ("PATH")
					.help ("Path to file")
			)

		)

	);

	let argument_matches =
		application.clone ().get_matches ();

	if let Some (dedupe_matches) =
		argument_matches.subcommand_matches ("dedupe") {

		let database_path =
			dedupe_matches.value_of_os (
				"database",
			).map (
				|os_value|

				PathBuf::from (
					os_value)

			);

		let content_hash_batch_size = (

			parse_size (
				dedupe_matches.value_of (
					"content-hash-batch-size",
				).unwrap ()
			)

		).map_err (
			|error|

			clap::Error {

				message:
					format! (
						"Can't parse --content-hash-batch-size: {}",
						error),

				kind:
					clap::ErrorKind::InvalidValue,

				info:
					None,

			}.exit ()

		).unwrap ();

		let extent_hash_batch_size = (

			parse_size (
				dedupe_matches.value_of (
					"extent-hash-batch-size",
				).unwrap ()
			)

		).map_err (
			|error|

			clap::Error {

				message:
					format! (
						"Can't parse --extent-hash-batch-size: {}",
						error),

				kind:
					clap::ErrorKind::InvalidValue,

				info:
					None,

			}.exit ()

		).unwrap ();

		let dedupe_batch_size = (

			parse_size (
				dedupe_matches.value_of (
					"dedupe-batch-size",
				).unwrap ()
			)

		).map_err (
			|error|

			clap::Error {

				message:
					format! (
						"Can't parse --dedupe-batch-size: {}",
						error),

				kind:
					clap::ErrorKind::InvalidValue,

				info:
					None,

			}.exit ()

		).unwrap ();

		let dedupe_sleep_time = (

			dedupe_matches.value_of (
				"dedupe-sleep-time",
			).unwrap ().parse::<u64> ()

		).map_err (
			|error|

			clap::Error {

				message:
					format! (
						"Can't parse --dedupe-sleep-time: {}",
						error),

				kind:
					clap::ErrorKind::InvalidValue,

				info:
					None,

			}.exit ()

		).unwrap ();

		let minimum_file_size = (

			parse_size (
				dedupe_matches.value_of (
					"minimum-file-size",
				).unwrap ()
			)

		).map_err (
			|error|

			clap::Error {

				message:
					format! (
						"Can't parse --minimum-file-size: {}",
						error),

				kind:
					clap::ErrorKind::InvalidValue,

				info:
					None,

			}.exit ()

		).unwrap ();

		let mut root_paths = (
			dedupe_matches.values_of_os (
				"root-path",
			)
		).map (
			|os_values|

			os_values.map (
				|os_value|

				Rc::new (
					fs::canonicalize (
						PathBuf::from (
							os_value),
					).unwrap ()
				)

			).collect ()

		).unwrap_or (
			Vec::new (),
		);

		root_paths.sort ();

		Arguments {
			command: Command::Dedupe,
			database_path: database_path,
			minimum_file_size: minimum_file_size,
			content_hash_batch_size: content_hash_batch_size,
			extent_hash_batch_size: extent_hash_batch_size,
			dedupe_batch_size: dedupe_batch_size,
			dedupe_sleep_time: dedupe_sleep_time,
			root_paths: root_paths,
		}

	} else if let Some (print_extents_matches) =
		argument_matches.subcommand_matches ("print-extents") {

		let paths = (
			print_extents_matches.values_of_os (
				"file-path",
			)
		).map (
			|os_values|

			os_values.map (
				|os_value|

				Rc::new (
					fs::canonicalize (
						PathBuf::from (
							os_value),
					).unwrap ()
				)

			).collect ()

		).unwrap_or (
			Vec::new (),
		);

		Arguments {
			command: Command::PrintExtents,
			database_path: None,
			minimum_file_size: 0,
			content_hash_batch_size: 0,
			extent_hash_batch_size: 0,
			dedupe_batch_size: 0,
			dedupe_sleep_time: 0,
			root_paths: paths,
		}

	} else {

		let stderr =
			io::stderr ();

		let mut stderr_lock =
			stderr.lock ();

		stderr_lock.write (
			b"\n");

		application.write_help (
			& mut stderr_lock);

		stderr_lock.write (
			b"\n\n");

		process::exit (0);

	}

}

pub fn parse_size (
	size_string: & str,
) -> Result <u64, String> {

	let (multiplier, suffix_length) =

		if size_string.ends_with ("KiB") {
			(1024, 3)
		} else if size_string.ends_with ("MiB") {
			(1024 * 1024, 3)
		} else if size_string.ends_with ("GiB") {
			(1024 * 1024 * 1024, 3)
		} else if size_string.ends_with ("TiB") {
			(1024 * 1024 * 1024 * 1024, 3)

		} else if size_string.ends_with ("KB") {
			(1000, 2)
		} else if size_string.ends_with ("MB") {
			(1000 * 1000, 2)
		} else if size_string.ends_with ("GB") {
			(1000 * 1000 * 1000, 2)
		} else if size_string.ends_with ("TB") {
			(1000 * 1000 * 1000 * 1000, 2)

		} else if size_string.ends_with ("K") {
			(1024, 1)
		} else if size_string.ends_with ("M") {
			(1024 * 1024, 1)
		} else if size_string.ends_with ("G") {
			(1024 * 1024 * 1024, 1)
		} else if size_string.ends_with ("T") {
			(1024 * 1024 * 1024 * 1024, 1)

		} else if size_string.ends_with ("B") {
			(1, 1)

		} else {

			return Err (
				"Units not specified or recognised".to_owned ());

		};

	let quantity_string =
		& size_string [
			0 ..
			size_string.len () - suffix_length];

	let quantity_integer =
		try! (
			quantity_string.parse::<u64> (
			).map_err (
				|_|
				"Unable to parse integer value".to_owned ()
			));

	Ok (
		multiplier * quantity_integer
	)

}

// ex: noet ts=4 filetype=rust
