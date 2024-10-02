use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub user_id: usize,
    #[serde(flatten)]
    pub date: EventDate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventDate {
    #[serde(with = "event_date_format")]
    #[serde(rename(serialize = "date", deserialize = "date"))]
    pub inner: NaiveDate,
}

mod event_date_format {
    use chrono::NaiveDate;
    use serde::{Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%d";

    pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let string = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&string)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let string = String::deserialize(deserializer)?;
        let date = NaiveDate::parse_from_str(&string, FORMAT).map_err(Error::custom)?;

        Ok(date)
    }
}
