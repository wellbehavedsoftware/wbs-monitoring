//Rust file
#![feature(env)]
#![feature(core)]
#![feature(path)]

extern crate getopts;
extern crate git2;

use getopts::Options;
use std::env;
use std::option::{ Option };
use git2::{ Repository, StatusOptions };

fn print_usage (program: &str, opts: Options) {
	let brief = format!("Usage: {} [options]", program);
	println!("{}", opts.usage(brief.as_slice()));
}

fn print_help (program: &str, opts: Options) {
	let brief = format!("Help: {} [options]", program);
	println!("{}", opts.usage(brief.as_slice()));
}

struct Opts {
	local: String,
	remote: String,
}

fn parse_options () -> Option<Opts> {

	let args = env::args ();

	let mut opts = Options::new();

	opts.optflag (	
			"h",
			"help",
			"print this help menu");

	opts.reqopt (
			"l",
			"local",
			"folder in which the local repository is placed",
			"<local-repository>");

	opts.reqopt (
			"r",
			"remote",
			"remote repository ssh",
			"<remote-repository-ssh>");

	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("check_git", opts);
			return None;
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_git", opts);
		return None;
	}

	let local = matches.opt_str ("local").unwrap ();
	let remote = matches.opt_str ("remote").unwrap ();

	return Some (Opts {
		local: local,
		remote: remote,
	});

}

fn check_git_changes(local: &str, untracked: bool, submodules: bool, ignored: bool) -> String {

	let path = Path::new(local);
	let repo = match Repository::open(&path) {
	    Ok(repo) => repo,
	    Err(e) => panic!("failed to open `{}`: {}", path.display(), e),
	};

	let mut opts = StatusOptions::new();
	opts.include_untracked(untracked).recurse_untracked_dirs(untracked);
	opts.include_ignored(ignored);
	opts.exclude_submodules(!submodules);

	let statuses = match repo.statuses(Some(&mut opts)) {
	
		Ok ( st ) => { st },
		Err (e) => { return format!("CHECK GIT STATUS ERROR:\n {}", e).to_string(); }

	};
	
	let result = git_status(statuses);

	return result;
	
}

fn git_status(statuses: git2::Statuses) -> String {

	let mut header = false;
	let mut rm_in_workdir = false;
	let mut changes_in_index = false;
	let mut changed_in_workdir = false;

	let mut status: String = "".to_string();

	for entry in statuses.iter().filter(|e| e.status() != git2::STATUS_CURRENT) {
		if entry.status().contains(git2::STATUS_WT_DELETED) {
			rm_in_workdir = true;
		}

		let istatus = match entry.status() {
			s if s.contains(git2::STATUS_INDEX_NEW) => "new file: ",
			s if s.contains(git2::STATUS_INDEX_MODIFIED) => "modified: ",
			s if s.contains(git2::STATUS_INDEX_DELETED) => "deleted: ",
			s if s.contains(git2::STATUS_INDEX_RENAMED) => "renamed: ",
			s if s.contains(git2::STATUS_INDEX_TYPECHANGE) => "typechange:",
			_ => continue,
		};

		if !header {
			status = status + "Changes to be committed: (use \"git reset HEAD <file>...\" to unstage)\n";
			header = true;
		}

		let old_path = entry.head_to_index().unwrap().old_file().path();
		let new_path = entry.head_to_index().unwrap().new_file().path();

		match (old_path, new_path) {
			(Some(ref old), Some(ref new)) if old != new => {
				status = status + format!("->\t{} {} -> {}\n", istatus, old.display(), new.display()).as_slice();
				}
			(old, new) => {
				status = status + format!("->\t{} {}\n", istatus, old.or(new).unwrap().display()).as_slice();
			}
		}
	}

	if header {
		changes_in_index = true;
		status = status + "\n";
	}

	header = false;

	for entry in statuses.iter() {

		if entry.status() == git2::STATUS_CURRENT || entry.index_to_workdir().is_none() {
			continue
		}

		let istatus = match entry.status() {
			s if s.contains(git2::STATUS_WT_MODIFIED) => "modified: ",
			s if s.contains(git2::STATUS_WT_DELETED) => "deleted: ",
			s if s.contains(git2::STATUS_WT_RENAMED) => "renamed: ",
			s if s.contains(git2::STATUS_WT_TYPECHANGE) => "typechange:",
			_ => continue,
		};

		if !header {
			status = status + format!("Changes not staged for commit: (use \"git add{} <file>...\" to update what will be committed) (use \"git checkout -- <file>...\" to discard changes in working directory)\n", if rm_in_workdir {"/rm"} else {""}).as_slice();
			header = true;
		}

		let old_path = entry.index_to_workdir().unwrap().old_file().path();
		let new_path = entry.index_to_workdir().unwrap().new_file().path();

		match (old_path, new_path) {
			(Some(ref old), Some(ref new)) if old != new => {
				status = status + format!("->\t{} {} -> {}\n", istatus, old.display(), new.display()).as_slice();
			}
			(old, new) => {
				status = status + format!("->\t{} {}\n", istatus, old.or(new).unwrap().display()).as_slice();
			}
		}
	}

	if header { 
		changed_in_workdir = true;
		status = status + "\n";
	}

	header = false;

	for entry in statuses.iter().filter(|e| e.status() == git2::STATUS_WT_NEW) {
		if !header {
			status = status + "Untracked files (use \"git add <file>...\" to include in what will be committed)\n";
			header = true;
		}

		let file = entry.index_to_workdir().unwrap().old_file().path().unwrap();
		status = status + format!("->\t{}\n", file.display()).as_slice();
	}

	header = false;

	for entry in statuses.iter().filter(|e| e.status() == git2::STATUS_IGNORED) {

		if !header {
			status = status + "Ignored files (use \"git add -f <file>...\" to include in what will be committed)\n";
			header = true;
		}

		let file = entry.index_to_workdir().unwrap().old_file().path().unwrap();
		status = status + format!("->\t{}\n", file.display()).as_slice();
	}

	if !changes_in_index && changed_in_workdir {

		status = status + "no changes added to commit (use \"git add\" and/or \"git commit -a\")\n";
	}

	if header || rm_in_workdir || changes_in_index || changed_in_workdir {
		return status;
	}
	else {
		return "OK".to_string();
	}
}

/*fn check_git_sync(local: &str, remote: &str) -> String {

	let changes_output =
		match Command::new ("git")
			.arg ("-C".to_string ())
			.arg (local.to_string ())
			.arg ("fetch".to_string ())
			.arg (remote.to_string ())
			.arg ("--dry-run".to_string ())
			.output () {
		Ok (output) => { output }
		Err (_) => { return "GIT ERROR".to_string(); }
	};

	let changes = String::from_utf8_lossy(changes_output.output.as_slice()).to_string();
	
	return "CHANGES".to_string();
}

fn check_checkout(local: &str, compare_to: &str) -> String {

	let changes_output =
		match Command::new ("git")
			.arg ("diff".to_string ())
			.arg (local.to_string ())
			.arg (compare_to.to_string ())
			.output () {
		Ok (output) => { output }
		Err (_) => { return "GIT ERROR".to_string(); }
	};

	let changes = String::from_utf8_lossy(changes_output.output.as_slice()).to_string();
	
	return "CHANGES".to_string();

}*/

fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => { return }
	};

	let changes = check_git_changes(opts.local.as_slice(), true, false, false);

	if changes.contains("CHECK GIT STATUS ERROR") {
		println!("GIT-UNKNOWN: Could not git status check:\n{}.", changes); 
		env::set_exit_status(3);	
	}
	else if changes.contains("OK") {
		println!("GIT-OK: Git repo \"{}\" is up to date.\n", opts.local.as_slice()); 
		env::set_exit_status(0);	
	}
	else {
		println!("GIT-WARNING: Git repo \"{}\" has been modified:\n{}", opts.local.as_slice(), changes); 
		env::set_exit_status(1);
	}	
	
	return;
}

