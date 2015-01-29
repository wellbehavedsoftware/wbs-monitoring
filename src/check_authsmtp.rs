//Rust file
#![allow(unstable)]
extern crate getopts;
extern crate curl;

use getopts::{ optflag, reqopt, getopts, short_usage, usage, OptGroup };
use std::os;
use std::option::{ Option };
use curl::http;
use std::f64;


fn print_usage (program: &str, opts: &[OptGroup]) {
	println! ("{}", short_usage (program, opts));
}

fn print_help (program: &str, opts: &[OptGroup]) {
	println! ("{}", usage (program, opts));
}

struct Options {
	data_level: String,
	message_level: String,
	username: String,
	password: String,
}

fn parse_options () -> Option<Options> {

	let args: Vec<String> = os::args ();

	let program = args [0].clone ();

	let opts = &[

		optflag (
			"h",
			"help",
			"print this help menu"),

		reqopt (
			"d",
			"data-level",
			"maximum data level allowed",
			"<data-level>"),

		reqopt (
			"m",
			"message-level",
			"maximum messages level allowed",
			"<message-level>"),

		reqopt (
			"u",
			"username",
			"authsmtp api username",
			"<username>"),

		reqopt (
			"p",
			"password",
			"authsmtp api password",
			"<password>"),

	];

	let matches = match getopts (args.tail (), opts) {
		Ok (m) => { m }
		Err (_) => {
			print_usage (program.as_slice (), opts);
			os::set_exit_status(3);	
			return None;
		}
	};

	if matches.opt_present ("help") {
		print_help (program.as_slice (), opts);
		os::set_exit_status(3);	
		return None;
	}

	if ! matches.free.is_empty () {
		print_usage (program.as_slice (), opts);
		os::set_exit_status(3);	
		return None;
	}

	
	let data_level = matches.opt_str ("data-level").unwrap ();
	let message_level = matches.opt_str ("message-level").unwrap ();
	let username = matches.opt_str ("username").unwrap ();
	let password = matches.opt_str ("password").unwrap ();

	return Some (Options {
		data_level: data_level,
		message_level: message_level,
		username: username,
		password: password,
	});

}

fn get_authsmtp_data (username: &str, password: &str, messages_level: f64, data_level: f64) -> String {

	let url = "https://secure.authsmtp.com/restful/basic_user/".to_string() + username;
	let userpass = username.to_string() + ":" + password;
	
   	let resp = http::handle()
		  	.connect_timeout(30000)
			.ssl_verifypeer(false)
			.follow_location(1)
			.userpwd(userpass.as_slice())
		  	.get(url)
		  	.exec().unwrap();

	let url_code = String::from_utf8_lossy(resp.get_body());

	let response_lines: Vec<&str> = url_code.as_slice().split_str("\n").collect();
	
	//Messages limit
	let mut messages_limit_array: Vec<&str> = response_lines[7].split('>').collect();
	messages_limit_array = messages_limit_array[1].split('<').collect();
	let messages_limit_str = messages_limit_array[0];

	let messages_limit_aux: Option<f64> = messages_limit_str.parse();
	if messages_limit_aux.is_none() { return "AUTHSMTP ERROR".to_string(); }
	let messages_limit: f64 = messages_limit_aux.unwrap();

	//Messages sent
	let mut messages_sent_array: Vec<&str> = response_lines[9].split('>').collect();
	messages_sent_array = messages_sent_array[1].split('<').collect();
	let messages_sent_str = messages_sent_array[0];

	let messages_sent_aux: Option<f64> = messages_sent_str.parse();
	if messages_sent_aux.is_none() { return "AUTHSMTP ERROR".to_string(); }
	let messages_sent: f64 = messages_sent_aux.unwrap();

	//Data limit
	let mut data_limit_array: Vec<&str> = response_lines[8].split('>').collect();
	data_limit_array = data_limit_array[1].split('<').collect();
	let data_limit_str = data_limit_array[0];

	let data_limit_aux: Option<f64> = data_limit_str.parse();
	if data_limit_aux.is_none() { return "AUTHSMTP ERROR".to_string(); }
	let data_limit: f64 = data_limit_aux.unwrap();

	//Data sent
	let mut data_sent_array: Vec<&str> = response_lines[10].split('>').collect();
	data_sent_array = data_sent_array[1].split('<').collect();
	let data_sent_str = data_sent_array[0];

	let data_sent_aux: Option<f64> = data_sent_str.parse();
	if data_sent_aux.is_none() { return "AUTHSMTP ERROR".to_string(); }
	let data_sent: f64 = data_sent_aux.unwrap();

	//From address
	let mut from_address_array: Vec<&str> = response_lines[14].split('>').collect();
	from_address_array = from_address_array[1].split('<').collect();
	let from_address_str = from_address_array[0];

	let from_address_aux: Option<f64> = from_address_str.parse();
	if from_address_aux.is_none() { return "AUTHSMTP ERROR".to_string(); }
	let from_address: f64 = from_address_aux.unwrap();

	//From address used
	let mut from_address_used_array: Vec<&str> = response_lines[15].split('>').collect();
	from_address_used_array = from_address_used_array[1].split('<').collect();
	let from_address_used_str = from_address_used_array[0];

	let from_address_used_aux: Option<f64> = from_address_used_str.parse();
	if from_address_used_aux.is_none() { return "AUTHSMTP ERROR".to_string(); }
	let from_address_used: f64 = from_address_used_aux.unwrap();
	
	let messages_percentage = messages_sent / messages_limit;
	let messages_percentage_format = f64::to_str_exact(messages_percentage * 100.0, 2);

	let data_percentage = data_sent / data_limit;
	let data_percentage_format = f64::to_str_exact(data_percentage * 100.0, 2);

	let messages_level_format = f64::to_str_exact(messages_level * 100.0, 2);
	let data_level_format = f64::to_str_exact(data_level * 100.0, 2);
	
	let mut message: String = "".to_string();
	let mut messages_msg: String = format!("AUTHSMTP-OK: Messages quota OK. {}% out of {}%.\n", messages_percentage_format, messages_level_format);  

	let mut data_msg: String = format!("AUTHSMTP-OK: Data quota OK. {}% out of {}%.\n", data_percentage_format, data_level_format); 
	let mut data_bool: bool = true;

	let mut address_msg: String = "AUTHSMTP-OK: From address quota OK.\n".to_string(); 
	let mut address_bool: bool = true;

	if messages_percentage >= messages_level { 
		messages_msg = format!("AUTHSMTP-CRITICAL: Messages limit reached. {}% out of {}%.\n", messages_percentage_format, messages_level_format); 
	}
	if data_percentage >= data_level { 
		data_msg = format!("AUTHSMTP-CRITICAL: Data limit reached. {}% out of {}%.\n", data_percentage_format, data_level_format);
		data_bool = false;	
	}
	if from_address_used == from_address { 
		address_msg = "AUTHSMTP-CRITICAL: From address limit reached.\n".to_string(); 
		address_bool = false;
	}

	
	if !data_bool { message = message + data_msg.as_slice() + messages_msg.as_slice() + address_msg.as_slice(); }
	else if !address_bool { message = message + address_msg.as_slice() + messages_msg.as_slice() + data_msg.as_slice() }	
	else { message = message + messages_msg.as_slice() + data_msg.as_slice() + address_msg.as_slice(); }

	return message;

}

fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => { return }
	};

	let data_level_aux: Option<f64> = opts.data_level.as_slice().parse();
	if data_level_aux.is_none() { 		
		os::set_exit_status(3);
		println!("UNKNOWN: data_level has an incorrect type (0.0 - 1.0)."); 	
		return;
	}
	let data_level: f64 = data_level_aux.unwrap();

	let messages_level_aux: Option<f64> = opts.message_level.as_slice().parse();
	if messages_level_aux.is_none() { 		
		os::set_exit_status(3);
		println!("UNKNOWN: message_level has an incorrect type (0.0 - 1.0)."); 
		return;
	}
	let messages_level: f64 = messages_level_aux.unwrap();

	let response = get_authsmtp_data (opts.username.as_slice(), opts.password.as_slice(), messages_level, data_level);		

	if response.contains("AUTHSMTP ERROR") {
		os::set_exit_status(3);
		println!("{}", response);
	}
	else if response.contains("OK") {
		os::set_exit_status(0);
		println!("{}", response);
	}
	else if response.contains("CRITICAL") {
		os::set_exit_status(2);
		println!("{}", response);
	}
	else {
		os::set_exit_status(3);
		println!("UNKNOWN: Check_authsmtp failed.");
	}

	return;
}



