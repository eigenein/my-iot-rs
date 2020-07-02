/// Format value to keep only 3 digits before the decimal point.
pub fn human_format(value: f64, unit: &str) -> String {
    if value.is_zero() {
        return format!("{:.1} {}", value, unit);
    }
    let abs_value = value.abs();
    match abs_value {
        _ if abs_value < 1e-21 => format!("{:.1} y{}", value * 1e24, unit),
        _ if abs_value < 1e-18 => format!("{:.1} z{}", value * 1e21, unit),
        _ if abs_value < 1e-15 => format!("{:.1} a{}", value * 1e18, unit),
        _ if abs_value < 1e-12 => format!("{:.1} f{}", value * 1e15, unit),
        _ if abs_value < 1e-9 => format!("{:.1} p{}", value * 1e12, unit),
        _ if abs_value < 1e-6 => format!("{:.1} n{}", value * 1e9, unit),
        _ if abs_value < 1e-3 => format!("{:.1} µ{}", value * 1e6, unit),
        _ if abs_value < 1.0 => format!("{:.1} m{}", value * 1e3, unit),
        _ if abs_value < 1e3 => format!("{:.1} {}", value, unit),
        _ if abs_value < 1e6 => format!("{:.1} k{}", value * 1e-3, unit),
        _ if abs_value < 1e9 => format!("{:.1} M{}", value * 1e-6, unit),
        _ if abs_value < 1e12 => format!("{:.1} G{}", value * 1e-9, unit),
        _ if abs_value < 1e15 => format!("{:.1} T{}", value * 1e-12, unit),
        _ if abs_value < 1e18 => format!("{:.1} P{}", value * 1e-15, unit),
        _ if abs_value < 1e21 => format!("{:.1} E{}", value * 1e-18, unit),
        _ if abs_value < 1e24 => format!("{:.1} Z{}", value * 1e-21, unit),
        _ => format!("{:.1} Y{}", value * 1e-24, unit),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metres() {
        assert_eq!(human_format(100.0, "m"), "100.0 m");
    }

    #[test]
    fn megametres() {
        assert_eq!(human_format(12.756e6, "m"), "12.8 Mm");
    }

    #[test]
    fn millimetres() {
        assert_eq!(human_format(0.005, "m"), "5.0 mm");
    }

    #[test]
    fn negative() {
        assert_eq!(human_format(-793.0, "W"), "-793.0 W");
    }
}
