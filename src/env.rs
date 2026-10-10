use std::collections::HashMap;
use std::path::PathBuf;

/// The environment a command runs in. It is passed in, not read in place, so a
/// test never touches the real home folder or the real variables.
pub struct Env {
    vars: HashMap<String, String>,
    home: Option<PathBuf>,
}

impl Env {
    pub fn real() -> Self {
        let vars = std::env::vars_os()
            .filter_map(|(name, value)| Some((name.into_string().ok()?, value.into_string().ok()?)))
            .collect();
        Self {
            vars,
            home: std::env::home_dir(),
        }
    }

    /// A variable that is set to something. An empty one counts as not set.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.vars
            .get(name)
            .map(String::as_str)
            .filter(|value| !value.is_empty())
    }

    /// `$XDG_CONFIG_HOME/openrouter-media`, or the same under `~/.config`.
    pub fn config_dir(&self) -> crate::Result<PathBuf> {
        let config_home = match (self.get("XDG_CONFIG_HOME"), &self.home) {
            (Some(dir), _) => PathBuf::from(dir),
            (None, Some(home)) => home.join(".config"),
            (None, None) => return Err("Cannot find your home folder. Set XDG_CONFIG_HOME.".into()),
        };
        Ok(config_home.join("openrouter-media"))
    }

    /// `$XDG_CACHE_HOME/openrouter-media`, or the same under `~/.cache`.
    /// `bin/ormedia` keeps the program there.
    pub fn cache_dir(&self) -> Option<PathBuf> {
        let cache_home = match self.get("XDG_CACHE_HOME") {
            Some(dir) => PathBuf::from(dir),
            None => self.home.as_ref()?.join(".cache"),
        };
        Some(cache_home.join("openrouter-media"))
    }
}

#[cfg(test)]
impl Env {
    pub fn fake(vars: &[(&str, &str)]) -> Self {
        let vars = vars
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect();
        Self { vars, home: None }
    }

    /// A fake with its own empty config folder and cache folder. Keep the folder alive for the test.
    pub fn temp() -> (Self, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let cache = dir.path().join("cache");
        let env = Self::fake(&[
            ("XDG_CONFIG_HOME", dir.path().to_str().unwrap()),
            ("XDG_CACHE_HOME", cache.to_str().unwrap()),
        ]);
        (env, dir)
    }

    pub fn with(mut self, name: &str, value: &str) -> Self {
        self.vars.insert(name.into(), value.into());
        self
    }
}
