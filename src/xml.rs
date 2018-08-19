use std::collections::HashSet;
use std::fmt::{self, Debug};
use std::str::FromStr;

use chrono::{DateTime, Utc, TimeZone};

use sxd_document::Package;
use sxd_document::dom::{ChildOfRoot, ChildOfElement, Element};
use sxd_document::parser::parse as parse_xml;

use log::{SmsMessageKind, PhoneNumber, SmsMessage, TextLog};

pub fn parse_log(text: String) -> TextLog {
    let dom = parse_xml(&text).unwrap();
    let root = dom.as_document().root();
    let element = root.children().into_iter()
        .find_map(ChildOfRoot::element)
        .unwrap();
    let mut sms_messages = Vec::new();
    for sms in find_children(element, "sms") {
        sms_messages.push(parse_sms(&sms));
    }
    eprintln!("TODO: SUPPORT MMS");
    TextLog { sms_messages }
}
fn parse_sms(element: &Element) -> SmsMessage {
    let address = PhoneNumber(element.attribute_value("address").unwrap().into());
    let date = parse_unix_epoch(element.attribute_value("date").unwrap());
    let body = element.attribute_value("body").unwrap().to_owned();
    if !body.is_ascii() {
        let mut escaped = String::new();
        for c in body.chars() {
            if c.is_ascii() {
                escaped.push(c);
            } else {
                escaped.extend(c.escape_unicode());
            }
        }
        panic!("Unexpected body {}", escaped);
    }
    let readable_date = element.attribute_value("readable_date").unwrap().to_owned();
    let contact_name = element.attribute_value("contact_name").unwrap().to_owned();
    let msg_type = element.attribute_value("type").unwrap();
    let kind = match msg_type {
        "2" => {
            // sent
            SmsMessageKind::Sent
        },
        "1" => {
            // received
            SmsMessageKind::Received {
                date_sent: parse_unix_epoch(element.attribute_value("date_sent").unwrap())
            }
        },
        _ => panic!("Unknown msg type {:?}", msg_type)
    };
    SmsMessage { kind, date, body, readable_date, contact_name, address }
}
fn parse_unix_epoch(date: &str) -> DateTime<Utc> {
    Utc.timestamp_millis(i64::from_str(date).unwrap())
}
fn find_children<'a, 'p: 'a>(element: Element<'p>, name: &'a str) -> impl Iterator<Item=Element<'p>> + 'a {
    element.children().into_iter()
        .filter_map(ChildOfElement::element)
        .filter(move |element| element.name().local_part() == name)
}
