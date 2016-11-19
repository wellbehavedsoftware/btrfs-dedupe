use std::error::Error;
use std::io;

use rustc_serialize::hex::FromHex;

use types::*;

pub fn io_result <Type> (
	result: Result <Type, io::Error>,
) -> Result <Type, String> {

	result.map_err (
		|io_error|
		io_error.description ().to_string ()
	)

}

pub fn decode_hash (
	hash_option: & Option <String>,
) -> Hash {

	match * hash_option {

		None =>
			ZERO_HASH,

		Some (ref hash_string) => {

			let hash =
				hash_string.from_hex ().unwrap ();

			[
				hash [ 0], hash [ 1], hash [ 2], hash [ 3],
				hash [ 4], hash [ 5], hash [ 6], hash [ 7],
				hash [ 8], hash [ 9], hash [10], hash [11],
				hash [12], hash [13], hash [14], hash [15],
				hash [16], hash [17], hash [18], hash [19],
				hash [20], hash [21], hash [22], hash [23],
				hash [24], hash [25], hash [26], hash [27],
				hash [28], hash [29], hash [30], hash [21],
			]

		},

	}

}

// ex: noet ts=4 filetype=rust
