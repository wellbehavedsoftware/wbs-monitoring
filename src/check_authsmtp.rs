//Rust file
extern crate getopts;
extern crate curl;

use getopts::Options;
use std::env;
use std::process;
use std::option::{ Option };
use curl::http;

fn print_usage (program: &str, opts: Options) {
	let brief = format!("Usage: {} [options]", program);
	println!("{}", opts.usage(&brief));
}

fn print_help (program: &str, opts: Options) {
	let brief = format!("Help: {} [options]", program);
	println!("{}", opts.usage(&brief));
}

struct Opts {
	data_level: String,
	message_level: String,
	username: String,
	password: String,
}

fn parse_options () -> Option<Opts> {

	let args = env::args ();

	let mut opts = Options::new();

	opts.optflag (	
			"h", 
			"help", 
			"print this help menu");

	opts.reqopt (	
			"d", 
			"data-level", 
			"maximum data level allowed", 
			"<data-level>");

	opts.reqopt (	
			"m", 
			"message-level", 
			"maximum messages level allowed", 
			"<message-level>");

	opts.reqopt (	
			"u", 
			"username",
			"authsmtp api username",
			"<username>");

	opts.reqopt (
			"p",
			"password",
			"authsmtp api password",
			"<password>");

	

	let matches = match opts.parse (args) {
		Ok (m) => { m }
		Err (_) => {
			print_usage ("Check_authsmtp", opts);
			process::exit(3);	
		}
	};

	if matches.opt_present ("help") {
		print_help ("check_authsmtp", opts);
		process::exit(3);	
	}

	let data_level = matches.opt_str ("data-level").unwrap ();
	let message_level = matches.opt_str ("message-level").unwrap ();
	let username = matches.opt_str ("username").unwrap ();
	let password = matches.opt_str ("password").unwrap ();

	return Some (Opts {
		data_level: data_level,
		message_level: message_level,
		username: username,
		password: password,
	});

}

fn get_authsmtp_data (username: &str, password: &str, messages_level: f64, data_level: f64) -> (String, String, String) {

	let url = "https://secure.authsmtp.com/restful/basic_user/".to_string() + username;
	let userpass = username.to_string() + ":" + password;
	
   	let resp = http::handle()
		  	.connect_timeout(30000)
			.ssl_verifypeer(false)
			.follow_location(1)
			.userpwd(&userpass)
		  	.get(url)
		  	.exec().unwrap();

	let url_code = String::from_utf8_lossy(resp.get_body());

	let response_lines: Vec<&str> = url_code.split("\n").collect();
	
	//Messages limit
	let mut messages_limit_array: Vec<&str> = response_lines[7].split('>').collect();
	messages_limit_array = messages_limit_array[1].split('<').collect();
	let messages_limit_str = messages_limit_array[0];

	let messages_limit : f64 = match messages_limit_str.parse() {
		Ok (f64) => { f64 }
		Err (_) => { return ("AUTHSMTP ERROR".to_string(), "".to_string(), "".to_string()); }
	};

	//Messages sent
	let mut messages_sent_array: Vec<&str> = response_lines[9].split('>').collect();
	messages_sent_array = messages_sent_array[1].split('<').collect();
	let messages_sent_str = messages_sent_array[0];

	let messages_sent : f64 = match messages_sent_str.parse() {
		Ok (f64) => { f64 }
		Err (_) => { return ("AUTHSMTP ERROR".to_string(), "".to_string(), "".to_string()); }
	};

	//Data limit
	let mut data_limit_array: Vec<&str> = response_lines[8].split('>').collect();
	data_limit_array = data_limit_array[1].split('<').collect();
	let data_limit_str = data_limit_array[0];

	let data_limit : f64 = match data_limit_str.parse() {
		Ok (f64) => { f64 }
		Err (_) => { return ("AUTHSMTP ERROR".to_string(), "".to_string(), "".to_string()); }
	};

	//Data sent
	let mut data_sent_array: Vec<&str> = response_lines[10].split('>').collect();
	data_sent_array = data_sent_array[1].split('<').collect();
	let data_sent_str = data_sent_array[0];

	let data_sent : f64 = match data_sent_str.parse() {
		Ok (f64) => { f64 }
		Err (_) => { return ("AUTHSMTP ERROR".to_string(), "".to_string(), "".to_string()); }
	};

	//From address
	let mut from_address_array: Vec<&str> = response_lines[14].split('>').collect();
	from_address_array = from_address_array[1].split('<').collect();
	let from_address_str = from_address_array[0];

	let from_address : f64 = match from_address_str.parse() {
		Ok (f64) => { f64 }
		Err (_) => { return ("AUTHSMTP ERROR".to_string(), "".to_string(), "".to_string()); }
	};

	//From address used
	let mut from_address_used_array: Vec<&str> = response_lines[15].split('>').collect();
	from_address_used_array = from_address_used_array[1].split('<').collect();
	let from_address_used_str = from_address_used_array[0];

	let from_address_used : f64 = match from_address_used_str.parse() {
		Ok (f64) => { f64 }
		Err (_) => { return ("AUTHSMTP ERROR".to_string(), "".to_string(), "".to_string()); }
	};

	let messages_percentage = messages_sent / messages_limit;
	let messages_percentage_format = format!("{0:.1$}", messages_percentage * 100.0, 2);

	let data_percentage = data_sent / data_limit;
	let data_percentage_format = format!("{0:.1$}", data_percentage * 100.0, 2);

	let messages_level_format = format!("{0:.1$}", messages_level * 100.0, 2);
	let data_level_format = format!("{0:.1$}", data_level * 100.0, 2);
	
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

	
	if !data_bool { message = message + &data_msg + &messages_msg + &address_msg; }
	else if !address_bool { message = message + &address_msg + &messages_msg + &data_msg; }	
	else { message = message + &messages_msg + &data_msg + &address_msg; }

	return (message, messages_percentage_format, data_percentage_format);

}

fn main () {

	let opts = match parse_options () {
		Some (opts) => { opts }
		None => { return }
	};

	let data_level : f64 = match opts.data_level.parse() {
		Ok (f64) => { f64 }
		Err (_) => { 
			println!("UNKNOWN: data_level has an incorrect type (0.0 - 1.0)."); 	
			process::exit(3);
		}
	};

	let messages_level : f64 = match opts.message_level.parse() {
		Ok (f64) => { f64 }
		Err (_) => { 		
			println!("UNKNOWN: message_level has an incorrect type (0.0 - 1.0)."); 	
			process::exit(3);
		}
	};

	let (response, data, messages) = get_authsmtp_data (&opts.username, &opts.password, messages_level, data_level);		

	if response.contains("AUTHSMTP ERROR") {
		println!("{} | data_quota%={};;;; messages_quota%={};;;;", response, data, messages);
		process::exit(3);
	}
	else if response.contains("OK") {
		println!("{} | data_quota%={};;;; messages_quota%={};;;;", response, data, messages);
		process::exit(0);
	}
	else if response.contains("CRITICAL") {
		println!("{} | data_quota%={};;;; messages_quota%={};;;;", response, data, messages);
		process::exit(2);
	}
	else {
		println!("UNKNOWN: Check_authsmtp failed. | data_quota%={};;;; messages_quota%={};;;;", data, messages);
		process::exit(3);
	}

}



