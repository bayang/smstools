pub mod base64_opt {
    use base64::{engine::general_purpose::STANDARD as ENGINE, Engine};
    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match bytes {
            Some(bytes) => serializer.serialize_some(&ENGINE.encode(bytes)),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = Option::<String>::deserialize(deserializer)?;
        Ok(match s {
            Some(s) => Some(ENGINE.decode(s).map_err(de::Error::custom)?),
            None => None,
        })
    }
}
