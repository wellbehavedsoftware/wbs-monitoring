use std::process;

fn main () {

	process::Command::new ("make")
		.current_dir ("libaptc")
		.output ()
		.unwrap ();

	println! (
		"cargo:rustc-link-search=native=libaptc");

}
