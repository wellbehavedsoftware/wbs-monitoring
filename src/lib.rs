extern crate btrfs;
//extern crate dns_lookup;
extern crate getopts;
extern crate hyper;
extern crate hyper_native_tls;
extern crate itertools;
extern crate resolv;

#[ macro_use ]
pub mod logic;

#[ macro_use ]
extern crate serde_derive;

pub mod checks;
pub mod lowlevel;

// ex: noet ts=4 filetype=rust
