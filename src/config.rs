use crate::errors::{Result, HylaeanError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use log::{info, debug};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub config_dir: PathBuf,
    pub database_path: PathBuf,
    pub tools_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub agent_config: AgentConfig,
    pub format_config: FormatConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentConfig {
    pub enabled: bool,
    pub update_interval_hours: u64,
    pub github_token: Option<String>,
    pub arxiv_search_terms: Vec<String>,
    pub max_repos_to_track: usize,
    pub auto_install_recommendations: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FormatConfig {
    pub default_point_cloud_format: String,
    pub default_camera_format: String,
    pub conversion_cache_size_mb: usize,
    pub preserve_metadata: bool,
}

impl Default for Config {
    fn default() -> Self {
        let home_dir = dirs::home_dir().expect("Failed to get home directory");
        let config_dir = home_dir.join(".hylaean_splat");
        
        Self {
            database_path: config_dir.join("database"),
            tools_dir: config_dir.join("tools"),
            cache_dir: config_dir.join("cache"),
            config_dir,
            agent_config: AgentConfig::default(),
            format_config: FormatConfig::default(),
        }
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            update_interval_hours: 24,
            github_token: None,
            arxiv_search_terms: vec![
                "3d gaussian splatting".to_string(),
                "gaussian splatting".to_string(),
                "neural radiance field".to_string(),
                "nerf".to_string(),
            ],
            max_repos_to_track: 1000,
            auto_install_recommendations: false,
        }
    }
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            default_point_cloud_format: "ply".to_string(),
            default_camera_format: "colmap".to_string(),
            conversion_cache_size_mb: 1024,
            preserve_metadata: true,
        }
    }
}

impl Config {
    pub fn load_or_default() -> Result<Self> {
        let config = Self::default();
        
        let config_file = config.config_dir.join("config.toml");
        
        if config_file.exists() {
            debug!("Loading config from: {}", config_file.display());
            let content = fs::read_to_string(&config_file)?;
            let loaded_config: Config = toml::from_str(&content)
                .map_err(|e| HylaeanError::ConfigError {
                    message: format!("Failed to parse config file: {}", e),
                })?;
            Ok(loaded_config)
        } else {
            debug!("Using default configuration");
            Ok(config)
        }
    }
    
    pub fn save(&self) -> Result<()> {
        fs::create_dir_all(&self.config_dir)?;
        
        let config_file = self.config_dir.join("config.toml");
        let content = toml::to_string_pretty(self)
            .map_err(|e| HylaeanError::ConfigError {
                message: format!("Failed to serialize config: {}", e),
            })?;
        
        fs::write(&config_file, content)?;
        info!("Configuration saved to: {}", config_file.display());
        Ok(())
    }
    
    pub fn initialize(&mut self) -> Result<()> {
        // Create necessary directories
        fs::create_dir_all(&self.config_dir)?;
        fs::create_dir_all(&self.tools_dir)?;
        fs::create_dir_all(&self.cache_dir)?;
        fs::create_dir_all(&self.database_path)?;
        
        // Save the configuration
        self.save()?;
        
        info!("Hylaean Splat directories created:");
        info!("  Config: {}", self.config_dir.display());
        info!("  Tools: {}", self.tools_dir.display());
        info!("  Cache: {}", self.cache_dir.display());
        info!("  Database: {}", self.database_path.display());
        
        Ok(())
    }
    
    pub fn is_initialized(&self) -> bool {
        self.config_dir.exists() && 
        self.config_dir.join("config.toml").exists()
    }
    
    pub fn get_tool_install_path(&self, tool_name: &str) -> PathBuf {
        self.tools_dir.join(tool_name)
    }
    
    pub fn get_cache_path(&self, cache_type: &str) -> PathBuf {
        self.cache_dir.join(cache_type)
    }
}