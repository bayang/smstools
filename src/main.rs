#![feature(
    iterator_find_map, // indespensible
    str_escape,
    range_contains // Why isn't this already stable?
)]
#[macro_use]
extern crate clap;
extern crate chrono;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;

extern crate xml5ever;

use std::{fs};
use std::path::PathBuf;

mod sanitize;
mod xml;
mod log;
mod formatter;

fn main() {
    let matches = clap_app!(sms2markdown => 
    	(version: crate_version!())
    	(author: "Techcable <Techcable@techcable.net>")
    	// SMS Backup & Restore v10.05.210 
    	(about: "Converts SMS Backups from XML to JSON")
    	//(@arg contact: +required "Sets the contact whose texts we need to log")
    	(@arg file: +required "Sets the file to read data from")
    	(@arg output: +required "Output JSON file")
    	(@arg verbose: -v --verbose "Gives verbose error and status information")
    ).get_matches();
    let file: PathBuf = matches.value_of("file").unwrap().into();
    let output: PathBuf = matches.value_of("output").unwrap().into();
    let verbose = matches.is_present("verbose");
    let raw_text = String::from_utf8(fs::read(&file).unwrap()).unwrap();
    let sanitized = ::sanitize::cleanup_html_escapes(&raw_text);
    fs::write("sanitized.xml", &sanitized).unwrap();
    let log = ::xml::parse_log(verbose, sanitized);
    fs::write(&output, ::formatter::to_string_escaped(&log)).unwrap();
}
