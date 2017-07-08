#[ derive (Clone, Copy, Debug) ]
pub struct HttpConnection <'a> {

	pub address: & 'a str,
	pub hostname: & 'a str,
	pub port: u64,
	pub secure: bool,

}

// ex: noet ts=4 filetype=rust
