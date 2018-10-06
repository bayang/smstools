use super::log::{TextMessage, TextLog, MessageKind, BodyKind, MmsMessagePart};

pub fn render_log(log: &TextLog, sender: &str, contact: &str, con) -> Markup {
    let mut messages = log.iter()
        .filter(|message| message.contact_name() == contact)
        .collect_vec();
}