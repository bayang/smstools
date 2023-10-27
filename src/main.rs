#![warn(rust_2021_compatibility, rust_2018_compatibility, rust_2018_idioms)]
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fs;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::time::Instant;

use itertools::Itertools;

mod formatter;
mod html;
mod model;
mod sanitize;
mod utils;
mod xml;

use self::model::PhoneNumber;

/// A set of utilities for processing SMS backups
///
/// Originially used with SMS Backup & Restore v10.05.210
#[derive(clap::Parser)]
#[command(author, version)]
#[command(propagate_version = true)]
struct App {
    /// Gives verbose error and status information
    #[arg(short, long)]
    verbose: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Renders a HTML file of all texts with a particular contact
    RenderHtml {
        /// The input XML file
        input_file: PathBuf,
        /// The contact whose texts to print
        #[arg(long, required = true)]
        contact: String,
    },
    /// Lists the names of all contexts ever texted
    ListContacts(ListContacts),
    /// Dumps a json formatted version of the input file
    DumpJson {
        /// The input XML file to read from
        input_file: PathBuf,
        /// Output JSON file
        #[arg(long, required = true)]
        output: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .init();
    let app = <App as clap::Parser>::parse();
    let options = CommonOptions {
        verbose: app.verbose,
    };
    match app.command {
        Command::RenderHtml {
            input_file,
            contact,
        } => {
            let log = options.parse_log(&input_file)?;
            println!("{}", crate::html::render_log(&log, &contact).0);
        }
        Command::ListContacts(args) => list_contacts(&options, &args)?,
        Command::DumpJson { input_file, output } => {
            let log = options.parse_log(&input_file)?;
            fs::write(output, crate::formatter::to_string_escaped(&log))?;
        }
    }
    Ok(())
}
#[derive(clap::Args)]
struct ListContacts {
    /// The input XML file to read from
    input_file: PathBuf,
}
fn list_contacts(options: &CommonOptions, contacts: &ListContacts) -> anyhow::Result<()> {
    const UNKNOWN_CONTACT_NAME: &str = "(Unknown)";
    let log = options.parse_log(&contacts.input_file)?;
    let contacts = log.list_contacts();
    let mut by_name = HashMap::with_capacity(contacts.len());
    let mut unnamed_contacts = Vec::new();
    for (number, names) in contacts.iter() {
        let mut found_name = false;
        for name in names {
            if name != UNKNOWN_CONTACT_NAME {
                by_name
                    .entry(name.clone())
                    .or_insert_with(HashSet::new)
                    .insert(number.clone());
                found_name = true;
            }
        }
        if !found_name {
            unnamed_contacts.push(number.clone());
        }
    }
    let mut named_contacts = by_name
        .iter()
        .map(|(name, phones)| {
            let mut phones = phones.iter().cloned().collect::<Vec<PhoneNumber>>();
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
    Ok(())
}
fn bold_underline<T: AsRef<str>>(text: T) -> String {
    format!("\u{1B}[1;4m{}\u{1B}[0m", text.as_ref())
}
struct CommonOptions {
    verbose: bool,
}
impl CommonOptions {
    fn parse_log(&self, path: &Path) -> Result<crate::model::TextLog, anyhow::Error> {
        let start = Instant::now();
        let mut file = BufReader::new(std::fs::File::open(path)?);
        let success = match path.extension().and_then(OsStr::to_str) {
            Some("xml") => {
                let mut raw_text = String::new();
                file.read_to_string(&mut raw_text)?;
                let sanitized = crate::sanitize::cleanup_html_escapes(&raw_text);
                crate::xml::parse_log(self.verbose, sanitized)
            }
            Some("json") => ::serde_json::from_reader(file)?,
            _ => anyhow::bail!("Unable to determine extension of {}", path.display()),
        };
        let duration = start.elapsed();
        log::info!(
            "Parsed {} in {}s",
            path.display(),
            (duration.as_secs() as f64) + ((duration.subsec_millis() as f64) / 1000.0)
        );
        Ok(success)
    }
}
