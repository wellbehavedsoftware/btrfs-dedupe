#![ allow (unused_parens) ]

#[ macro_use ]
extern crate clap;

#[ macro_use ]
extern crate output;

extern crate btrfs;
extern crate rustc_serialize;
extern crate serde_json;
extern crate sha2;
extern crate time;

#[ doc (hidden) ]
#[ macro_use ]
mod misc;

mod arguments;
mod content;
mod dedupe;
mod extent;
mod scan;
mod serde_types;
mod storage;
mod types;

use std::process;

use output::Output;
use output::RawConsole;

use arguments::*;
use dedupe::*;
use extent::*;

fn main () {

	// process arguments

	let arguments =
		parse_arguments ();

	let mut output =
		RawConsole::new ().unwrap ();

	// delegate to command

	let command_result =
		match arguments.command {

		Command::Dedupe =>
			dedupe_command (
				& arguments,
				& mut output),

		Command::PrintExtents =>
			print_extents_command (
				& arguments,
				& mut output),

	};

	match command_result {

		Ok (_) => (),

		Err (error_message) => {

			output.clear_status ();

			output.message (
				& format! (
					"Error: {}",
					error_message));

			process::exit (1);

		},

	}

}

// ex: noet ts=4 filetype=rust
