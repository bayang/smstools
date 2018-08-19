#![feature(
    iterator_find_map, // indespensible
    str_escape
)]
#[macro_use]
extern crate clap;
extern crate chrono;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate html5ever;

use std::{fs};
use std::path::PathBuf;

use chrono::DateTime;

mod xml;
mod log;

fn main() {
    let matches = clap_app!(sms2markdown => 
    	(version: crate_version!())
    	(author: "Techcable <Techcable@techcable.net>")
    	// SMS Backup & Restore v10.05.210 
    	(about: "Converts SMS Backups from XML to JSON")
    	//(@arg contact: +required "Sets the contact whose texts we need to log")
    	(@arg file: +required "Sets the file to read data from")
    	(@arg output: +required "Output JSON file")
    ).get_matches();
    let file: PathBuf = matches.value_of("file").unwrap().into();
    let output: PathBuf = matches.value_of("output").unwrap().into();
    let raw_text = String::from_utf8(fs::read(&file).unwrap()).unwrap();
    let log = ::xml::parse_log(raw_text.into());
    fs::write(&output, ::serde_json::to_string_pretty(&log).unwrap()).unwrap();
}
