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

fn debug_width_cell() -> &'static OnceLock<i32> {
    static WIDTH: OnceLock<i32> = OnceLock::new();
    &WIDTH
}

/// Sets the debug wrap width (the C# AnsiConsole profile width, which is
/// terminalWidth * 2). Called once at prompt startup.
pub fn set_debug_width(width: i32) {
    let _ = debug_width_cell().set(width);
}

/// The wrap width for debug output lines; 0 means no wrapping (unset).
pub fn debug_width() -> i32 {
    *debug_width_cell().get().unwrap_or(&0)
}
