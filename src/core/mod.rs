use crate::errors::{Result, HylaeanError};
use crate::config::Config;
use sled::Db;
use std::path::PathBuf;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use log::{info, warn, error};

pub mod tool_manager;
pub mod data_manager;
pub mod agent;

pub use tool_manager::ToolManager;
pub use data_manager::DataManager;
pub use agent::Agent;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolEntry {
    pub id: String,
    pub name: String,
    pub version: String,
    pub install_path: PathBuf,
    pub repository_url: String,
    pub supported_formats: Vec<String>,
    pub dependencies: Vec<String>,
    pub last_updated: DateTime<Utc>,
    pub capabilities: ToolCapabilities,
    pub installation_method: InstallationMethod,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolCapabilities {
    pub can_train: bool,
    pub can_render: bool,
    pub can_convert: bool,
    pub supports_dynamic: bool,
    pub supports_realtime: bool,
    pub input_formats: Vec<String>,
    pub output_formats: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InstallationMethod {
    GitClone,
    PipInstall,
    CondaInstall,
    Manual,
    Binary,
}

pub struct HylaeanSplat {
    pub config: Config,
    pub database: Db,
    pub tool_manager: ToolManager,
    pub data_manager: DataManager,
    pub agent: Agent,
}

impl HylaeanSplat {
    pub async fn new() -> Result<Self> {
        info!("Initializing Hylaean Splat...");
        
        let config = Config::load_or_default()?;
        let database = sled::open(&config.database_path)?;
        let tool_manager = ToolManager::new(database.clone())?;
        let data_manager = DataManager::new()?;
        let agent = Agent::new(database.clone())?;
        
        Ok(Self {
            config,
            database,
            tool_manager,
            data_manager,
            agent,
        })
    }
    
    pub async fn initialize_config(&mut self, force: bool) -> Result<()> {
        if !force && self.config.is_initialized() {
            warn!("Hylaean Splat is already initialized. Use --force to reinitialize.");
            return Ok(());
        }
        
        self.config.initialize()?;
        self.tool_manager.initialize().await?;
        self.agent.initialize().await?;
        
        info!("Configuration initialized at: {}", self.config.config_dir.display());
        Ok(())
    }
    
    pub async fn discover_tools(&mut self, path: Option<String>) -> Result<Vec<ToolEntry>> {
        self.tool_manager.discover_tools(path).await
    }
    
    pub async fn install_tool(&mut self, name: String, path: Option<String>) -> Result<()> {
        self.tool_manager.install_tool(name, path).await
    }
    
    pub async fn remove_tool(&mut self, name: String) -> Result<()> {
        self.tool_manager.remove_tool(name).await
    }
    
    pub async fn update_tool(&mut self, name: String) -> Result<()> {
        self.tool_manager.update_tool(name).await
    }
    
    pub async fn update_all_tools(&mut self) -> Result<()> {
        self.tool_manager.update_all_tools().await
    }
    
    pub async fn show_tool_info(&self, name: String) -> Result<()> {
        self.tool_manager.show_tool_info(name).await
    }
    
    pub async fn run_tool(&mut self, name: String, args: Vec<String>) -> Result<()> {
        self.tool_manager.run_tool(name, args).await
    }
    
    pub async fn list_tools(&self, detailed: bool) -> Result<()> {
        self.tool_manager.list_tools(detailed).await
    }
    
    pub async fn convert_file(
        &mut self,
        input: String,
        output: String,
        input_format: Option<String>,
        output_format: String,
    ) -> Result<()> {
        self.data_manager.convert_file(input, output, input_format, output_format).await
    }
    
    pub async fn execute_workflow(
        &mut self,
        name: String,
        input: Option<String>,
        output: Option<String>,
    ) -> Result<()> {
        info!("Executing workflow: {}", name);
        // TODO: Implement workflow execution
        Ok(())
    }
    
    pub async fn start_agent_daemon(&mut self) -> Result<()> {
        self.agent.start_daemon().await
    }
    
    pub async fn run_agent_once(&mut self) -> Result<()> {
        self.agent.run_once().await
    }
    
    pub async fn stop_agent(&mut self) -> Result<()> {
        self.agent.stop().await
    }
    
    pub async fn show_agent_status(&self) -> Result<()> {
        self.agent.show_status().await
    }
    
    pub async fn update_agent_database(&mut self) -> Result<()> {
        self.agent.update_database().await
    }
    
    pub async fn generate_install_script(
        &mut self,
        tool: String,
        output: Option<String>,
    ) -> Result<()> {
        self.agent.generate_install_script(tool, output).await
    }
    
    pub async fn show_recommendations(&self, use_case: Option<String>) -> Result<()> {
        self.agent.show_recommendations(use_case).await
    }
}