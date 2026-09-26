// Test-only helpers. Unit tests mutate process-wide environment variables
// (PROMPT_*), which are shared across parallel test threads, so every test
// that depends on them must hold the guard and clear them first.

#[cfg(test)]
pub static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
pub struct PromptEnvGuard {
    saved: Vec<(String, String)>,
}

#[cfg(test)]
impl PromptEnvGuard {
    pub fn clear() -> Self {
        const NAMES: [&str; 10] = [
            "PROMPT_GIT_DIR_CACHED",
            "PROMPT_GIT_BRANCH_CACHED",
            "PROMPT_PR_NUMBER_CACHED",
            "PROMPT_PR_STATE_CACHED",
            "PROMPT_GIT_DIR",
            "PROMPT_GIT_BRANCH",
            "PROMPT_GIT_HEAD",
            "PROMPT_GIT_CACHE_DIR",
            "PROMPT_PR_NUMBER",
            "PROMPT_PR_STATE",
        ];
        let saved: Vec<(String, String)> = NAMES
            .iter()
            .filter_map(|n| std::env::var(n).ok().map(|v| (n.to_string(), v)))
            .collect();
        for name in NAMES {
            remove_var(name);
        }
        PromptEnvGuard { saved }
    }
}

#[cfg(test)]
impl Drop for PromptEnvGuard {
    fn drop(&mut self) {
        for (name, value) in &self.saved {
            set_var(name, value);
        }
    }
}

// Environment mutation is unsafe since Rust 2024. Every call site holds
// ENV_LOCK, so mutations are serialized and no other thread observes a
// partially updated environment.
#[cfg(test)]
pub fn set_var(key: &str, value: &str) {
    // SAFETY: callers hold ENV_LOCK, serializing all env mutations in tests.
    unsafe { std::env::set_var(key, value) };
}

#[cfg(test)]
pub fn remove_var(key: &str) {
    // SAFETY: callers hold ENV_LOCK, serializing all env mutations in tests.
    unsafe { std::env::remove_var(key) };
}
