//! Conversions from `Value` to standard types.

use crate::prelude::*;
use bytes::Bytes;

impl TryFrom<&Value> for f64 {
    type Error = ();

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Temperature(value)
            | Value::Cloudiness(value)
            | Value::Duration(value)
            | Value::Energy(value)
            | Value::Length(value)
            | Value::Power(value)
            | Value::Rh(value)
            | Value::Speed(value)
            | Value::Volume(value)
            | Value::RelativeIntensity(value)
            | Value::BatteryLife(value) => Ok(*value),
            _ => Err(()),
        }
    }
}

impl TryFrom<&Value> for i64 {
    type Error = ();

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Bft(value) => (*value).try_into().map_err(|_| ()),
            Value::Counter(value) | Value::DataSize(value) => (*value).try_into().map_err(|_| ()),
            _ => Err(()),
        }
    }
}

impl TryFrom<&Value> for bool {
    type Error = ();

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Boolean(value) => Ok(*value),
            _ => Err(()),
        }
    }
}

impl TryFrom<&Value> for String {
    type Error = ();

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::ImageUrl(value) | Value::Text(value) | Value::StringEnum(value) => Ok(value.clone()),
            _ => Err(()),
        }
    }
}

impl TryFrom<&Value> for Arc<Bytes> {
    type Error = ();

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Blob(bytes) => Ok(bytes.clone()),
            _ => Err(()),
        }
    }
}
