use std::collections::HashSet;
use std::fmt::{self, Debug};
use std::str::FromStr;

use chrono::{DateTime, Utc, TimeZone};

use xml5ever::tendril::{TendrilSink};
use xml5ever::rcdom::{NodeData, Handle, Node, RcDom};
use xml5ever::interface::TreeSink;
use xml5ever::QualName;
use xml5ever::Attribute as XmlAttribute;
use xml5ever::driver::XmlParseOpts;

use log::{MessageKind, PhoneNumber, MmsMessagePart, MmsMessage, SmsMessage, TextLog};

pub fn parse_log(verbose: bool, text: String) -> TextLog {
    let mut opts = XmlParseOpts::default();
    opts.tokenizer.exact_errors = verbose;
    let parser = ::xml5ever::driver::parse_document(
        RcDom::default(), opts);
    let mut dom = parser.one(text);
    if verbose {
        println!("Errors {:#?}", dom.errors);
    }
    let document = dom.get_document();
    let element = document.children.borrow().iter()
        .find_map(|node| element_contents(&**node))
        .unwrap();
    let mut sms_messages = Vec::new();
    let mut mms_messages = Vec::new();

    for sms in element.filter_elements("sms") {
        sms_messages.push(parse_sms(&sms));
    }
    for mms in element.filter_elements("mms") {
        mms_messages.push(parse_mms(&mms));
    }
    eprintln!("TODO: SUPPORT MMS");
    TextLog { sms_messages, mms_messages }
}

fn parse_mms(element: &ElementData) -> MmsMessage {
    let address = PhoneNumber(element.attr("address").into());
    let date = parse_unix_epoch(element.attr("date"));
    let readable_date = element.attr("readable_date").to_owned();
    let contact_name = element.attr("contact_name").to_owned();
    let msg_type = element.attr("read");
    let kind = match msg_type {
        "1" => {
            // sent
            MessageKind::Sent
        },
        "2" => {
            // received
            MessageKind::Received {
                date_sent: parse_unix_epoch(element.attr("date_sent"))
            }
        },
        _ => panic!("Unknown read type {:?}", msg_type)
    };
    let parts = element.find_child("parts");
    let parts = parts.child_elements()
        .map(|part| parse_mms_part(&part))
        .collect::<Vec<MmsMessagePart>>();
    MmsMessage { kind, date, readable_date, contact_name, address, parts }
}
fn parse_mms_part(element: &ElementData) -> MmsMessagePart {
    assert_eq!(element.name, "part");
    let content_type = element.attr("ct").to_owned();
    let content_location = element.attr("cl").to_owned();
    let text = parse_opt_text(element.attr("text"));
    let seq = i32::from_str(element.attr("seq")).unwrap();
    let data = element.get_attr("data")
        .map(|data| ::base64::decode(data).expect("Unable to base64 decode"));
    MmsMessagePart { content_type, content_location, text, seq, data, }
}
fn parse_opt_text(text: &str) -> Option<String> {
    if text == "null" {
        None
    } else {
        Some(text.into())
    }
}
fn parse_sms(element: &ElementData) -> SmsMessage {
    let address = PhoneNumber(element.attr("address").into());
    let date = parse_unix_epoch(element.attr("date"));
    let body = element.attr("body").to_owned();
    let readable_date = element.attr("readable_date").to_owned();
    let contact_name = element.attr("contact_name").to_owned();
    let msg_type = element.attr("type");
    let kind = match msg_type {
        "2" => {
            // sent
            MessageKind::Sent
        },
        "1" => {
            // received
            MessageKind::Received {
                date_sent: parse_unix_epoch(element.attr("date_sent"))
            }
        },
        _ => panic!("Unknown msg type {:?}", msg_type)
    };
    SmsMessage { kind, date, body, readable_date, contact_name, address }
}
fn parse_unix_epoch(date: &str) -> DateTime<Utc> {
    Utc.timestamp_millis(i64::from_str(date)
        .unwrap_or_else(|e| panic!("Invalid digits in {:?}: {}", date, e)))
}

struct ElementData {
    name: String,
    attrs: Vec<Attribute>,
    children: Vec<Handle>
}
impl ElementData {
    fn child_elements<'a>(&'a self) -> impl Iterator<Item=ElementData> + 'a {
        self.children.iter().filter_map(|node| element_contents(&*node))
    }
    #[inline]
    fn find_child(&self, name: &str) -> ElementData {
        self.filter_elements(name).next().unwrap()
    }
    fn filter_elements<'a>(&'a self, name: &'a str) -> impl Iterator<Item=ElementData> + 'a {
        self.child_elements().filter(move |element| element.name == name)
    }
    fn child_name_set(&self) -> HashSet<String> {
        self.child_elements()
            .map(|element| element.name)
            .collect::<HashSet<_>>()
    }
    #[inline]
    fn attr(&self, name: &str) -> &str {
        self.get_attr(name).unwrap()
    }
    fn get_attr(&self, name: &str) -> Option<&str> {
        self.attrs.iter()
            .find(|attr| attr.name == name)
            .map(|attr| &*attr.value)
    }
}
impl Debug for ElementData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ElementData")
            .field("name", &self.name)
            .field("attrs", &self.attrs)
            .field("child_names", &self.child_name_set())
            .finish()
    }
}
#[derive(Debug)]
struct Attribute {
    name: String,
    value: String
}
impl<'a> From<&'a XmlAttribute> for Attribute {
    fn from(xml_attr: &'a XmlAttribute) -> Self {
        Attribute {
            name: local_name(&xml_attr.name),
            value: String::from(&xml_attr.value)
        }
    }
}
fn local_name(name: &QualName) -> String {
    String::from(&*name.local)
}
fn element_contents(node: &Node) -> Option<ElementData> {
    if let NodeData::Element { ref name, ref attrs, .. } = node.data {
        Some(ElementData {
            name: local_name(&*name),
            attrs: attrs.borrow().iter().map(Attribute::from).collect(),
            children: node.children.borrow().clone(),
        })
    } else {
        None
    }
}