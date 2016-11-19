extern crate serde_codegen;

use std::env;
use std::path::Path;

fn main () {
 
	let out_dir =
		env::var_os (
			"OUT_DIR"
		).unwrap ();

	let input =
		Path::new (
			"src/serde_types.in.rs");

	let output =
		Path::new (
			& out_dir,
		).join (
			"serde_types.rs",
		);

	serde_codegen::expand (
		& input,
		& output,)
	.unwrap ();

}
