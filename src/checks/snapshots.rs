#![ allow (unused_parens) ]

extern crate getopts;
extern crate glob;
extern crate libc;
extern crate time;

use std::error;
use std::time::Duration;

use logic::*;
use logic::check_helper::*;

check! {

	new = new,
	name = "check-snapshots",
	prefix = "SNAPSHOTS",

	provider = CheckSnapshotsProvider,

	instance = CheckSnapshotsInstance {

		warning_time: Option <Duration>,
		critical_time: Option <Duration>,

		local_pattern: Option <String>,
		local_warning_time: Option <Duration>,
		local_critical_time: Option <Duration>,

		archive_pattern: Option <String>,
		archive_warning_time: Option <Duration>,
		archive_critical_time: Option <Duration>,

	},

	options_spec = |options_spec| {

		// overall

		options_spec.optopt (
			"",
			"warning",
			"maximum snapshot age before warning",
			"DURATION");

		options_spec.optopt (
			"",
			"critical",
			"maximum snapshot age before critical",
			"DURATION");

		// local

		options_spec.optopt (
			"",
			"local-pattern",
			"where to find local snapshots, with a {date} placeholer",
			"PATTERN");

		options_spec.optopt (
			"",
			"local-warning",
			"maximum local snapshot age before warning",
			"DURATION");

		options_spec.optopt (
			"",
			"local-critical",
			"maximum local snapshot age before critical",
			"DURATION");

		// archive

		options_spec.optopt (
			"",
			"archive-pattern",
			"where to find archive snapshots, with a {date} placeholer",
			"PATTERN");

		options_spec.optopt (
			"",
			"archive-warning",
			"maximum archive snapshot age before warning",
			"DURATION");

		options_spec.optopt (
			"",
			"archive-critical",
			"maximum archive snapshot age before critical",
			"DURATION");

	},

	options_parse = |options_matches| {

		// overall

		let warning_time =
			try! (
				arg_helper::parse_duration (
					options_matches,
					"warning"));

		let critical_time =
			try! (
				arg_helper::parse_duration (
					options_matches,
					"critical"));

		// local

		let local_pattern =
			options_matches.opt_str (
				"local-pattern");

		let local_warning_time =
			try! (
				arg_helper::parse_duration (
					options_matches,
					"local-warning"));

		let local_critical_time =
			try! (
				arg_helper::parse_duration (
					options_matches,
					"local-critical"));

		// archive

		let archive_pattern =
			options_matches.opt_str (
				"archive-pattern");

		let archive_warning_time =
			try! (
				arg_helper::parse_duration (
					options_matches,
					"archive-warning"));

		let archive_critical_time =
			try! (
				arg_helper::parse_duration (
					options_matches,
					"archive-critical"));

		CheckSnapshotsInstance {

			warning_time: warning_time,
			critical_time: critical_time,

			local_pattern: local_pattern,
			local_warning_time: local_warning_time,
			local_critical_time: local_critical_time,

			archive_pattern: archive_pattern,
			archive_warning_time: archive_warning_time,
			archive_critical_time: archive_critical_time,

		}

	},

	perform = |self, plugin_provider, check_result_builder| {

		let now =
			time::now ();

		// check for snapshots

		let most_recent_local_snapshot = if (
			self.local_pattern.is_some ()
		) {
			Self::most_recent_snapshot (
				self.local_pattern.as_ref ().unwrap ())
		} else {
			None
		};

		let most_recent_archive_snapshot = if (
			self.archive_pattern.is_some ()
		) {
			Self::most_recent_snapshot (
				self.archive_pattern.as_ref ().unwrap ())
		} else {
			None
		};

		let most_recent_snapshot = vec! [
			most_recent_local_snapshot,
			most_recent_archive_snapshot,
		].into_iter ().filter_map (
			|optional| optional
		).max ();

		// overall

		if most_recent_snapshot.is_some () {

			let most_recent_time = (
				now - most_recent_snapshot.unwrap ()
			).to_std ().unwrap ();

			check_duration_less_than (
				& mut check_result_builder,
				& self.warning_time,
				& self.critical_time,
				& format! (
					"snapshot on {}",
					time::strftime (
						"%Y-%m-%d",
						& most_recent_snapshot.unwrap (),
					).unwrap ()),
				& most_recent_time);

			// local

			if most_recent_local_snapshot.is_some () {

				let most_recent_local_time = (
					now - most_recent_local_snapshot.unwrap ()
				).to_std ().unwrap ();

				check_duration_less_than (
					& mut check_result_builder,
					& self.local_warning_time,
					& self.local_critical_time,
					& format! (
						"local snapshot on {}",
						time::strftime (
							"%Y-%m-%d",
							& most_recent_local_snapshot.unwrap (),
						).unwrap ()),
					& most_recent_local_time);

			} else if self.local_pattern.is_some () {

				if self.local_critical_time.is_some () {

					check_result_builder.critical (
						"no local snapshots (critical)");

				} else if self.local_warning_time.is_some () {

					check_result_builder.warning (
						"no local snapshots (warning)");

				} else {

					check_result_builder.ok (
						"no local snapshots");

				}

			}

			// archive

			if most_recent_archive_snapshot.is_some () {

				let most_recent_archive_time = (
					now - most_recent_archive_snapshot.unwrap ()
				).to_std ().unwrap ();

				check_duration_less_than (
					& mut check_result_builder,
					& self.archive_warning_time,
					& self.archive_critical_time,
					& format! (
						"archive snapshot on {}",
						time::strftime (
							"%Y-%m-%d",
							& most_recent_archive_snapshot.unwrap (),
						).unwrap ()),
					& most_recent_archive_time);

			} else if self.archive_pattern.is_some () {

				if self.archive_critical_time.is_some () {

					check_result_builder.critical (
						"no archive snapshots (critical)");

				} else if self.archive_warning_time.is_some () {

					check_result_builder.warning (
						"no archive snapshots (warning)");

				} else {

					check_result_builder.ok (
						"no archive snapshots");

				}

			}

		} else {

			if self.critical_time.is_some ()
			|| self.local_critical_time.is_some ()
			|| self.archive_critical_time.is_some () {

				check_result_builder.critical (
					"no snapshots (critical)");

			} else if self.warning_time.is_some ()
			|| self.local_warning_time.is_some ()
			|| self.archive_warning_time.is_some () {

				check_result_builder.warning (
					"no snapshots (warning)");

			} else {

				check_result_builder.ok (
					"no snapshots");

			}

		}

	},

}

impl CheckSnapshotsInstance {

	fn most_recent_snapshot (
		pattern: & str,
	) -> Option <time::Tm> {

		let mut pattern_parts =
			pattern.splitn (
				2,
				"{date}");

		let pattern_prefix =
			pattern_parts.next ().unwrap ();

		let pattern_suffix =
			pattern_parts.next ().unwrap ();

		let pattern_glob =
			format! (
				"{}*{}",
				pattern_prefix,
				pattern_suffix);

		let mut most_recent = None;

		for path_result in (
			glob::glob (
				& pattern_glob,
			).unwrap ()
		) {

			if let Ok (path) = path_result {

				if let Some (path_string) = path.to_str () {

					let date_string =
						& path_string [
							path_string.len ()
							- pattern_suffix.len ()
							- 10
						..
							path_string.len ()
							- pattern_suffix.len ()
						];

					if let Ok (time) = (
						time::strptime (
							date_string,
							"%Y-%m-%d")
					) {

						if let Some (most_recent_value) = most_recent {
							if time < most_recent_value {
								continue;
							}
						}

						most_recent =
							Some (time);

					}

				}

			}

		}

		most_recent

	}

}

// ex: noet ts=4 filetype=rust
