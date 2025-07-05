//! Integration with SkySplat Blender addon

use crate::errors::{Result, HylaeanError};
use crate::integrations::{Integration, run_command_with_output};
use std::path::PathBuf;
use log::{info, debug, warn};
use which::which;

pub struct SkySplat {
    install_path: Option<PathBuf>,
    blender_executable: Option<PathBuf>,
}

impl SkySplat {
    pub fn new() -> Self {
        let install_path = Self::find_installation();
        let blender_executable = which("blender").ok();
        
        Self {
            install_path,
            blender_executable,
        }
    }
    
    fn find_installation() -> Option<PathBuf> {
        let possible_paths = vec![
            PathBuf::from("./skysplat_blender"),
            PathBuf::from("../skysplat_blender"),
            dirs::home_dir()?.join("skysplat_blender"),
        ];
        
        for path in possible_paths {
            if path.exists() && path.join("__init__.py").exists() {
                return Some(path);
            }
        }
        
        None
    }
    
    pub fn install_addon(&self) -> Result<()> {
        let install_path = self.install_path.as_ref()
            .ok_or_else(|| HylaeanError::ToolNotFound {
                name: "SkySplat".to_string(),
            })?;
        
        let blender = self.blender_executable.as_ref()
            .ok_or_else(|| HylaeanError::ToolNotFound {
                name: "blender".to_string(),
            })?;
        
        info!("Installing SkySplat addon to Blender...");
        
        // Create a Python script to install the addon
        let install_script = format!(
            r#"
import bpy
import sys
sys.path.append('{}')
bpy.ops.preferences.addon_install(filepath='{}')
bpy.ops.preferences.addon_enable(module='skysplat_blender')
bpy.ops.wm.save_userpref()
"#,
            install_path.display(),
            install_path.join("__init__.py").display()
        );
        
        let script_path = std::env::temp_dir().join("install_skysplat.py");
        std::fs::write(&script_path, install_script)?;
        
        let args = vec![
            "--background".to_string(),
            "--python".to_string(),
            script_path.to_string_lossy().to_string(),
        ];
        
        let output = run_command_with_output(
            &blender.to_string_lossy(),
            &args,
            None,
        )?;
        
        info!("SkySplat addon installed successfully");
        debug!("Installation output: {}", output);
        
        Ok(())
    }
    
    pub fn render_splat(&self, splat_file: &str, output_path: &str) -> Result<()> {
        let blender = self.blender_executable.as_ref()
            .ok_or_else(|| HylaeanError::ToolNotFound {
                name: "blender".to_string(),
            })?;
        
        info!("Rendering splat file with SkySplat...");
        
        // Create a Python script to render the splat
        let render_script = format!(
            r#"
import bpy
import skysplat_blender

# Load the splat file
skysplat_blender.load_splat('{}')

# Set up rendering
bpy.context.scene.render.filepath = '{}'
bpy.ops.render.render(write_still=True)
"#,
            splat_file,
            output_path
        );
        
        let script_path = std::env::temp_dir().join("render_skysplat.py");
        std::fs::write(&script_path, render_script)?;
        
        let args = vec![
            "--background".to_string(),
            "--python".to_string(),
            script_path.to_string_lossy().to_string(),
        ];
        
        let output = run_command_with_output(
            &blender.to_string_lossy(),
            &args,
            None,
        )?;
        
        info!("SkySplat rendering completed successfully");
        debug!("Render output: {}", output);
        
        Ok(())
    }
}

impl Integration for SkySplat {
    fn name(&self) -> &str {
        "SkySplat"
    }
    
    fn version(&self) -> Result<String> {
        if let Some(install_path) = &self.install_path {
            if let Ok(output) = run_command_with_output("git", &["rev-parse".to_string(), "--short".to_string(), "HEAD".to_string()], Some(install_path)) {
                return Ok(format!("git-{}", output.trim()));
            }
        }
        Ok("unknown".to_string())
    }
    
    fn is_available(&self) -> bool {
        self.install_path.is_some() && self.blender_executable.is_some()
    }
    
    fn get_executable_path(&self) -> Option<PathBuf> {
        self.blender_executable.clone()
    }
    
    fn run_command(&self, command: &str, args: &[String]) -> Result<()> {
        match command {
            "install" => {
                self.install_addon()
            }
            "render" => {
                if args.len() >= 2 {
                    self.render_splat(&args[0], &args[1])
                } else {
                    Err(HylaeanError::ToolExecutionFailed {
                        tool: self.name().to_string(),
                        message: "render command requires splat_file and output_path".to_string(),
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
            "install".to_string(),
            "render".to_string(),
        ]
    }
    
    fn validate_installation(&self) -> Result<()> {
        if self.install_path.is_none() {
            return Err(HylaeanError::ToolNotFound {
                name: self.name().to_string(),
            });
        }
        
        if self.blender_executable.is_none() {
            return Err(HylaeanError::ToolNotFound {
                name: "blender".to_string(),
            });
        }
        
        info!("SkySplat installation validated");
        Ok(())
    }
}