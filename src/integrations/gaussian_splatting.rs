//! Integration with the original 3D Gaussian Splatting implementation

use crate::errors::{Result, HylaeanError};
use crate::integrations::{Integration, run_command_with_output};
use std::path::PathBuf;
use log::{info, debug, warn};
use which::which;

pub struct GaussianSplatting {
    install_path: Option<PathBuf>,
    python_executable: Option<PathBuf>,
}

impl GaussianSplatting {
    pub fn new() -> Self {
        let install_path = Self::find_installation();
        let python_executable = which("python").ok().or_else(|| which("python3").ok());
        
        Self {
            install_path,
            python_executable,
        }
    }
    
    fn find_installation() -> Option<PathBuf> {
        // Common installation locations
        let possible_paths = vec![
            PathBuf::from("./gaussian-splatting"),
            PathBuf::from("./3d-gaussian-splatting"),
            PathBuf::from("../gaussian-splatting"),
            PathBuf::from("../3d-gaussian-splatting"),
            dirs::home_dir()?.join("gaussian-splatting"),
            dirs::home_dir()?.join("3d-gaussian-splatting"),
        ];
        
        for path in possible_paths {
            if path.exists() && path.join("train.py").exists() {
                return Some(path);
            }
        }
        
        None
    }
    
    pub fn train(&self, data_path: &str, output_path: &str, iterations: Option<u32>) -> Result<()> {
        let install_path = self.install_path.as_ref()
            .ok_or_else(|| HylaeanError::ToolNotFound {
                name: "3D Gaussian Splatting".to_string(),
            })?;
        
        let python = self.python_executable.as_ref()
            .ok_or_else(|| HylaeanError::ToolNotFound {
                name: "python".to_string(),
            })?;
        
        let mut args = vec![
            "train.py".to_string(),
            "-s".to_string(),
            data_path.to_string(),
            "-m".to_string(),
            output_path.to_string(),
        ];
        
        if let Some(iters) = iterations {
            args.push("--iterations".to_string());
            args.push(iters.to_string());
        }
        
        info!("Starting 3D Gaussian Splatting training...");
        debug!("Command: {} {}", python.display(), args.join(" "));
        
        let output = run_command_with_output(
            &python.to_string_lossy(),
            &args,
            Some(install_path),
        )?;
        
        info!("Training completed successfully");
        debug!("Training output: {}", output);
        
        Ok(())
    }
    
    pub fn render(&self, model_path: &str, output_path: &str) -> Result<()> {
        let install_path = self.install_path.as_ref()
            .ok_or_else(|| HylaeanError::ToolNotFound {
                name: "3D Gaussian Splatting".to_string(),
            })?;
        
        let python = self.python_executable.as_ref()
            .ok_or_else(|| HylaeanError::ToolNotFound {
                name: "python".to_string(),
            })?;
        
        let args = vec![
            "render.py".to_string(),
            "-m".to_string(),
            model_path.to_string(),
            "--output_path".to_string(),
            output_path.to_string(),
        ];
        
        info!("Starting 3D Gaussian Splatting rendering...");
        debug!("Command: {} {}", python.display(), args.join(" "));
        
        let output = run_command_with_output(
            &python.to_string_lossy(),
            &args,
            Some(install_path),
        )?;
        
        info!("Rendering completed successfully");
        debug!("Render output: {}", output);
        
        Ok(())
    }
    
    pub fn convert_to_ply(&self, model_path: &str, output_path: &str) -> Result<()> {
        let install_path = self.install_path.as_ref()
            .ok_or_else(|| HylaeanError::ToolNotFound {
                name: "3D Gaussian Splatting".to_string(),
            })?;
        
        let python = self.python_executable.as_ref()
            .ok_or_else(|| HylaeanError::ToolNotFound {
                name: "python".to_string(),
            })?;
        
        let args = vec![
            "convert.py".to_string(),
            "-s".to_string(),
            model_path.to_string(),
            "--output_path".to_string(),
            output_path.to_string(),
        ];
        
        info!("Converting model to PLY format...");
        
        let output = run_command_with_output(
            &python.to_string_lossy(),
            &args,
            Some(install_path),
        )?;
        
        info!("Conversion completed successfully");
        debug!("Convert output: {}", output);
        
        Ok(())
    }
}

impl Integration for GaussianSplatting {
    fn name(&self) -> &str {
        "3D Gaussian Splatting"
    }
    
    fn version(&self) -> Result<String> {
        // Try to get version from git or package info
        if let Some(install_path) = &self.install_path {
            if let Ok(output) = run_command_with_output("git", &["rev-parse".to_string(), "--short".to_string(), "HEAD".to_string()], Some(install_path)) {
                return Ok(format!("git-{}", output.trim()));
            }
        }
        
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
                    let iterations = args.get(2).and_then(|s| s.parse().ok());
                    self.train(&args[0], &args[1], iterations)
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
            "convert" => {
                if args.len() >= 2 {
                    self.convert_to_ply(&args[0], &args[1])
                } else {
                    Err(HylaeanError::ToolExecutionFailed {
                        tool: self.name().to_string(),
                        message: "convert command requires model_path and output_path".to_string(),
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
        vec![
            "train".to_string(),
            "render".to_string(),
            "convert".to_string(),
        ]
    }
    
    fn validate_installation(&self) -> Result<()> {
        let install_path = self.install_path.as_ref()
            .ok_or_else(|| HylaeanError::ToolNotFound {
                name: self.name().to_string(),
            })?;
        
        // Check for required files
        let required_files = vec!["train.py", "render.py"];
        for file in required_files {
            if !install_path.join(file).exists() {
                return Err(HylaeanError::ToolExecutionFailed {
                    tool: self.name().to_string(),
                    message: format!("Required file {} not found", file),
                });
            }
        }
        
        // Check Python availability
        if self.python_executable.is_none() {
            return Err(HylaeanError::ToolNotFound {
                name: "python".to_string(),
            });
        }
        
        info!("3D Gaussian Splatting installation validated");
        Ok(())
    }
}