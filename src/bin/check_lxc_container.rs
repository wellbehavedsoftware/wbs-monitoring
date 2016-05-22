extern crate wbsmonitoring;

use wbsmonitoring::checks;
use wbsmonitoring::logic;

fn main () {

	let plugin_provider =
		checks::lxc_container::new ();

	logic::run_from_command_line (
		& * plugin_provider);

}
