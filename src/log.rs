use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhoneNumber(pub String);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextLog {
    pub sms_messages: Vec<SmsMessage>,
    pub mms_messages: Vec<MmsMessage>
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename = "lower")]
pub enum MessageKind {
    Sent,
    Received {
        /// Date they claimed to send the text (date is when we actually received it)
        date_sent: DateTime<Utc>
    }
}