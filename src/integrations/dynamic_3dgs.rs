//! Integration with Dynamic 3D Gaussian Splatting

use crate::errors::{Result, HylaeanError};
use crate::integrations::{Integration, run_command_with_output};
use std::path::PathBuf;
use log::{info, debug, warn};
use which::which;

pub struct Dynamic3DGS {
    install_path: Option<PathBuf>,
    python_executable: Option<PathBuf>,
}

impl Dynamic3DGS {
    pub fn new() -> Self {
        let install_path = Self::find_installation();
        let python_executable = which("python").ok().or_else(|| which("python3").ok());
        
        Self {
            install_path,
            python_executable,
        }
    }
    
    fn find_installation() -> Option<PathBuf> {
        let possible_paths = vec![
            PathBuf::from("./dynamic3dgaussians"),
            PathBuf::from("./dynamic-3d-gaussians"),
            PathBuf::from("../dynamic3dgaussians"),
            dirs::home_dir()?.join("dynamic3dgaussians"),
        ];
        
        for path in possible_paths {
            if path.exists() && path.join("train.py").exists() {
                return Some(path);
            }
        }
        
        None
    }
    
    pub fn train(&self, data_path: &str, output_path: &str) -> Result<()> {
        let install_path = self.install_path.as_ref()
            .ok_or_else(|| HylaeanError::ToolNotFound {
                name: "Dynamic 3DGS".to_string(),
            })?;
        
        let python = self.python_executable.as_ref()
            .ok_or_else(|| HylaeanError::ToolNotFound {
                name: "python".to_string(),
            })?;
        
        let args = vec![
            "train.py".to_string(),
            "--source_path".to_string(),
            data_path.to_string(),
            "--model_path".to_string(),
            output_path.to_string(),
        ];
        
        info!("Starting Dynamic 3DGS training...");
        
        let output = run_command_with_output(
            &python.to_string_lossy(),
            &args,
            Some(install_path),
        )?;
        
        info!("Dynamic 3DGS training completed successfully");
        Ok(())
    }
}

impl Integration for Dynamic3DGS {
    fn name(&self) -> &str {
        "Dynamic 3DGS"
    }
    
    fn version(&self) -> Result<String> {
        Ok("unknown".to_string())
    }
    
    fn is_available(&self) -> bool {
        self.install_path.is_some() && self.python_executable.is_some()
    }
    
    fn get_executable_path(&self) -> Option<PathBuf> {
        self.python_executable.clone()
    }
    
    fn run_command(&self, command: &str, args: &[String]) -> Result<()> {
        match command {
            "train" => {
                if args.len() >= 2 {
                    self.train(&args[0], &args[1])
                } else {
                    Err(HylaeanError::ToolExecutionFailed {
                        tool: self.name().to_string(),
                        message: "train command requires data_path and output_path".to_string(),
                    })
                }
            }
            _ => Err(HylaeanError::ToolExecutionFailed {
                tool: self.name().to_string(),
                message: format!("Unknown command: {}", command),
            }),
        }
    }
    
    fn get_supported_commands(&self) -> Vec<String> {
        vec!["train".to_string()]
    }
    
    fn validate_installation(&self) -> Result<()> {
        if self.install_path.is_none() {
            return Err(HylaeanError::ToolNotFound {
                name: self.name().to_string(),
            });
        }
        Ok(())
    }
}