use crate::errors::{Result, HylaeanError};
use crate::core::{ToolEntry, ToolCapabilities, InstallationMethod};
use crate::integrations::{Integration, colmap, brush_app};
use sled::Db;
use std::path::PathBuf;
use std::process::Command;
use std::collections::HashMap;
use log::{info, warn};
use chrono::Utc;
use uuid::Uuid;
use walkdir::WalkDir;
use which::which;
use url::Url;
use regex::Regex;

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
    pub binary_names: Vec<String>, // Binary names to check for in PATH
    pub tool_type: ToolType, // How this tool should be discovered
}

#[derive(Clone, Debug)]
pub enum ToolType {
    StandaloneBinary,  // Tool is a standalone binary (e.g., colmap, brush_app)
    PythonScript,      // Tool is Python-based (e.g., gaussian_splatting)
    BlenderAddon,      // Tool is a Blender addon (e.g., skysplat_blender)
    HostDependent,     // Tool depends on host application but isn't standalone
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
            binary_names: vec!["python".to_string()], // Look for python to run train.py/render.py
            tool_type: ToolType::PythonScript,
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
            binary_names: vec!["python".to_string()], // Look for python to run seasplat scripts
            tool_type: ToolType::PythonScript,
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
            binary_names: vec!["blender".to_string()], // Dependency check only
            tool_type: ToolType::BlenderAddon, // This is an addon, not a standalone tool
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
            binary_names: vec!["python".to_string()], // Look for python to run dynamic scripts
            tool_type: ToolType::PythonScript,
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
            binary_names: vec!["python".to_string()], // Look for python to run 4D scripts
            tool_type: ToolType::PythonScript,
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
            binary_names: vec!["brush_app".to_string(), "brush".to_string()], // Look for brush binary
            tool_type: ToolType::StandaloneBinary,
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
            binary_names: vec!["colmap".to_string()], // Look for colmap binary
            tool_type: ToolType::StandaloneBinary,
        });
    }
    
    pub async fn discover_tools(&mut self, search_path: Option<String>) -> Result<Vec<ToolEntry>> {
        info!("Discovering tools...");
        let mut discovered_tools = Vec::new();
        
        // Check for binary tools in PATH (only for standalone binaries)
        for (_tool_id, template) in &self.known_tools {
            match template.tool_type {
                ToolType::StandaloneBinary => {
                    if let Some(tool_path) = self.find_tool_in_path(&template.binary_names) {
                        info!("Found {} (standalone binary) in PATH at: {}", template.name, tool_path.display());
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
                },
                ToolType::PythonScript => {
                    // For Python scripts, we only check dependencies for now
                    // The actual discovery happens through filesystem scanning
                    info!("Skipping {} (Python script) - requires filesystem discovery", template.name);
                },
                ToolType::BlenderAddon => {
                    // For Blender addons, we only discover through filesystem scanning
                    // Don't register just because Blender is installed
                    info!("Skipping {} (Blender addon) - requires filesystem discovery", template.name);
                },
                ToolType::HostDependent => {
                    // For host-dependent tools, we need special logic
                    info!("Skipping {} (host-dependent) - requires filesystem discovery", template.name);
                },
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
    
    fn find_tool_in_path(&self, binary_names: &[String]) -> Option<PathBuf> {
        for binary_name in binary_names {
            info!("Checking for binary '{}' in PATH using 'which'", binary_name);
            if let Ok(path) = which(binary_name) {
                info!("Found binary '{}' at: {}", binary_name, path.display());
                return Some(path);
            } else {
                info!("Binary '{}' not found in PATH", binary_name);
            }
        }
        None
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
    
    pub async fn install_tool(&mut self, name_or_url: String, path: Option<String>, force: bool, branch: Option<String>) -> Result<()> {
        info!("Installing tool: {}", name_or_url);
        
        // Check if it's a GitHub URL
        if self.is_github_url(&name_or_url) {
            info!("Detected GitHub URL, using GitHub installation path");
            self.install_from_github_url(name_or_url, path, force, branch).await
        } else {
            info!("Not a GitHub URL, checking known tools for: {}", name_or_url);
            // Try known tools first
            if let Some(template) = self.known_tools.get(&name_or_url).cloned() {
                info!("Found known tool template for: {}", name_or_url);
                self.install_known_tool(template, path, force, branch).await
            } else {
                info!("Tool not found in known tools: {}", name_or_url);
                Err(HylaeanError::ToolNotFound { name: name_or_url })
            }
        }
    }
    
    // Helper method to check if a string is a GitHub URL
    fn is_github_url(&self, url: &str) -> bool {
        url.starts_with("https://github.com/") || url.starts_with("http://github.com/") || url.starts_with("git@github.com:")
    }
    
    // Install tool from GitHub URL
    async fn install_from_github_url(&self, url: String, path: Option<String>, _force: bool, _branch: Option<String>) -> Result<()> {
        let install_path = if let Some(p) = path {
            PathBuf::from(p)
        } else {
            // Extract repo name from URL for default path
            let repo_name = url.split('/').last().unwrap_or("unknown").replace(".git", "");
            PathBuf::from("./tools").join(&repo_name)
        };
        
        info!("Cloning {} to {}", url, install_path.display());
        
        let output = Command::new("git")
            .args(&["clone", &url, &install_path.to_string_lossy()])
            .output()?;
        
        if !output.status.success() {
            return Err(HylaeanError::InstallationFailed {
                tool: url,
            });
        }
        
        info!("Successfully installed tool from {}", url);
        Ok(())
    }
    
    // Install known tool using template
    async fn install_known_tool(&self, template: ToolTemplate, path: Option<String>, _force: bool, _branch: Option<String>) -> Result<()> {
        info!("Installing known tool: {} using method: {:?}", template.name, template.installation_method);
        
        match template.installation_method {
            InstallationMethod::GitClone => {
                info!("Using git clone installation method");
                // Clone the repository first
                self.install_via_git(&template, path.clone()).await?;
                
                // Special handling for brush_app - build it after cloning
                if template.name == "Brush" {
                    info!("Building and installing Brush after clone");
                    self.build_and_install_brush(&template, path).await?;
                }
                
                Ok(())
            }
            InstallationMethod::Binary => {
                // For binary tools, we assume they're already installed or need manual installation
                info!("Binary tool {} should be installed manually", template.name);
                Ok(())
            }
            _ => {
                Err(HylaeanError::InstallationFailed {
                    tool: template.name,
                })
            }
        }
    }
    
    async fn install_via_git(&self, template: &ToolTemplate, path: Option<String>) -> Result<()> {
        let install_path = if let Some(p) = path {
            PathBuf::from(p)
        } else {
            // For brush_app, use a specific path since the repo name is "brush"
            if template.name == "Brush" {
                PathBuf::from("./tools")
            } else {
                PathBuf::from("./tools").join(&template.name)
            }
        };
        
        info!("Checking for existing installation at {}", install_path.display());
        
        // Check if directory already exists
        if install_path.exists() {
            info!("Directory {} already exists", install_path.display());
            // If it's already a git repo, try to update it
            if install_path.join(".git").exists() {
                info!("Found git repository, updating instead of cloning");
                
                let output = Command::new("git")
                    .args(&["pull"])
                    .current_dir(&install_path)
                    .output()?;
                
                if !output.status.success() {
                    warn!("Git pull failed, but continuing with existing repository");
                }
            } else {
                info!("Directory exists but is not a git repo, skipping clone");
            }
        } else {
            info!("Cloning {} to {}", template.repository_url, install_path.display());
            
            let output = Command::new("git")
                .args(&["clone", &template.repository_url, &install_path.to_string_lossy()])
                .output()?;
            
            if !output.status.success() {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                return Err(HylaeanError::InstallationFailed {
                    tool: format!("{} (clone failed: {})", template.name, error_msg),
                });
            }
        }
        
        info!("Successfully installed {}", template.name);
        Ok(())
    }
    
    async fn build_and_install_brush(&self, template: &ToolTemplate, path: Option<String>) -> Result<()> {
        let install_path = if let Some(p) = path {
            PathBuf::from(p)
        } else {
            // For brush_app, use the tools directory directly since the repo is cloned there
            PathBuf::from("./tools")
        };
        
        info!("Building Brush from source at {}...", install_path.display());
        
        // Check if Rust/Cargo is available
        if which("cargo").is_err() {
            return Err(HylaeanError::ToolNotFound {
                name: "cargo".to_string(),
            });
        }
        
        // Build the project in release mode
        let build_output = Command::new("cargo")
            .args(&["build", "--release"])
            .current_dir(&install_path)
            .output()?;
        
        if !build_output.status.success() {
            let error_msg = String::from_utf8_lossy(&build_output.stderr);
            return Err(HylaeanError::InstallationFailed {
                tool: format!("{} (build failed: {})", template.name, error_msg),
            });
        }
        
        info!("Brush build completed successfully");
        
        // Try to install the binary to ~/.cargo/bin or local PATH
        let binary_path = install_path.join("target/release/brush_app");
        if binary_path.exists() {
            // Check if we can install to ~/.cargo/bin
            if let Some(home_dir) = dirs::home_dir() {
                let cargo_bin = home_dir.join(".cargo/bin");
                if cargo_bin.exists() {
                    let dest_path = cargo_bin.join("brush");
                    
                    info!("Installing brush binary to {}", dest_path.display());
                    
                    // Copy the binary
                    std::fs::copy(&binary_path, &dest_path)
                        .map_err(|e| HylaeanError::IoError(e.into()))?;
                    
                    // Make it executable on Unix-like systems
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let mut perms = std::fs::metadata(&dest_path)
                            .map_err(|e| HylaeanError::IoError(e.into()))?
                            .permissions();
                        perms.set_mode(0o755);
                        std::fs::set_permissions(&dest_path, perms)
                            .map_err(|e| HylaeanError::IoError(e.into()))?;
                    }
                    
                    info!("Brush binary installed successfully to {}", dest_path.display());
                } else {
                    warn!("~/.cargo/bin not found, brush binary available at {}", binary_path.display());
                }
            } else {
                warn!("Home directory not found, brush binary available at {}", binary_path.display());
            }
        } else {
            warn!("Built binary not found at expected location: {}", binary_path.display());
        }
        
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
        
        if args.is_empty() {
            return Err(HylaeanError::ToolExecutionFailed {
                tool: name.clone(),
                message: "No command specified".to_string(),
            });
        }
        
        let command = &args[0];
        let command_args = &args[1..];
        
        match name.as_str() {
            "colmap" => {
                let colmap = colmap::Colmap::new();
                colmap.run_command(command, &command_args.to_vec())
            }
            "brush_app" | "brush" => {
                let brush = brush_app::BrushApp::new();
                brush.run_command(command, &command_args.to_vec())
            }
            _ => {
                // Try to find the tool in the registry and run it generically
                self.run_external_tool(name, args).await
            }
        }
    }
    
    async fn run_external_tool(&self, name: String, args: Vec<String>) -> Result<()> {
        // Look up tool in registry
        let iter = self.db.scan_prefix(b"tool:");
        for item in iter {
            let (_, value) = item?;
            let tool: ToolEntry = serde_json::from_slice(&value)?;
            
            if tool.name.to_lowercase() == name.to_lowercase() {
                // Found the tool, try to execute it
                let executable = tool.install_path.join(&tool.name);
                
                info!("Executing external tool: {} with args: {:?}", executable.display(), args);
                
                let output = Command::new(&executable)
                    .args(&args)
                    .output()?;
                
                if output.status.success() {
                    info!("Tool {} executed successfully", name);
                    return Ok(());
                } else {
                    let error_msg = String::from_utf8_lossy(&output.stderr);
                    return Err(HylaeanError::ToolExecutionFailed {
                        tool: name,
                        message: error_msg.to_string(),
                    });
                }
            }
        }
        
        Err(HylaeanError::ToolNotFound { name })
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