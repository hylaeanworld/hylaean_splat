//! Integrations with various 3D Gaussian splatting tools and frameworks

pub mod gaussian_splatting;
pub mod seasplat;
pub mod skysplat;
pub mod dynamic_3dgs;
pub mod four_d_gaussians;
pub mod brush_app;
pub mod colmap;

use crate::errors::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolIntegration {
    pub name: String,
    pub version: String,
    pub executable_path: PathBuf,
    pub config_path: Option<PathBuf>,
    pub supported_commands: Vec<String>,
}

pub trait Integration {
    fn name(&self) -> &str;
    fn version(&self) -> Result<String>;
    fn is_available(&self) -> bool;
    fn get_executable_path(&self) -> Option<PathBuf>;
    fn run_command(&self, command: &str, args: &[String]) -> Result<()>;
    fn get_supported_commands(&self) -> Vec<String>;
    fn validate_installation(&self) -> Result<()>;
}

pub fn detect_installed_tools() -> Result<Vec<Box<dyn Integration>>> {
    let mut tools: Vec<Box<dyn Integration>> = Vec::new();
    
    // Check for each tool
    let gaussian_splatting = gaussian_splatting::GaussianSplatting::new();
    if gaussian_splatting.is_available() {
        tools.push(Box::new(gaussian_splatting));
    }
    
    let seasplat = seasplat::SeaSplat::new();
    if seasplat.is_available() {
        tools.push(Box::new(seasplat));
    }
    
    let skysplat = skysplat::SkySplat::new();
    if skysplat.is_available() {
        tools.push(Box::new(skysplat));
    }
    
    let dynamic_3dgs = dynamic_3dgs::Dynamic3DGS::new();
    if dynamic_3dgs.is_available() {
        tools.push(Box::new(dynamic_3dgs));
    }
    
    let four_d_gaussians = four_d_gaussians::FourDGaussians::new();
    if four_d_gaussians.is_available() {
        tools.push(Box::new(four_d_gaussians));
    }
    
    let brush_app = brush_app::BrushApp::new();
    if brush_app.is_available() {
        tools.push(Box::new(brush_app));
    }
    
    let colmap = colmap::Colmap::new();
    if colmap.is_available() {
        tools.push(Box::new(colmap));
    }
    
    Ok(tools)
}

pub fn run_command_with_output(command: &str, args: &[String], working_dir: Option<&PathBuf>) -> Result<String> {
    let mut cmd = Command::new(command);
    cmd.args(args);
    
    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }
    
    let output = cmd.output()?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        Err(crate::errors::HylaeanError::ToolExecutionFailed {
            tool: command.to_string(),
            message: error_msg.to_string(),
        })
    }
}