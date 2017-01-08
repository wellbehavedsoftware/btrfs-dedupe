use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;
use std::rc::Rc;

use database::*;
use types::*;

pub struct FileDatabaseBuilder {
	path_pool: HashSet <PathRef>,
	file_data_ordered: Vec <FileData>,
	file_data_by_parent: HashMap <PathRef, Vec <usize>>,
}

impl FileDatabaseBuilder {

	pub fn new (
	) -> FileDatabaseBuilder {

		FileDatabaseBuilder {
			path_pool: HashSet::new (),
			file_data_ordered: Vec::new (),
			file_data_by_parent: HashMap::new (),
		}

	}

	pub fn get_path (
		& mut self,
		path: Rc <PathBuf>,
	) -> Rc <PathBuf> {

		{

			let path_pool =
				& mut self.path_pool;

			if ! path_pool.contains (
				& path) {

				path_pool.insert (
					path.clone ());

			}

		}

		self.path_pool.get (
			& path,
		).unwrap ().clone ()

	}

	pub fn insert (
		& mut self,
		file_data: FileData,
	) {

		if let Some (last_file_data) =
			self.file_data_ordered.last () {

			if file_data.path <= last_file_data.path {

				panic! (
					"Tried to insert {:?} after {:?}",
					file_data.path,
					last_file_data.path);

			}

		}

		let parent =
			Rc::new (
				PathBuf::from (
					file_data.path.parent ().unwrap ()));

		self.file_data_by_parent.entry (
			parent,
		).or_insert_with (
			|| Vec::new ()
		).push (
			self.file_data_ordered.len (),
		);

		self.file_data_ordered.push (
			file_data,
		);

	}

	pub fn find_root (
		& mut self,
		root_map: & mut HashMap <PathRef, Option <PathRef>>,
		file_path: PathRef,
	) -> Option <PathRef> {

		let mut search_path =
			Some (file_path.clone ());

		let mut new_paths: Vec <PathRef> =
			Vec::new ();

		while (
			search_path.is_some ()
			&& ! root_map.contains_key (
				search_path.as_ref ().unwrap ())
		) {

			new_paths.push (
				search_path.as_ref ().unwrap ().clone ());

			let search_parent =
				search_path.unwrap ().parent ().map (
					|search_parent|

					self.get_path (
						Rc::new (
							PathBuf::from (
								search_parent)))

				);

			if search_parent.is_none () {

				search_path = None;

				break;

			}

			search_path =
				search_parent;

		}

		let root_path =
			search_path.and_then (
				|search_path|

			root_map.get (
				& search_path,
			).unwrap ().clone ()

		);

		for new_path in new_paths.into_iter () {

			root_map.insert (
				new_path,
				root_path.clone ());

		}

		root_path

	}

	pub fn build (
		self,
	) -> FileDatabase {

		FileDatabase::new (
			self.file_data_ordered,
		)

	}

	pub fn len (& self) -> usize {
		self.file_data_ordered.len ()
	}

}

// ex: noet ts=4 filetype=rust
