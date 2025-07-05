//! Integration with COLMAP for structure-from-motion and multi-view stereo

use crate::errors::{Result, HylaeanError};
use crate::integrations::{Integration, run_command_with_output};
use std::path::PathBuf;
use log::{info, debug, warn};
use which::which;

pub struct Colmap {
    executable_path: Option<PathBuf>,
}

impl Colmap {
    pub fn new() -> Self {
        let executable_path = which("colmap").ok();
        
        Self {
            executable_path,
        }
    }
    
    pub fn feature_extractor(&self, database_path: &str, image_path: &str) -> Result<()> {
        let colmap = self.executable_path.as_ref()
            .ok_or_else(|| HylaeanError::ToolNotFound {
                name: "COLMAP".to_string(),
            })?;
        
        let args = vec![
            "feature_extractor".to_string(),
            "--database_path".to_string(),
            database_path.to_string(),
            "--image_path".to_string(),
            image_path.to_string(),
        ];
        
        info!("Running COLMAP feature extraction...");
        debug!("Command: {} {}", colmap.display(), args.join(" "));
        
        let output = run_command_with_output(
            &colmap.to_string_lossy(),
            &args,
            None,
        )?;
        
        info!("Feature extraction completed successfully");
        debug!("Feature extraction output: {}", output);
        
        Ok(())
    }
    
    pub fn exhaustive_matcher(&self, database_path: &str) -> Result<()> {
        let colmap = self.executable_path.as_ref()
            .ok_or_else(|| HylaeanError::ToolNotFound {
                name: "COLMAP".to_string(),
            })?;
        
        let args = vec![
            "exhaustive_matcher".to_string(),
            "--database_path".to_string(),
            database_path.to_string(),
        ];
        
        info!("Running COLMAP exhaustive matching...");
        debug!("Command: {} {}", colmap.display(), args.join(" "));
        
        let output = run_command_with_output(
            &colmap.to_string_lossy(),
            &args,
            None,
        )?;
        
        info!("Exhaustive matching completed successfully");
        debug!("Matching output: {}", output);
        
        Ok(())
    }
    
    pub fn mapper(&self, database_path: &str, image_path: &str, output_path: &str) -> Result<()> {
        let colmap = self.executable_path.as_ref()
            .ok_or_else(|| HylaeanError::ToolNotFound {
                name: "COLMAP".to_string(),
            })?;
        
        let args = vec![
            "mapper".to_string(),
            "--database_path".to_string(),
            database_path.to_string(),
            "--image_path".to_string(),
            image_path.to_string(),
            "--output_path".to_string(),
            output_path.to_string(),
        ];
        
        info!("Running COLMAP mapping...");
        debug!("Command: {} {}", colmap.display(), args.join(" "));
        
        let output = run_command_with_output(
            &colmap.to_string_lossy(),
            &args,
            None,
        )?;
        
        info!("Mapping completed successfully");
        debug!("Mapping output: {}", output);
        
        Ok(())
    }
    
    pub fn model_converter(&self, input_path: &str, output_path: &str, output_type: &str) -> Result<()> {
        let colmap = self.executable_path.as_ref()
            .ok_or_else(|| HylaeanError::ToolNotFound {
                name: "COLMAP".to_string(),
            })?;
        
        let args = vec![
            "model_converter".to_string(),
            "--input_path".to_string(),
            input_path.to_string(),
            "--output_path".to_string(),
            output_path.to_string(),
            "--output_type".to_string(),
            output_type.to_string(),
        ];
        
        info!("Running COLMAP model conversion...");
        debug!("Command: {} {}", colmap.display(), args.join(" "));
        
        let output = run_command_with_output(
            &colmap.to_string_lossy(),
            &args,
            None,
        )?;
        
        info!("Model conversion completed successfully");
        debug!("Conversion output: {}", output);
        
        Ok(())
    }
    
    pub fn run_full_pipeline(&self, image_path: &str, output_path: &str) -> Result<()> {
        let database_path = format!("{}/database.db", output_path);
        let sparse_path = format!("{}/sparse", output_path);
        
        // Create output directory
        std::fs::create_dir_all(output_path)?;
        std::fs::create_dir_all(&sparse_path)?;
        
        // Run full pipeline
        info!("Running full COLMAP pipeline...");
        
        // 1. Feature extraction
        self.feature_extractor(&database_path, image_path)?;
        
        // 2. Feature matching
        self.exhaustive_matcher(&database_path)?;
        
        // 3. Mapping
        self.mapper(&database_path, image_path, &sparse_path)?;
        
        info!("Full COLMAP pipeline completed successfully");
        Ok(())
    }
}

impl Integration for Colmap {
    fn name(&self) -> &str {
        "COLMAP"
    }
    
    fn version(&self) -> Result<String> {
        if let Some(colmap) = &self.executable_path {
            if let Ok(output) = run_command_with_output(&colmap.to_string_lossy(), &["--version".to_string()], None) {
                return Ok(output.trim().to_string());
            }
        }
        
        Ok("unknown".to_string())
    }
    
    fn is_available(&self) -> bool {
        self.executable_path.is_some()
    }
    
    fn get_executable_path(&self) -> Option<PathBuf> {
        self.executable_path.clone()
    }
    
    fn run_command(&self, command: &str, args: &[String]) -> Result<()> {
        match command {
            "feature_extractor" => {
                if args.len() >= 2 {
                    self.feature_extractor(&args[0], &args[1])
                } else {
                    Err(HylaeanError::ToolExecutionFailed {
                        tool: self.name().to_string(),
                        message: "feature_extractor requires database_path and image_path".to_string(),
                    })
                }
            }
            "exhaustive_matcher" => {
                if args.len() >= 1 {
                    self.exhaustive_matcher(&args[0])
                } else {
                    Err(HylaeanError::ToolExecutionFailed {
                        tool: self.name().to_string(),
                        message: "exhaustive_matcher requires database_path".to_string(),
                    })
                }
            }
            "mapper" => {
                if args.len() >= 3 {
                    self.mapper(&args[0], &args[1], &args[2])
                } else {
                    Err(HylaeanError::ToolExecutionFailed {
                        tool: self.name().to_string(),
                        message: "mapper requires database_path, image_path, and output_path".to_string(),
                    })
                }
            }
            "model_converter" => {
                if args.len() >= 3 {
                    self.model_converter(&args[0], &args[1], &args[2])
                } else {
                    Err(HylaeanError::ToolExecutionFailed {
                        tool: self.name().to_string(),
                        message: "model_converter requires input_path, output_path, and output_type".to_string(),
                    })
                }
            }
            "full_pipeline" => {
                if args.len() >= 2 {
                    self.run_full_pipeline(&args[0], &args[1])
                } else {
                    Err(HylaeanError::ToolExecutionFailed {
                        tool: self.name().to_string(),
                        message: "full_pipeline requires image_path and output_path".to_string(),
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
            "feature_extractor".to_string(),
            "exhaustive_matcher".to_string(),
            "mapper".to_string(),
            "model_converter".to_string(),
            "full_pipeline".to_string(),
        ]
    }
    
    fn validate_installation(&self) -> Result<()> {
        let colmap = self.executable_path.as_ref()
            .ok_or_else(|| HylaeanError::ToolNotFound {
                name: self.name().to_string(),
            })?;
        
        // Try to run colmap with --help to validate
        if let Err(_) = run_command_with_output(&colmap.to_string_lossy(), &["--help".to_string()], None) {
            return Err(HylaeanError::ToolExecutionFailed {
                tool: self.name().to_string(),
                message: "Failed to run COLMAP executable".to_string(),
            });
        }
        
        info!("COLMAP installation validated");
        Ok(())
    }
}