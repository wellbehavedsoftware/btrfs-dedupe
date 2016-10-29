use std::cmp;
use std::io;
use std::io::Stdout;

use termion;
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;

pub struct Output {
	terminal: Option <RawTerminal <Stdout>>,
	columns: u64,
	status: Option <String>,
}

impl Output {

	pub fn new (
	) -> Output {

		let mut terminal =
			match io::stdout ().into_raw_mode () {

			Ok (terminal) =>
				terminal,

			Err (_) =>
				return Output {
					terminal: None,
					columns: 99999,
					status: None,
				},

		};

		let columns: u64 =
			match termion::terminal_size () {

			Ok ((columns, _rows)) =>
				columns as u64,

			Err (_) => 80,

		};

		Output {
			terminal: Some (terminal),
			columns: columns,
			status: None,
		}

	}

	pub fn status (
		& mut self,
		status: & str,
	) {

		let status =
			& status [
				0 .. cmp::min (
					self.columns as usize,
					status.len (),
				)
			];

		if self.status.is_some () {

			stderr! (
				"\r\x1b[A\x1b[K");

		}

		self.status =
			Some (status.to_owned ());

		stderr! (
			"{}\r\n",
			status);

	}

	pub fn message (
		& mut self,
		message: & str,
	) {

		if self.status.is_some () {

			stderr! (
				"\r\x1b[A\x1b[K");

		}

		stderr! (
			"{}\r\n",
			message);

		if self.status.is_some () {

			stderr! (
				"{}\r\n",
				self.status.as_ref ().unwrap ());

		}

	}

	pub fn clear_status (
		& mut self,
	) {

		if self.status.is_some () {

			stderr! (
				"\r\x1b[A\x1b[K");

		}

		self.status =
			None;

	}

}

// ex: noet ts=4 filetype=rust
