pub mod base64_opt {
    extern crate base64;
    use serde::{Serializer, de, Deserialize, Deserializer};

    pub fn serialize<S>(bytes: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        match bytes {
            Some(bytes) => serializer.serialize_some(&base64::encode(bytes)),
            None => serializer.serialize_none()
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
        where D: Deserializer<'de>
    {
        let s = Option::<&str>::deserialize(deserializer)?;
        Ok(match s {
            Some(s) => Some(base64::decode(s).map_err(de::Error::custom)?),
            None => None
        })
    }
}