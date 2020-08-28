use std::str::FromStr;

use serde::{de::Error, Deserialize, Deserializer};
use serde_json::Value;


pub fn i64_from_str<'de, D>(deserializer: D) -> Result<i64, D::Error>
    where D: Deserializer<'de>
{
    let value = Value::deserialize(deserializer)?;
    return match value {
        Value::String(str_value) => {
            i64::from_str(str_value.as_str()).map_err(|e| Error::custom(e))
        },
        Value::Number(number_value) => {
            let i64_value = number_value.as_i64();
            match i64_value {
                Some(i64_value) => Ok(i64_value),
                None => Err(Error::custom("value is not a valid i64"))
            }
        }
        _ => {
            Err(Error::custom("value is not a valid i64"))
        }
    };
}
