// Environment variable access, injected so the code that reads PROMPT_* and
// home-directory variables can be unit tested without touching the
// process-wide environment.

/// Source of environment variables.
pub trait EnvSource {
    fn var(&self, name: &str) -> Option<String>;
}

/// Reads the real process environment.
pub struct RealEnv;

impl EnvSource for RealEnv {
    fn var(&self, name: &str) -> Option<String> {
        std::env::var(name).ok()
    }
}

#[cfg(test)]
#[derive(Clone)]
pub struct FakeEnv {
    pub vars: std::collections::HashMap<String, String>,
}

#[cfg(test)]
impl FakeEnv {
    pub fn empty() -> Self {
        FakeEnv {
            vars: std::collections::HashMap::new(),
        }
    }

    pub fn with(mut self, name: &str, value: &str) -> Self {
        self.vars.insert(name.to_string(), value.to_string());
        self
    }
}

#[cfg(test)]
impl EnvSource for FakeEnv {
    fn var(&self, name: &str) -> Option<String> {
        self.vars.get(name).cloned()
    }
}
