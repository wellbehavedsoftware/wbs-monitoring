use logic::plugin_provider::*;

#[ derive (Clone, Copy, Debug) ]
pub enum CheckStatus {
	Ok,
	Warning,
	Critical,
	Unknown,
}

impl CheckStatus {

	pub fn default_message (
		& self,
	) -> & str {

		match * self {

			CheckStatus::Ok =>
				"no problems detected",

			CheckStatus::Warning =>
				"minor problems detected",

			CheckStatus::Critical =>
				"major problems detected",

			CheckStatus::Unknown =>
				"unable to perform check",

		}

	}

	pub fn prefix (
		& self,
	) -> & str {

		match * self {

			CheckStatus::Ok =>
				"OK",

			CheckStatus::Warning =>
				"WARNING",

			CheckStatus::Critical =>
				"CRITICAL",

			CheckStatus::Unknown =>
				"UNKNOWN",

		}

	}

	pub fn update (
		& mut self,
		new_status: CheckStatus,
	) {

		* self = match (new_status, * self) {

			(CheckStatus::Critical, CheckStatus::Ok) |
			(CheckStatus::Critical, CheckStatus::Warning) |
			(CheckStatus::Critical, CheckStatus::Unknown) =>
				CheckStatus::Critical,

			(CheckStatus::Warning, CheckStatus::Ok) |
			(CheckStatus::Warning, CheckStatus::Unknown) =>
				CheckStatus::Warning,

			(CheckStatus::Unknown, CheckStatus::Ok) =>
				CheckStatus::Unknown,

			_ =>
				* self,

		};

	}

}

impl From <CheckStatus> for i32 {

	fn from (
		check_status: CheckStatus,
	) -> i32 {

		match check_status {
			CheckStatus::Ok => 0,
			CheckStatus::Warning => 1,
			CheckStatus::Critical => 2,
			CheckStatus::Unknown => 3,
		}

	}

}

pub struct CheckResult {
	status: CheckStatus,
	prefix: String,
	status_message: String,
	status_messages: Vec <String>,
	performance_data: Vec <String>,
	extra_information: Vec <String>,
}

impl CheckResult {

	// constructor

	pub fn new (
		status: CheckStatus,
		prefix: String,
		status_messages: Vec <String>,
		performance_data: Vec <String>,
		extra_information: Vec <String>,
	) -> CheckResult {

		CheckResult {

			status:
				status,

			prefix:
				prefix,

			status_message:
				if status_messages.is_empty () {
					status.default_message ().to_string ()
				} else {
					status_messages.join (", ")
				},

			status_messages:
				status_messages,

			performance_data:
				performance_data,

			extra_information:
				extra_information,

		}

	}

	// accessors

	pub fn status (
		& self,
	) -> & CheckStatus {
		& self.status
	}

	pub fn prefix (
		& self,
	) -> & str {
		self.prefix.as_str ()
	}

	pub fn status_message (
		& self,
	) -> & str {
		self.status_message.as_str ()
	}

	pub fn status_messages (
		& self,
	) -> & Vec <String> {
		& self.status_messages
	}

	pub fn performance_data (
		& self,
	) -> & Vec <String> {
		& self.performance_data
	}

	pub fn extra_information (
		& self,
	) -> & Vec <String> {
		& self.extra_information
	}

}

pub struct CheckResultBuilder {
	status: CheckStatus,
	status_messages: Vec <String>,
	performance_data: Vec <String>,
	extra_information: Vec <String>,
}

impl CheckResultBuilder {

	// constructor

	pub fn new (
	) -> CheckResultBuilder {

		CheckResultBuilder {
			status: CheckStatus::Ok,
			status_messages: vec! [],
			performance_data: vec! [],
			extra_information: vec! [],
		}

	}

	// updates

	pub fn ok <IntoString: Into <String>> (
		& mut self,
		message: IntoString,
	) {

		self.status_messages.push (
			message.into ());

	}

	pub fn unknown <IntoString: Into <String>> (
		& mut self,
		message: IntoString,
	) {

		self.status_messages.push (
			message.into ());

		self.status.update (
			CheckStatus::Unknown);

	}

	pub fn warning <IntoString: Into <String>> (
		& mut self,
		message: IntoString,
	) {

		self.status_messages.push (
			message.into ());

		self.status.update (
			CheckStatus::Warning);

	}

	pub fn critical <IntoString: Into <String>> (
		& mut self,
		message: IntoString,
	) {

		self.status_messages.push (
			message.into ());

		self.status.update (
			CheckStatus::Critical);

	}

	pub fn extra_information <IntoString: Into <String>> (
		& mut self,
		information_temp: IntoString,
	) {

		let information =
			information_temp.into ();

		for information_line in information.trim ().split ("\n") {

			self.extra_information.push (
				information_line.to_string ());

		}

	}

	pub fn update_status (
		& mut self,
		new_status: CheckStatus,
	) {

		self.status.update (
			new_status);

	}

	// transformer

	pub fn into_check_result (
		self,
		plugin_provider: & PluginProvider,
	) -> CheckResult {

		CheckResult {

			status:
				self.status.clone (),

			prefix:
				plugin_provider.prefix ().to_string (),

			status_message:
				if self.status_messages.is_empty () {

					match self.status {

						CheckStatus::Ok =>
							"no problems detected".to_string (),

						CheckStatus::Warning =>
							"minor problems detected".to_string (),

						CheckStatus::Critical =>
							"major problems detected".to_string (),

						CheckStatus::Unknown =>
							"unable to perform check".to_string (),

					}

				} else {

					self.status_messages.join (", ")

				},

			status_messages:
				self.status_messages,

			performance_data:
				self.performance_data,

			extra_information:
				self.extra_information,

		}

	}

}

// ex: noet ts=4 filetype=rust
