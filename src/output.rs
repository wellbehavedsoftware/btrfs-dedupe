use std::cmp;
use std::env;

pub struct Output {
	status: Option <String>,
	columns: u64,
}

impl Output {

	pub fn new (
	) -> Output {

		let columns: u64 =
			env::var_os (
				"COLUMNS",
			).and_then (
				|string_value|

				string_value.to_string_lossy ().parse ().ok ()

			).unwrap_or (
				80
			);

		Output {
			status: None,
			columns: columns,
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
				"\x1b[A\x1b[K");

		}

		self.status =
			Some (status.to_owned ());

		stderrln! (
			"{}",
			status);

	}

	pub fn message (
		& mut self,
		message: & str,
	) {

		if self.status.is_some () {

			stderr! (
				"\x1b[A\x1b[K");

		}

		stderrln! (
			"{}",
			message);

		if self.status.is_some () {

			stderrln! (
				"{}",
				self.status.as_ref ().unwrap ());

		}

	}

	pub fn clear_status (
		& mut self,
	) {

		if self.status.is_some () {

			stderr! (
				"\x1b[A\x1b[K");

		}

		self.status =
			None;

	}

}

// ex: noet ts=4 filetype=rust
