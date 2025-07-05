//! Integration with Brush - a Rust-based 3D Gaussian Splatting renderer

use crate::errors::{Result, HylaeanError};
use crate::integrations::{Integration, run_command_with_output};
use std::path::PathBuf;
use log::{info, debug, warn};
use which::which;

pub struct BrushApp {
    install_path: Option<PathBuf>,
    executable_path: Option<PathBuf>,
}

impl BrushApp {
    pub fn new() -> Self {
        let install_path = Self::find_installation();
        let executable_path = which("brush").ok()
            .or_else(|| install_path.as_ref().and_then(|p| {
                let target_path = p.join("target/release/brush");
                if target_path.exists() {
                    Some(target_path)
                } else {
                    None
                }
            }));
        
        Self {
            install_path,
            executable_path,
        }
    }
    
    fn find_installation() -> Option<PathBuf> {
        // Common installation locations
        let possible_paths = vec![
            PathBuf::from("./brush"),
            PathBuf::from("../brush"),
            dirs::home_dir()?.join("brush"),
            PathBuf::from("./tools/brush"),
        ];
        
        for path in possible_paths {
            if path.exists() && path.join("Cargo.toml").exists() {
                // Check if it's actually the Brush project
                if let Ok(content) = std::fs::read_to_string(path.join("Cargo.toml")) {
                    if content.contains("brush") && content.contains("gaussian") {
                        return Some(path);
                    }
                }
            }
        }
        
        None
    }
    
    pub fn train(&self, data_path: &str, output_path: &str) -> Result<()> {
        let brush = self.executable_path.as_ref()
            .ok_or_else(|| HylaeanError::ToolNotFound {
                name: "Brush".to_string(),
            })?;
        
        let args = vec![
            "train".to_string(),
            "--data".to_string(),
            data_path.to_string(),
            "--output".to_string(),
            output_path.to_string(),
        ];
        
        info!("Starting Brush training...");
        debug!("Command: {} {}", brush.display(), args.join(" "));
        
        let output = run_command_with_output(
            &brush.to_string_lossy(),
            &args,
            None,
        )?;
        
        info!("Brush training completed successfully");
        debug!("Training output: {}", output);
        
        Ok(())
    }
    
    pub fn render(&self, model_path: &str, output_path: &str) -> Result<()> {
        let brush = self.executable_path.as_ref()
            .ok_or_else(|| HylaeanError::ToolNotFound {
                name: "Brush".to_string(),
            })?;
        
        let args = vec![
            "render".to_string(),
            "--model".to_string(),
            model_path.to_string(),
            "--output".to_string(),
            output_path.to_string(),
        ];
        
        info!("Starting Brush rendering...");
        debug!("Command: {} {}", brush.display(), args.join(" "));
        
        let output = run_command_with_output(
            &brush.to_string_lossy(),
            &args,
            None,
        )?;
        
        info!("Brush rendering completed successfully");
        debug!("Render output: {}", output);
        
        Ok(())
    }
    
    pub fn viewer(&self, model_path: &str) -> Result<()> {
        let brush = self.executable_path.as_ref()
            .ok_or_else(|| HylaeanError::ToolNotFound {
                name: "Brush".to_string(),
            })?;
        
        let args = vec![
            "viewer".to_string(),
            "--model".to_string(),
            model_path.to_string(),
        ];
        
        info!("Starting Brush viewer...");
        debug!("Command: {} {}", brush.display(), args.join(" "));
        
        let output = run_command_with_output(
            &brush.to_string_lossy(),
            &args,
            None,
        )?;
        
        info!("Brush viewer started successfully");
        debug!("Viewer output: {}", output);
        
        Ok(())
    }
    
    pub fn build(&self) -> Result<()> {
        let install_path = self.install_path.as_ref()
            .ok_or_else(|| HylaeanError::ToolNotFound {
                name: "Brush".to_string(),
            })?;
        
        info!("Building Brush from source...");
        
        let output = run_command_with_output(
            "cargo",
            &["build", "--release"].iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            Some(install_path),
        )?;
        
        info!("Brush build completed successfully");
        debug!("Build output: {}", output);
        
        Ok(())
    }
}

impl Integration for BrushApp {
    fn name(&self) -> &str {
        "Brush"
    }
    
    fn version(&self) -> Result<String> {
        if let Some(install_path) = &self.install_path {
            // Try to get version from Cargo.toml
            let cargo_toml = install_path.join("Cargo.toml");
            if let Ok(content) = std::fs::read_to_string(cargo_toml) {
                for line in content.lines() {
                    if line.starts_with("version") {
                        if let Some(version) = line.split('=').nth(1) {
                            return Ok(version.trim().trim_matches('"').to_string());
                        }
                    }
                }
            }
            
            // Try to get version from git
            if let Ok(output) = run_command_with_output("git", &["rev-parse".to_string(), "--short".to_string(), "HEAD".to_string()], Some(install_path)) {
                return Ok(format!("git-{}", output.trim()));
            }
        }
        
        Ok("unknown".to_string())
    }
    
    fn is_available(&self) -> bool {
        self.executable_path.is_some() || self.install_path.is_some()
    }
    
    fn get_executable_path(&self) -> Option<PathBuf> {
        self.executable_path.clone()
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
            "render" => {
                if args.len() >= 2 {
                    self.render(&args[0], &args[1])
                } else {
                    Err(HylaeanError::ToolExecutionFailed {
                        tool: self.name().to_string(),
                        message: "render command requires model_path and output_path".to_string(),
                    })
                }
            }
            "viewer" => {
                if args.len() >= 1 {
                    self.viewer(&args[0])
                } else {
                    Err(HylaeanError::ToolExecutionFailed {
                        tool: self.name().to_string(),
                        message: "viewer command requires model_path".to_string(),
                    })
                }
            }
            "build" => {
                self.build()
            }
            _ => Err(HylaeanError::ToolExecutionFailed {
                tool: self.name().to_string(),
                message: format!("Unknown command: {}", command),
            }),
        }
    }
    
    fn get_supported_commands(&self) -> Vec<String> {
        vec![
            "train".to_string(),
            "render".to_string(),
            "viewer".to_string(),
            "build".to_string(),
        ]
    }
    
    fn validate_installation(&self) -> Result<()> {
        if let Some(install_path) = &self.install_path {
            // Check for Cargo.toml
            if !install_path.join("Cargo.toml").exists() {
                return Err(HylaeanError::ToolExecutionFailed {
                    tool: self.name().to_string(),
                    message: "Cargo.toml not found".to_string(),
                });
            }
            
            // Check if cargo is available
            if which("cargo").is_err() {
                return Err(HylaeanError::ToolNotFound {
                    name: "cargo".to_string(),
                });
            }
        } else if self.executable_path.is_none() {
            return Err(HylaeanError::ToolNotFound {
                name: self.name().to_string(),
            });
        }
        
        info!("Brush installation validated");
        Ok(())
    }
}