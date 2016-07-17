// ###################################################################
// ##########################    WARNING    ##########################
// ###################################################################
// ##                                                               ##
// ##  This file is generated, please do not edit it directly.      ##
// ##  Instead, update the data, templates and code in              ##
// ##  scripts/generate-code and run that script.                   ##
// ##                                                               ##
// ###################################################################

extern crate wbsmonitoring;

use wbsmonitoring::checks;
use wbsmonitoring::logic;

fn main () {

	let plugin_provider =
		checks::btrfs::new ();

	logic::run_from_command_line (
		& * plugin_provider);

}

// ex: noet ts=4 filetype=rust
