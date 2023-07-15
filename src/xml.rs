use std::fmt::{self, Display};
use std::str::FromStr;

use anyhow::{anyhow, Context};
use base64::{engine::general_purpose::STANDARD as BASE64_ENGINE, Engine};
use chrono::{DateTime, TimeZone, Utc};
use itertools::Itertools;

use roxmltree::Node;

use crate::model::{MessageKind, MmsMessage, MmsMessagePart, PhoneNumber, SmsMessage, TextLog};

pub fn parse_log(text: String) -> anyhow::Result<TextLog> {
    let document = ::roxmltree::Document::parse(&text).context("Failed to parse XML")?;
    let element = document.root_element();
    let mut sms_messages = Vec::new();
    let mut mms_messages = Vec::new();

    for child in element.children().filter(Node::is_element) {
        match child.tag_name().name() {
            "sms" => {
                sms_messages.push(parse_sms(child).context(child.parse_fail_ctx("sms message"))?)
            }
            "mms" => mms_messages.push(parse_mms(child).context(child.parse_fail_ctx("mms part"))?),
            name => anyhow::bail!("Bad tag name {name:?} at {}", child.start_pos()),
        }
    }
    Ok(TextLog {
        sms_messages,
        mms_messages,
    })
}

fn parse_mms(element: Node<'_, '_>) -> anyhow::Result<MmsMessage> {
    let address = PhoneNumber(element.expect_attr("address")?.into());
    let date = parse_unix_epoch(element.expect_attr("date")?)?;
    let readable_date = element.expect_attr("readable_date")?.to_owned();
    let contact_name = element.expect_attr("contact_name")?.to_owned();
    let msg_type = element.expect_attr("m_type")?;
    let kind = match msg_type {
        "128" => {
            // sent
            MessageKind::Sent
        }
        "132" => {
            // received
            MessageKind::Received {
                date_sent: parse_unix_epoch(element.expect_attr("date_sent")?)?,
            }
        }
        _ => anyhow::bail!("Unknown message type {msg_type:?}"),
    };
    let parts = element.expect_child("parts")?;
    let parts = parts
        .child_elements()
        .map(|part| parse_mms_part(&part).context(part.parse_fail_ctx("mms message part")))
        .collect::<Result<Vec<MmsMessagePart>, _>>()?;
    Ok(MmsMessage {
        kind,
        date,
        readable_date,
        contact_name,
        address,
        parts,
    })
}
fn parse_mms_part(element: &Node<'_, '_>) -> anyhow::Result<MmsMessagePart> {
    assert!(element.has_tag_name("part"));
    let content_type = element.expect_attr("ct")?.into();
    let content_location = element.expect_attr("cl")?.into();
    let text = element
        .attribute("text")
        .filter(|&s| s != "null")
        .map(String::from);
    let seq = i32::from_str(element.expect_attr("seq")?).context("Invalid string for `seq`")?;
    let data = element
        .attribute("data")
        .map(|data| {
            BASE64_ENGINE
                .decode(data)
                .context("Unable to base64 decode `data`")
        })
        .transpose()?;
    Ok(MmsMessagePart {
        content_type,
        content_location,
        text,
        seq,
        data,
    })
}
fn parse_sms(element: Node<'_, '_>) -> anyhow::Result<SmsMessage> {
    let address = PhoneNumber(element.expect_attr("address")?.into());
    let date = parse_unix_epoch(element.expect_attr("date")?)?;
    let body = element.expect_attr("body")?.to_owned();
    let readable_date = element.expect_attr("readable_date")?.to_owned();
    let contact_name = element.expect_attr("contact_name")?.to_owned();
    let msg_type = element.expect_attr("type")?;
    let kind = match msg_type {
        "2" => {
            // sent
            MessageKind::Sent
        }
        "1" => {
            // received
            MessageKind::Received {
                date_sent: parse_unix_epoch(element.expect_attr("date_sent")?)?,
            }
        }
        _ => anyhow::bail!("Unknown msg type {msg_type:?}"),
    };
    Ok(SmsMessage {
        kind,
        date,
        body,
        readable_date,
        contact_name,
        address,
    })
}
struct NodeParseFailContext {
    role: &'static str,
    start_pos: roxmltree::TextPos,
    tag_name: String,
    tag_namespace: Option<String>,
}
impl Display for NodeParseFailContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Failed to parse XML node for {} (start: {}, tag: {}",
            self.role, self.start_pos, self.tag_name
        )?;
        if let Some(ref ns) = self.tag_namespace {
            write!(f, ", tag.namespace = {ns}")?;
        }
        f.write_str(")")?;
        Ok(())
    }
}
// Extension methods for roxmltree::Node
trait NodeParseUtils<'a, 'input: 'a> {
    fn as_node(&self) -> &roxmltree::Node<'a, 'input>;
    fn start_pos(&self) -> roxmltree::TextPos {
        let node = self.as_node();
        node.document().text_pos_at(node.range().start)
    }
    #[cold]
    fn parse_fail_ctx(&self, role: &'static str) -> NodeParseFailContext {
        let node = self.as_node();
        NodeParseFailContext {
            role,
            start_pos: self.start_pos(),
            tag_name: node.tag_name().name().into(),
            tag_namespace: node.tag_name().namespace().map(String::from),
        }
    }
    fn expect_attr(&self, name: &str) -> anyhow::Result<&'a str> {
        self.as_node()
            .attribute(name)
            .ok_or_else(|| anyhow!("Expected attribute {name}"))
    }
    fn expect_child(&self, name: &str) -> anyhow::Result<Node<'a, 'input>> {
        self.find_child(name)?
            .ok_or_else(|| anyhow!("Expected a child named {name:?}"))
    }
    fn find_child(&self, name: &str) -> anyhow::Result<Option<Node<'a, 'input>>> {
        self.child_elements()
            .filter(|child| child.has_tag_name(name))
            .at_most_one()
            .map_err(|_| anyhow!("Expected at most one child named {name:?}"))
    }
    fn child_elements(&self) -> ChildElementsIter<'a, 'input> {
        self.as_node().children().filter(Node::is_element)
    }
}
type ChildElementsIter<'a, 'input> =
    std::iter::Filter<roxmltree::Children<'a, 'input>, fn(&roxmltree::Node<'a, 'input>) -> bool>;
impl<'a, 'input: 'a> NodeParseUtils<'a, 'input> for roxmltree::Node<'a, 'input> {
    #[inline(always)]
    fn as_node(&self) -> &roxmltree::Node<'a, 'input> {
        self
    }
}
fn parse_unix_epoch(date: &str) -> anyhow::Result<DateTime<Utc>> {
    i64::from_str(date)
        .ok()
        .and_then(|val| Utc.timestamp_millis_opt(val).single())
        .ok_or_else(|| anyhow!("Invalid digits in {date:?}"))
}
