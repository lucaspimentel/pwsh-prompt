// Static settings, mirroring the C# Settings class.

use std::env;
use std::sync::OnceLock;

pub const LAST_COMMAND_DURATION_THRESHOLD_MS: i32 = 30;

/// The prompt glyph, rendered in lime.
pub const PROMPT: &str = " \u{276F} ";

fn debug_enabled() -> bool {
    static DEBUG: OnceLock<bool> = OnceLock::new();
    *DEBUG.get_or_init(|| env::var("DEBUG_PROMPT").is_ok_and(|v| v == "1"))
}

/// True when the DEBUG_PROMPT environment variable equals "1".
pub fn debug() -> bool {
    debug_enabled()
}
