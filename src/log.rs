use std::fmt::{self, Display};
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct PhoneNumber(pub String);
impl Display for PhoneNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

pub trait TextMessage {
    fn address(&self) -> &PhoneNumber;
    fn contact_name(&self) -> &str;
    fn date(&self) -> DateTime<Utc>;
    fn readable_date(&self) -> &str;
    fn kind(&self) -> MessageKind;
    fn body(&self) -> BodyKind;
}
pub enum BodyKind<'a> {
    Sms(&'a str),
    Mms {
        parts: &'a [MmsMessagePart]
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextLog {
    pub sms_messages: Vec<SmsMessage>,
    pub mms_messages: Vec<MmsMessage>
}
impl TextLog {
    //noinspection RsNeedlessLifetimes
    pub fn iter<'a>(&'a self) -> impl Iterator<Item=&'a dyn TextMessage> + 'a {
        self.sms_messages.iter()
            .map(|message| message as &dyn TextMessage)
            .chain(self.mms_messages.iter().map(|message| message as &dyn TextMessage))
    }
    pub fn list_contacts(&self) -> HashMap<PhoneNumber, HashSet<String>> {
        let mut result = HashMap::with_capacity(
            self.sms_messages.len() + self.mms_messages.len());
        for sms in &self.sms_messages {
            result.entry(sms.address.clone())
                .or_insert_with(HashSet::new)
                .insert(sms.contact_name.clone());
        }
        for mms in &self.mms_messages {
            result.entry(mms.address.clone())
                .or_insert_with(HashSet::new)
                .insert(mms.contact_name.clone());
        }
        result
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MmsMessage {
    /// The phone number we're texting
    pub address: PhoneNumber,
    /// The name of the contact
    pub contact_name: String,
    /// The date we received/sent the text
    pub date: DateTime<Utc>,
    /// The human-readable version of `date`
    ///
    /// This is _included_ in the text dump,
    /// so I can't really just ignore it in case
    /// there is something with time zones.
    pub readable_date: String,
    /// Whether this message was sent or received
    pub kind: MessageKind,
    /// The parts of this MMS message
    pub parts: Vec<MmsMessagePart>
}
impl TextMessage for MmsMessage {
    #[inline]
    fn address(&self) -> &PhoneNumber {
        &self.address
    }

    #[inline]
    fn contact_name(&self) -> &str {
        &self.contact_name
    }

    #[inline]
    fn date(&self) -> DateTime<Utc> {
        self.date
    }

    #[inline]
    fn readable_date(&self) -> &str {
        &self.readable_date
    }

    #[inline]
    fn kind(&self) -> MessageKind {
        self.kind
    }

    #[inline]
    fn body(&self) -> BodyKind {
        BodyKind::Mms { parts: &self.parts }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MmsMessagePart {
    /// The content type of this message part
    pub content_type: String,
    /// The name of where this content is located
    pub content_location: String,
    /// The text of this message part
    pub text: Option<String>,
    pub seq: i32,
    /// The binary data of this message part
    #[serde(with = "::utils::base64_opt")]
    pub data: Option<Vec<u8>>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SmsMessage {
    /// The phone number we're texting
    pub address: PhoneNumber,
    /// The name of the contact
    pub contact_name: String,
    /// The date we received/sent the text
    pub date: DateTime<Utc>,
    /// The human-readable version of `date`
    ///
    /// This is _included_ in the text dump,
    /// so I can't really just ignore it in case
    /// there is something with time zones.
    pub readable_date: String,
    /// Whether this message was sent or received
    pub kind: MessageKind,
    /// The body of this SMS message
    pub body: String
}
impl TextMessage for SmsMessage {
    #[inline]
    fn address(&self) -> &PhoneNumber {
        &self.address
    }

    #[inline]
    fn contact_name(&self) -> &str {
        &self.contact_name
    }

    #[inline]
    fn date(&self) -> DateTime<Utc> {
        self.date
    }

    #[inline]
    fn readable_date(&self) -> &str {
        &self.readable_date
    }

    #[inline]
    fn kind(&self) -> MessageKind {
        self.kind
    }

    #[inline]
    fn body(&self) -> BodyKind {
        BodyKind::Sms(&self.body)
    }
}
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename = "lower")]
pub enum MessageKind {
    Sent,
    Received {
        /// Date they claimed to send the text (date is when we actually received it)
        date_sent: DateTime<Utc>
    }
}