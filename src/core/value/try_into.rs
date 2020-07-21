use crate::prelude::*;
use bytes::Bytes;

impl TryInto<f64> for &Value {
    type Error = ();

    fn try_into(self) -> Result<f64, Self::Error> {
        match self {
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

impl TryInto<i64> for &Value {
    type Error = ();

    fn try_into(self) -> Result<i64, Self::Error> {
        match self {
            Value::Bft(value) => (*value).try_into().map_err(|_| ()),
            Value::Counter(value) | Value::DataSize(value) => (*value).try_into().map_err(|_| ()),
            _ => Err(()),
        }
    }
}

impl TryInto<bool> for &Value {
    type Error = ();

    fn try_into(self) -> Result<bool, Self::Error> {
        match self {
            Value::Boolean(value) => Ok(*value),
            _ => Err(()),
        }
    }
}

impl TryInto<String> for &Value {
    type Error = ();

    fn try_into(self) -> Result<String, Self::Error> {
        match self {
            Value::ImageUrl(value) | Value::Text(value) => Ok(value.clone()),
            _ => Err(()),
        }
    }
}

impl TryInto<Arc<Bytes>> for &Value {
    type Error = ();

    fn try_into(self) -> Result<Arc<Bytes>, Self::Error> {
        match self {
            Value::Blob(bytes) => Ok(bytes.clone()),
            _ => Err(()),
        }
    }
}
