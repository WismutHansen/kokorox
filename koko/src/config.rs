//! Configuration management for koko CLI
//!
//! Configuration priority (highest to lowest):
//! 1. Command line arguments
//! 2. Config file specified via --config flag
//! 3. Environment variables (KOKO_*)
//! 4. Local config file (./config.toml)
//! 5. Global config file ($XDG_CONFIG_HOME/koko/config.toml or ~/.config/koko/config.toml)
//!
//! XDG Base Directory Specification:
//! - Config: $XDG_CONFIG_HOME or ~/.config
//! - Data: $XDG_DATA_HOME or ~/.local/share
//! - State: $XDG_STATE_HOME or ~/.local/state
//! - Cache: $XDG_CACHE_HOME or ~/.cache
//!
//! Note: XDG environment variables are checked first on ALL platforms (including macOS)

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct AppConfig {
    /// Output directory for generated audio files
    pub output_dir: String,

    /// Default language for TTS
    pub language: String,

    /// Default voice style
    pub style: String,

    /// Default speech speed
    pub speed: f32,

    /// Auto-detect language from input text
    pub auto_detect: bool,

    /// Force the specified style (don't auto-select based on language)
    pub force_style: bool,

    /// Output audio in mono format
    pub mono: bool,

    /// Path to custom model file (optional)
    pub model_path: Option<String>,

    /// Path to custom voices data file (optional)
    pub data_path: Option<String>,

    /// Model type for HuggingFace downloads (fp16, q4, q8f16, etc.)
    pub model_type: Option<String>,

    /// Initial silence duration in tokens
    pub initial_silence: Option<usize>,

    /// Enable verbose debug output
    pub verbose: bool,

    /// Enable detailed accent debugging
    pub debug_accents: bool,

    /// Server configuration
    pub server: ServerConfig,
}

/// Server-specific configuration
#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct ServerConfig {
    /// Default IP address for servers
    pub ip: String,

    /// Default port for OpenAI-compatible server
    pub openai_port: u16,

    /// Default port for WebSocket server
    pub websocket_port: u16,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            output_dir: "tmp".to_string(),
            language: "en-us".to_string(),
            style: "af_heart".to_string(),
            speed: 1.0,
            auto_detect: false,
            force_style: false,
            mono: false,
            model_path: None,
            data_path: None,
            model_type: None,
            initial_silence: None,
            verbose: false,
            debug_accents: false,
            server: ServerConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            ip: "0.0.0.0".to_string(),
            openai_port: 3000,
            websocket_port: 8766,
        }
    }
}

// ============================================================================
// XDG Base Directory helpers - checked on ALL platforms including macOS
// ============================================================================

/// Get the XDG config directory ($XDG_CONFIG_HOME or ~/.config)
/// Checks environment variable first on all platforms
pub fn xdg_config_home() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg);
        }
    }
    // Fallback to ~/.config on all platforms
    dirs::home_dir()
        .map(|h| h.join(".config"))
        .unwrap_or_else(|| PathBuf::from(".config"))
}

/// Get the XDG data directory ($XDG_DATA_HOME or ~/.local/share)
/// Checks environment variable first on all platforms
pub fn xdg_data_home() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg);
        }
    }
    // Fallback to ~/.local/share on all platforms
    dirs::home_dir()
        .map(|h| h.join(".local").join("share"))
        .unwrap_or_else(|| PathBuf::from(".local/share"))
}

/// Get the XDG state directory ($XDG_STATE_HOME or ~/.local/state)
/// Checks environment variable first on all platforms
pub fn xdg_state_home() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_STATE_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg);
        }
    }
    // Fallback to ~/.local/state on all platforms
    dirs::home_dir()
        .map(|h| h.join(".local").join("state"))
        .unwrap_or_else(|| PathBuf::from(".local/state"))
}

/// Get the XDG cache directory ($XDG_CACHE_HOME or ~/.cache)
/// Checks environment variable first on all platforms
pub fn xdg_cache_home() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg);
        }
    }
    // Fallback to ~/.cache on all platforms
    dirs::home_dir()
        .map(|h| h.join(".cache"))
        .unwrap_or_else(|| PathBuf::from(".cache"))
}

impl AppConfig {
    /// Get the global config directory path for koko
    /// Uses $XDG_CONFIG_HOME/koko if set, otherwise ~/.config/koko
    pub fn global_config_dir() -> PathBuf {
        xdg_config_home().join("koko")
    }

    /// Get the global config file path
    pub fn global_config_path() -> PathBuf {
        Self::global_config_dir().join("config.toml")
    }

    /// Get the local config file path (current directory)
    pub fn local_config_path() -> PathBuf {
        PathBuf::from("config.toml")
    }

    /// Get the data directory for koko (models, voices, etc.)
    /// Uses $XDG_DATA_HOME/koko if set, otherwise ~/.local/share/koko
    pub fn data_dir() -> PathBuf {
        xdg_data_home().join("koko")
    }

    /// Get the state directory for koko (logs, history, etc.)
    /// Uses $XDG_STATE_HOME/koko if set, otherwise ~/.local/state/koko
    pub fn state_dir() -> PathBuf {
        xdg_state_home().join("koko")
    }

    /// Get the cache directory for koko (temporary files, HF cache, etc.)
    /// Uses $XDG_CACHE_HOME/koko if set, otherwise ~/.cache/koko
    pub fn cache_dir() -> PathBuf {
        xdg_cache_home().join("koko")
    }

    /// Load configuration with proper priority chain
    ///
    /// Priority (highest to lowest):
    /// 1. Command line arguments (handled separately by clap)
    /// 2. Config file specified via --config flag
    /// 3. Environment variables (KOKO_*)
    /// 4. Local config file (./config.toml)
    /// 5. Global config file ($XDG_CONFIG_HOME/koko/config.toml)
    pub fn load(config_file: Option<&str>) -> Result<Self, ConfigError> {
        let mut builder = Config::builder();

        // 5. Start with defaults (lowest priority)
        builder = builder.add_source(config::File::from_str(
            include_str!("default_config.toml"),
            config::FileFormat::Toml,
        ));

        // 4. Global config file
        let global_path = Self::global_config_path();
        if global_path.exists() {
            builder = builder.add_source(File::from(global_path).required(false));
        }

        // 3. Local config file (./config.toml)
        let local_path = Self::local_config_path();
        if local_path.exists() {
            builder = builder.add_source(File::from(local_path).required(false));
        }

        // 2. Environment variables (KOKO_*)
        // e.g., KOKO_OUTPUT_DIR, KOKO_LANGUAGE, KOKO_SERVER__OPENAI_PORT
        // Note: Environment variable names are converted to lowercase and underscores
        // become the separator for nested keys when using separator("__")
        builder = builder.add_source(
            Environment::with_prefix("KOKO")
                .prefix_separator("_")
                .separator("__")
                .try_parsing(true),
        );

        // 1. Config file specified via --config flag (highest priority from config sources)
        if let Some(config_path) = config_file {
            let expanded = expand_path(config_path);
            builder = builder.add_source(File::with_name(&expanded).required(true));
        }

        let config = builder.build()?;
        config.try_deserialize()
    }

    /// Ensure the global config directory exists and create default config if needed
    pub fn ensure_config_exists() -> std::io::Result<()> {
        let config_dir = Self::global_config_dir();
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)?;
        }

        let config_path = config_dir.join("config.toml");
        if !config_path.exists() {
            std::fs::write(&config_path, include_str!("default_config.toml"))?;
            eprintln!("Created default config at: {}", config_path.display());
        }
        Ok(())
    }

    /// Ensure all XDG directories exist for koko
    pub fn ensure_dirs_exist() -> std::io::Result<()> {
        let dirs = [
            Self::global_config_dir(),
            Self::data_dir(),
            Self::state_dir(),
            Self::cache_dir(),
        ];

        for dir in dirs {
            if !dir.exists() {
                std::fs::create_dir_all(&dir)?;
            }
        }
        Ok(())
    }

    /// Get the output path for a given filename
    pub fn output_path(&self, filename: &str) -> String {
        let expanded_dir = expand_path(&self.output_dir);
        format!("{}/{}", expanded_dir, filename)
    }

    /// Expand the output directory path (resolve ~, env vars, etc.)
    pub fn expanded_output_dir(&self) -> String {
        expand_path(&self.output_dir)
    }

    /// Print the current configuration paths (useful for debugging)
    pub fn print_paths() {
        eprintln!("Configuration paths:");
        eprintln!("  Config dir:  {}", Self::global_config_dir().display());
        eprintln!("  Config file: {}", Self::global_config_path().display());
        eprintln!("  Data dir:    {}", Self::data_dir().display());
        eprintln!("  State dir:   {}", Self::state_dir().display());
        eprintln!("  Cache dir:   {}", Self::cache_dir().display());
    }
}

/// Expand shell-like paths (~ and environment variables)
pub fn expand_path(path: &str) -> String {
    shellexpand::full(path)
        .map(|s| s.into_owned())
        .unwrap_or_else(|_| path.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.output_dir, "tmp");
        assert_eq!(config.language, "en-us");
        assert_eq!(config.style, "af_heart");
        assert_eq!(config.speed, 1.0);
    }

    #[test]
    fn test_expand_path() {
        // Test that ~ expansion works
        let expanded = expand_path("~/test");
        assert!(!expanded.starts_with('~'));
    }
}
