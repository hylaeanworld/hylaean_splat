use crate::errors::{Result, HylaeanError};
use crate::core::{ToolEntry, ToolCapabilities, InstallationMethod};
use sled::Db;
use std::path::PathBuf;
use std::process::Command;
use std::collections::HashMap;
use log::{info, warn, error, debug};
use chrono::Utc;
use uuid::Uuid;
use walkdir::WalkDir;
use which::which;

pub struct ToolManager {
    db: Db,
    known_tools: HashMap<String, ToolTemplate>,
}

#[derive(Clone, Debug)]
pub struct ToolTemplate {
    pub name: String,
    pub repository_url: String,
    pub installation_method: InstallationMethod,
    pub capabilities: ToolCapabilities,
    pub detection_patterns: Vec<String>,
    pub dependencies: Vec<String>,
    pub install_script: Option<String>,
}

impl ToolManager {
    pub fn new(db: Db) -> Result<Self> {
        let mut known_tools = HashMap::new();
        
        // Initialize known tools database
        Self::populate_known_tools(&mut known_tools);
        
        Ok(Self { db, known_tools })
    }
    
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing tool manager...");
        
        // Create tools table in database
        self.db.insert(b"tools_initialized", b"true")?;
        
        Ok(())
    }
    
    fn populate_known_tools(known_tools: &mut HashMap<String, ToolTemplate>) {
        // Original 3D Gaussian Splatting
        known_tools.insert("gaussian_splatting".to_string(), ToolTemplate {
            name: "3D Gaussian Splatting".to_string(),
            repository_url: "https://repo-sam.inria.fr/fungraph/3d-gaussian-splatting/".to_string(),
            installation_method: InstallationMethod::GitClone,
            capabilities: ToolCapabilities {
                can_train: true,
                can_render: true,
                can_convert: false,
                supports_dynamic: false,
                supports_realtime: false,
                input_formats: vec!["colmap".to_string(), "nerfstudio".to_string()],
                output_formats: vec!["ply".to_string(), "splat".to_string()],
            },
            detection_patterns: vec!["train.py".to_string(), "render.py".to_string()],
            dependencies: vec!["python".to_string(), "pytorch".to_string(), "cuda".to_string()],
            install_script: None,
        });
        
        // SeaSplat
        known_tools.insert("seasplat".to_string(), ToolTemplate {
            name: "SeaSplat".to_string(),
            repository_url: "https://github.com/seasplat/seasplat".to_string(),
            installation_method: InstallationMethod::GitClone,
            capabilities: ToolCapabilities {
                can_train: true,
                can_render: true,
                can_convert: false,
                supports_dynamic: false,
                supports_realtime: false,
                input_formats: vec!["colmap".to_string()],
                output_formats: vec!["ply".to_string()],
            },
            detection_patterns: vec!["seasplat_train.py".to_string()],
            dependencies: vec!["python".to_string(), "pytorch".to_string()],
            install_script: None,
        });
        
        // SkySplat Blender
        known_tools.insert("skysplat_blender".to_string(), ToolTemplate {
            name: "SkySplat Blender".to_string(),
            repository_url: "https://github.com/kyjohnso/skysplat_blender".to_string(),
            installation_method: InstallationMethod::GitClone,
            capabilities: ToolCapabilities {
                can_train: false,
                can_render: true,
                can_convert: true,
                supports_dynamic: false,
                supports_realtime: true,
                input_formats: vec!["ply".to_string()],
                output_formats: vec!["blend".to_string()],
            },
            detection_patterns: vec!["skysplat_addon.py".to_string()],
            dependencies: vec!["blender".to_string()],
            install_script: None,
        });
        
        // Dynamic 3DGS
        known_tools.insert("dynamic_3dgs".to_string(), ToolTemplate {
            name: "Dynamic 3DGS".to_string(),
            repository_url: "https://dynamic3dgaussians.github.io/".to_string(),
            installation_method: InstallationMethod::GitClone,
            capabilities: ToolCapabilities {
                can_train: true,
                can_render: true,
                can_convert: false,
                supports_dynamic: true,
                supports_realtime: false,
                input_formats: vec!["colmap".to_string()],
                output_formats: vec!["ply".to_string()],
            },
            detection_patterns: vec!["train_dynamic.py".to_string()],
            dependencies: vec!["python".to_string(), "pytorch".to_string()],
            install_script: None,
        });
        
        // 4DGaussians
        known_tools.insert("four_d_gaussians".to_string(), ToolTemplate {
            name: "4D Gaussians".to_string(),
            repository_url: "https://github.com/hustvl/4DGaussians".to_string(),
            installation_method: InstallationMethod::GitClone,
            capabilities: ToolCapabilities {
                can_train: true,
                can_render: true,
                can_convert: false,
                supports_dynamic: true,
                supports_realtime: false,
                input_formats: vec!["colmap".to_string()],
                output_formats: vec!["ply".to_string()],
            },
            detection_patterns: vec!["train_4d.py".to_string(), "train.py".to_string()],
            dependencies: vec!["python".to_string(), "pytorch".to_string()],
            install_script: None,
        });
        
        // Brush App
        known_tools.insert("brush_app".to_string(), ToolTemplate {
            name: "Brush".to_string(),
            repository_url: "https://github.com/ArthurBrussee/brush".to_string(),
            installation_method: InstallationMethod::GitClone,
            capabilities: ToolCapabilities {
                can_train: true,
                can_render: true,
                can_convert: false,
                supports_dynamic: false,
                supports_realtime: true,
                input_formats: vec!["ply".to_string()],
                output_formats: vec!["ply".to_string()],
            },
            detection_patterns: vec!["Cargo.toml".to_string()],
            dependencies: vec!["rust".to_string()],
            install_script: None,
        });
        
        // COLMAP
        known_tools.insert("colmap".to_string(), ToolTemplate {
            name: "COLMAP".to_string(),
            repository_url: "https://colmap.github.io/".to_string(),
            installation_method: InstallationMethod::Binary,
            capabilities: ToolCapabilities {
                can_train: false,
                can_render: false,
                can_convert: true,
                supports_dynamic: false,
                supports_realtime: false,
                input_formats: vec!["images".to_string()],
                output_formats: vec!["colmap".to_string(), "ply".to_string()],
            },
            detection_patterns: vec!["colmap".to_string()],
            dependencies: vec![],
            install_script: None,
        });
    }
    
    pub async fn discover_tools(&mut self, search_path: Option<String>) -> Result<Vec<ToolEntry>> {
        info!("Discovering tools...");
        let mut discovered_tools = Vec::new();
        
        // Check for binary tools in PATH
        for (tool_id, template) in &self.known_tools {
            if let Some(tool_path) = self.find_tool_in_path(&template.name) {
                let tool_entry = ToolEntry {
                    id: Uuid::new_v4().to_string(),
                    name: template.name.clone(),
                    version: "unknown".to_string(),
                    install_path: tool_path,
                    repository_url: template.repository_url.clone(),
                    supported_formats: template.capabilities.input_formats.clone(),
                    dependencies: template.dependencies.clone(),
                    last_updated: Utc::now(),
                    capabilities: template.capabilities.clone(),
                    installation_method: template.installation_method.clone(),
                };
                
                discovered_tools.push(tool_entry);
            }
        }
        
        // Search filesystem for installed tools
        if let Some(path) = search_path {
            discovered_tools.extend(self.scan_directory(PathBuf::from(path)).await?);
        }
        
        // Register discovered tools
        for tool in &discovered_tools {
            self.register_tool(tool.clone())?;
        }
        
        info!("Discovered {} tools", discovered_tools.len());
        Ok(discovered_tools)
    }
    
    fn find_tool_in_path(&self, tool_name: &str) -> Option<PathBuf> {
        which(tool_name).ok()
    }
    
    async fn scan_directory(&self, dir: PathBuf) -> Result<Vec<ToolEntry>> {
        let mut tools = Vec::new();
        
        for entry in WalkDir::new(dir).max_depth(3) {
            let entry = entry.map_err(|e| HylaeanError::IoError(e.into()))?;
            
            if entry.file_type().is_dir() {
                if let Some(tool) = self.identify_tool_in_directory(entry.path()) {
                    tools.push(tool);
                }
            }
        }
        
        Ok(tools)
    }
    
    fn identify_tool_in_directory(&self, path: &std::path::Path) -> Option<ToolEntry> {
        for (tool_id, template) in &self.known_tools {
            for pattern in &template.detection_patterns {
                if path.join(pattern).exists() {
                    return Some(ToolEntry {
                        id: Uuid::new_v4().to_string(),
                        name: template.name.clone(),
                        version: "unknown".to_string(),
                        install_path: path.to_path_buf(),
                        repository_url: template.repository_url.clone(),
                        supported_formats: template.capabilities.input_formats.clone(),
                        dependencies: template.dependencies.clone(),
                        last_updated: Utc::now(),
                        capabilities: template.capabilities.clone(),
                        installation_method: template.installation_method.clone(),
                    });
                }
            }
        }
        None
    }
    
    fn register_tool(&self, tool: ToolEntry) -> Result<()> {
        let key = format!("tool:{}", tool.id);
        let value = serde_json::to_vec(&tool)?;
        self.db.insert(key.as_bytes(), value)?;
        info!("Registered tool: {}", tool.name);
        Ok(())
    }
    
    pub async fn install_tool(&mut self, name: String, path: Option<String>) -> Result<()> {
        info!("Installing tool: {}", name);
        
        let template = self.known_tools.get(&name)
            .ok_or_else(|| HylaeanError::ToolNotFound { name: name.clone() })?;
        
        match template.installation_method {
            InstallationMethod::GitClone => {
                self.install_via_git(template, path).await?;
            }
            InstallationMethod::Binary => {
                info!("Binary installation not yet implemented for {}", name);
            }
            _ => {
                warn!("Installation method not supported yet for {}", name);
            }
        }
        
        Ok(())
    }
    
    async fn install_via_git(&self, template: &ToolTemplate, path: Option<String>) -> Result<()> {
        let install_path = if let Some(p) = path {
            PathBuf::from(p)
        } else {
            PathBuf::from("./tools").join(&template.name)
        };
        
        info!("Cloning {} to {}", template.repository_url, install_path.display());
        
        let output = Command::new("git")
            .args(&["clone", &template.repository_url, &install_path.to_string_lossy()])
            .output()?;
        
        if !output.status.success() {
            return Err(HylaeanError::InstallationFailed {
                tool: template.name.clone(),
            });
        }
        
        info!("Successfully installed {}", template.name);
        Ok(())
    }
    
    pub async fn remove_tool(&mut self, name: String) -> Result<()> {
        info!("Removing tool: {}", name);
        // TODO: Implement tool removal
        Ok(())
    }
    
    pub async fn update_tool(&mut self, name: String) -> Result<()> {
        info!("Updating tool: {}", name);
        // TODO: Implement tool update
        Ok(())
    }
    
    pub async fn update_all_tools(&mut self) -> Result<()> {
        info!("Updating all tools...");
        // TODO: Implement update all tools
        Ok(())
    }
    
    pub async fn show_tool_info(&self, name: String) -> Result<()> {
        info!("Showing info for tool: {}", name);
        // TODO: Implement tool info display
        Ok(())
    }
    
    pub async fn run_tool(&mut self, name: String, args: Vec<String>) -> Result<()> {
        info!("Running tool: {} with args: {:?}", name, args);
        // TODO: Implement tool execution
        Ok(())
    }
    
    pub async fn list_tools(&self, detailed: bool) -> Result<()> {
        info!("Listing tools (detailed: {})", detailed);
        
        let iter = self.db.scan_prefix(b"tool:");
        for item in iter {
            let (key, value) = item?;
            let tool: ToolEntry = serde_json::from_slice(&value)?;
            
            if detailed {
                println!("Tool: {}", tool.name);
                println!("  ID: {}", tool.id);
                println!("  Version: {}", tool.version);
                println!("  Path: {}", tool.install_path.display());
                println!("  Repository: {}", tool.repository_url);
                println!("  Capabilities:");
                println!("    Train: {}", tool.capabilities.can_train);
                println!("    Render: {}", tool.capabilities.can_render);
                println!("    Convert: {}", tool.capabilities.can_convert);
                println!("    Dynamic: {}", tool.capabilities.supports_dynamic);
                println!("    Realtime: {}", tool.capabilities.supports_realtime);
                println!("  Supported formats: {:?}", tool.supported_formats);
                println!();
            } else {
                println!("{}: {} ({})", tool.name, tool.version, tool.install_path.display());
            }
        }
        
        Ok(())
    }
}