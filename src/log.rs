use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhoneNumber(pub String);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextLog {
    pub sms_messages: Vec<SmsMessage>
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
    pub kind: SmsMessageKind,
    /// The body of this SMS message
    pub body: String
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename = "lower")]
pub enum SmsMessageKind {
    Sent,
    Received {
        /// Date they claimed to send the text (date is when we actually received it)
        date_sent: DateTime<Utc>
    }
}