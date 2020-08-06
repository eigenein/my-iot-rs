//! Convenience functions to construct a `Value`.

use crate::prelude::*;

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl Value {
    /// Builds a `Value` instance from [kilowatt-hours](https://en.wikipedia.org/wiki/Kilowatt-hour).
    #[inline(always)]
    pub fn from_kwh(kwh: f64) -> Self {
        Value::Energy(kwh * 1000.0 * JOULES_IN_WH)
    }

    #[inline(always)]
    pub fn from_mm(mm: f64) -> Self {
        Value::Length(mm / 1000.0)
    }
}
