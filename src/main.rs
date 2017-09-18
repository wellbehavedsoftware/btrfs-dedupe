#![ allow (unused_parens) ]

extern crate clap;

extern crate output;

extern crate btrfs;
extern crate flate2;
extern crate rustc_serialize;
extern crate serde_json;
extern crate sha2;
extern crate time;

#[ doc (hidden) ]
#[ macro_use ]
mod misc;

mod commands;
mod arguments;
mod database;
mod operations;
mod types;

use std::process;

use arguments::*;
use commands::*;

fn main () {

	// process arguments

	let arguments =
		parse_arguments ();

	let exit_code =
		main_real (
			& arguments);

	process::exit (
		exit_code);

}

fn main_real (
	arguments: & Arguments,
) -> i32 {

	let output =
		output::open ();

	// delegate to command

	let command_result =
		match arguments.command {

		Command::Dedupe =>
			dedupe_command (
				& output,
				arguments,
			),

		Command::PrintExtents =>
			print_extents_command (
				& output,
				arguments,
			)

	};

	match command_result {

		Ok (_) =>
			0,

		Err (error_message) => {

			output.clear_status ();

			output.message_format (
				format_args! (
					"{}",
					error_message));

			1

		},

	}

}

// ex: noet ts=4 filetype=rust
