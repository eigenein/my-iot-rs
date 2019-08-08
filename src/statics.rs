//! Contains compiled in static files such as favicons.

pub const FAVICON: &[u8] = include_bytes!("statics/favicon.ico");
pub const FAVICON_16: &[u8] = include_bytes!("statics/favicon-16x16.png");
pub const FAVICON_32: &[u8] = include_bytes!("statics/favicon-32x32.png");
pub const APPLE_TOUCH_ICON: &[u8] = include_bytes!("statics/apple-touch-icon.png");
pub const ANDROID_CHROME_192: &[u8] = include_bytes!("statics/android-chrome-192x192.png");
pub const ANDROID_CHROME_512: &[u8] = include_bytes!("statics/android-chrome-512x512.png");
