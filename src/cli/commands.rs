use crate::cli::{Commands, ToolAction, AgentAction};
use crate::core::HylaeanSplat;
use crate::errors::Result;
use log::{info, warn};

impl HylaeanSplat {
    pub async fn execute_command(&mut self, command: Commands) -> Result<()> {
        match command {
            Commands::Init { force } => {
                self.initialize_config(force).await?;
                info!("Hylaean Splat initialized successfully");
            }
            
            Commands::Tool { action } => {
                self.execute_tool_action(action).await?;
            }
            
            Commands::Convert { 
                input, 
                output, 
                input_format, 
                output_format 
            } => {
                self.convert_file(input, output, input_format, output_format).await?;
            }
            
            Commands::Agent { action } => {
                self.execute_agent_action(action).await?;
            }
            
            Commands::List { detailed } => {
                self.list_tools(detailed).await?;
            }
            
            Commands::Workflow { name, input, output } => {
                self.execute_workflow(name, input, output).await?;
            }
        }
        
        Ok(())
    }
    
    async fn execute_tool_action(&mut self, action: ToolAction) -> Result<()> {
        match action {
            ToolAction::Discover { path } => {
                let discovered = self.discover_tools(path).await?;
                info!("Discovered {} tools", discovered.len());
                for tool in discovered {
                    println!("  - {}: {}", tool.name, tool.install_path.display());
                }
            }
            
            ToolAction::Install { name, path } => {
                self.install_tool(name, path).await?;
            }
            
            ToolAction::Remove { name } => {
                self.remove_tool(name).await?;
            }
            
            ToolAction::Update { name } => {
                if let Some(tool_name) = name {
                    self.update_tool(tool_name).await?;
                } else {
                    self.update_all_tools().await?;
                }
            }
            
            ToolAction::Info { name } => {
                self.show_tool_info(name).await?;
            }
            
            ToolAction::Run { name, args } => {
                self.run_tool(name, args).await?;
            }
        }
        
        Ok(())
    }
    
    async fn execute_agent_action(&mut self, action: AgentAction) -> Result<()> {
        match action {
            AgentAction::Start { daemon } => {
                if daemon {
                    self.start_agent_daemon().await?;
                } else {
                    self.run_agent_once().await?;
                }
            }
            
            AgentAction::Stop => {
                self.stop_agent().await?;
            }
            
            AgentAction::Status => {
                self.show_agent_status().await?;
            }
            
            AgentAction::Update => {
                self.update_agent_database().await?;
            }
            
            AgentAction::Generate { tool, output } => {
                self.generate_install_script(tool, output).await?;
            }
            
            AgentAction::Recommend { use_case } => {
                self.show_recommendations(use_case).await?;
            }
        }
        
        Ok(())
    }
}