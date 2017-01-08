use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

pub type RecursivePathRef = Rc <RecursivePath>;

#[ derive (Clone, Eq, Hash, PartialEq) ]
pub struct RecursivePath {
	parent: Option <RecursivePathRef>,
	name: Option <OsString>,
	depth: u16,
}

pub struct RecursivePathDatabase {
	paths: HashSet <RecursivePathRef>,
}

impl RecursivePath {

	pub fn parent (
		& self,
	) -> Option <RecursivePathRef> {
		self.parent.clone ()
	}

	pub fn name (
		& self,
	) -> Option <& OsStr> {

		self.name.as_ref ().map (
			|name|

			name.as_os_str ()

		)

	}

	pub fn to_path (
		& self,
	) -> PathBuf {

		match self.parent {

			Some (ref parent) =>
				match self.name {

				Some (ref name) =>
					parent.to_path ().join (
						& name),

				None =>
					parent.to_path (),

			},

			None =>
				match self.name {

				Some (ref name) =>
					PathBuf::from ("/").join (
						& name),

				None =>
					PathBuf::from ("/"),

			},


		}

	}

	pub fn to_string_lossy (
		& self,
	) -> String {

		self.to_path ().to_string_lossy ().into_owned ()

	}

}

impl Debug for RecursivePath {

	fn fmt (
		& self,
		formatter: & mut Formatter,
	) -> Result <(), fmt::Error> {

		self.to_path ().fmt (
			formatter,
		)

	}

}

impl Ord for RecursivePath {

	fn cmp (
		& self,
		other: & RecursivePath,
	) -> Ordering {

		if self == other {

			Ordering::Equal

		} else if self.depth == other.depth {

			let parent_order =
				self.parent.cmp (
					& other.parent);

			if parent_order != Ordering::Equal {

				parent_order

			} else if self.name == other.name {

				Ordering::Equal

			} else if self.name.is_none () {

				Ordering::Less

			} else if other.name.is_none () {

				Ordering::Greater

			} else {

				self.name.cmp (
					& other.name)

			}

		} else if self.depth > other.depth {

			self.parent.as_ref ().unwrap ().as_ref ().cmp (
				other)

		} else {

			self.cmp (
				& other.parent.as_ref ().unwrap ())

		}

	}

}

impl PartialOrd for RecursivePath {

	fn partial_cmp (
		& self,
		other: & RecursivePath,
	) -> Option <Ordering> {

		Some (
			self.cmp (
				other),
		)

	}

}

impl RecursivePathDatabase {

	pub fn new (
	) -> RecursivePathDatabase {

		RecursivePathDatabase {
			paths: HashSet::new (),
		}

	}

	pub fn for_path <
		SourcePathRef: AsRef <Path>,
	> (
		& mut self,
		source_path: SourcePathRef,
	) -> Option <RecursivePathRef> {

		let source_path =
			source_path.as_ref ();

		if ! source_path.is_absolute () {
			println! (
				"Not absolute: {:?}",
				source_path);
			return None;
		}

		let parent =
			source_path.parent ().map (
				|parent_path|

				self.for_path (
					parent_path,
				).unwrap ()

			);

		let name =
			source_path.file_name ().map (
				|name|

				name.to_owned ()

			);

		let depth =
			parent.as_ref ().map (
				|parent|
				parent.depth + 1,
			).unwrap_or (
				0,
			);

		let recursive_path =
			Rc::new (
				RecursivePath {

			parent: parent,
			name: name,
			depth: depth,

		});

		if let Some (existing_path) =
			self.paths.get (
				& recursive_path) {

			return Some (
				existing_path.clone (),
			);

		}

		self.paths.insert (
			recursive_path.clone ());

		Some (
			self.paths.get (
				& recursive_path,
			).unwrap ().clone ()
		)

	}

}

// ex: noet ts=4 filetype=rust
