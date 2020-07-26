use crate::prelude::*;
use std::any::type_name;

pub fn expect<'a, T: TryFrom<&'a Value>>(
    service_id: &str,
    message: &'a Message,
    expected_sensor_id: &str,
) -> Option<T> {
    if message.sensor.id != expected_sensor_id {
        debug!(
            "[{}] `{}` does not match `{}`.",
            service_id, message.sensor.id, expected_sensor_id
        );
        return None;
    }
    match TryInto::<T>::try_into(&message.reading.value) {
        Ok(value) => Some(value),
        Err(..) => {
            error!("[{}] Value is not `{}`.", &service_id, type_name::<T>());
            None
        }
    }
}
