#![feature(
    iterator_find_map, // indespensible
    str_escape,
    range_contains, // Why isn't this already stable?
    proc_macro_non_items, // Needed for maud
)]
#[macro_use]
extern crate clap;
extern crate chrono;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
extern crate base64;
extern crate itertools;
extern crate maud;

extern crate xml5ever;

use std::time::{Instant};
use std::{fs};
use std::io::{self, BufReader, Read};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use chrono::{Date, Local};

mod sanitize;
mod xml;
mod log;
mod formatter;
mod utils;
mod html;
mod markdown;

use self::log::PhoneNumber;

fn app() -> ::clap::App<'static, 'static> {
    clap_app!(smstools =>
    	(version: crate_version!())
    	(author: "Techcable <Techcable@techcable.net>")
    	// SMS Backup & Restore v10.05.210
    	(about: "A set of utilities for processing SMS backups")
    	(@arg file: +required "Sets the file to read data from")
    	(@arg verbose: -v --verbose "Gives verbose error and status information")
    	(@subcommand html_log =>
    	    (about: "Creates a HTML log of texts with the specified person")
    	    (@arg contact: +required "The contact whose texts we're printing")
    	)
    	(@subcommand markdown_log =>
    	    (about: "Creates a markdown formatted log of a day's texts")
    	    (@arg contact: +required "The contact whose texts we're printing")
    	    (@arg date: +required "The date of the texts we're printed")
    	)
    	(@subcommand list_contacts =>
    	    (about: "Lists the names of everyone you've ever texted")
    	)
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
        ("html_log", Some(matches)) => {
            let contact = matches.value_of("contact").unwrap();
            html_log(&options, contact)
        },
        ("markdown_log", Some(matches)) => {
            let contact = matches.value_of("contact").unwrap();
            let date: Date<Local> = value_t!(matches, "date", Date<Local>)
                .unwrap_or_else(|e| e.exit());

        }
        ("list_contacts", Some(_)) => list_contacts(&options),
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
fn list_contacts(options: &CommonOptions) {
    const UNKNOWN_CONTACT_NAME: &str = "(Unknown)";
    let log = options.parse_log();
    let contacts = log.list_contacts();
    let mut by_name = HashMap::with_capacity(contacts.len());
    let mut unnamed_contacts = Vec::new();
    for (number, names) in contacts.iter() {
        let mut found_name = false;
        for name in names {
            if name != UNKNOWN_CONTACT_NAME {
                by_name.entry(name.clone())
                    .or_insert_with(HashSet::new)
                    .insert(number.clone());
                found_name = true;
            }
        }
        if !found_name {
            unnamed_contacts.push(number.clone());
        }
    }
    let mut named_contacts = by_name.iter()
        .map(|(name, phones)| {
            let mut phones = phones.iter().cloned()
                .collect::<Vec<PhoneNumber>>();
            phones.sort();
            (name.clone(), phones)
        })
        .collect::<Vec<(String, Vec<PhoneNumber>)>>();
    named_contacts.sort();
    unnamed_contacts.sort();
    println!("{}", bold_underline("Named contacts"));
    for (name, phones) in named_contacts {
        println!("  {} - {}", name, phones.iter().join(", "))
    }
    println!("{}", bold_underline("Unnamed contacts"));
    for phone in unnamed_contacts {
        println!("  {}", phone);
    }
}
fn bold_underline<T: AsRef<str>>(text: T) -> String {
    format!("\u{1B}[1;4m{}\u{1B}[0m", text.as_ref())
}
fn dump_json(options: &CommonOptions, output: &Path) {
    let log = options.parse_log();
    fs::write(&output, ::formatter::to_string_escaped(&log)).unwrap();
}
fn html_log(options: &CommonOptions, contact: &str) {
    let log = options.parse_log();
    println!("{}", ::html::render_log(&log, contact).0);
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
