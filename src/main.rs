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
extern crate base64;

extern crate xml5ever;

use std::time::{Instant};
use std::{fs};
use std::io::{self, BufReader, Read};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

mod sanitize;
mod xml;
mod log;
mod formatter;
mod utils;

fn app() -> ::clap::App<'static, 'static> {
    clap_app!(sms2markdown =>
    	(version: crate_version!())
    	(author: "Techcable <Techcable@techcable.net>")
    	// SMS Backup & Restore v10.05.210
    	(about: "Converts SMS Backups from XML to JSON")
    	//(@arg contact: +required "Sets the contact whose texts we need to log")
    	(@arg file: +required "Sets the file to read data from")
    	(@arg verbose: -v --verbose "Gives verbose error and status information")
    	(@subcommand dump_json =>
            (about: "Dumps a json formatted version of these logs")
        	(@arg output: +required "Output JSON file")
    	)
    )
}

fn main() {
    let matches = app().get_matches();
    let file: PathBuf = matches.value_of("file").unwrap().into();
    let verbose = matches.is_present("verbose");
    let options = CommonOptions { file, verbose };
    match matches.subcommand() {
        ("dump_json", Some(matches)) => {
            let output: PathBuf = matches.value_of("output").unwrap().into();
            dump_json(&options, &output);
        },
        _ => {
            if let Some(name) = matches.subcommand_name() {
                eprintln!("Invalid subcommand: {:?}", name);
            }
            app().print_help().unwrap();
        }
    }

}
fn dump_json(options: &CommonOptions, output: &Path) {
    let log = options.parse_log();
    fs::write(&output, ::formatter::to_string_escaped(&log)).unwrap();
}
struct CommonOptions {
    verbose: bool,
    file: PathBuf
}
impl CommonOptions {
    fn parse_log(&self) -> ::log::TextLog {
        let start = Instant::now();
        let log = self.try_parse_log()
            .unwrap_or_else(|e| panic!("Unable to parse {}: {:?}", self.file.display(), e));
        let duration = start.elapsed();
        eprintln!("Parsed {} in {}s", self.file.display(), (duration.as_secs() as f64) + ((duration.subsec_millis() as f64) / 1000.0));
        log
    }
    fn try_parse_log(&self) -> Result<::log::TextLog, FileParseError> {
        let mut file = BufReader::new(::fs::File::open(&self.file)?);
        Ok(match self.file.extension().and_then(OsStr::to_str) {
            Some("xml") => {
                let mut raw_text = String::new();
                file.read_to_string(&mut raw_text)?;
                let sanitized = ::sanitize::cleanup_html_escapes(&raw_text);
                ::xml::parse_log(self.verbose, sanitized)
            },
            Some("json") => ::serde_json::from_reader(file)?,
            _ => panic!("Unable to determine extension of {}", self.file.display())
        })

    }
}
#[derive(Debug)]
enum FileParseError {
    Io(io::Error),
    Json(::serde_json::Error),
}
macro_rules! from_errors {
    ($target:ident, {$($cause:ty => $variant:ident),*}) => {
        $(impl From<$cause> for $target {
            fn from(cause: $cause) -> $target {
                $target::$variant(cause)
            }
        })*
    };
}
from_errors!(FileParseError, {io::Error => Io, ::serde_json::Error => Json});
