use std::error;
use std::fmt;

#[ derive (Debug) ]
pub struct SimpleError {
	description: String,
}

impl error::Error for SimpleError {

	fn description (
		& self,
	) -> & str {
		self.description.as_str ()
	}

	fn cause (
		& self,
	) -> Option <& error::Error> {
		None
	}

}

impl fmt::Display for SimpleError {

	fn fmt (
		& self,
		formatter: & mut fmt::Formatter,
	) -> Result<(), fmt::Error> {

		try! (
			formatter.write_str (
				self.description.as_str ()));

		Ok (())

	}

}

impl From <String> for SimpleError {

	fn from (
		description: String,
	) -> SimpleError {

		SimpleError {
			description: description,
		}

	}

}

impl <'a> From <& 'a str> for SimpleError {

	fn from (
		description: & 'a str,
	) -> SimpleError {

		SimpleError {
			description: description.to_string (),
		}

	}

}

// ex: noet ts=4 filetype=rust
