use crate::errors::{Result, HylaeanError};
use sled::Db;
use std::path::PathBuf;
use std::collections::HashMap;
use log::{info, warn, error, debug};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::fs::File;
use std::io::Write;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInfo {
    pub id: String,
    pub name: String,
    pub url: String,
    pub description: Option<String>,
    pub stars: u32,
    pub forks: u32,
    pub language: Option<String>,
    pub last_updated: DateTime<Utc>,
    pub topics: Vec<String>,
    pub installation_detected: bool,
    pub installation_method: Option<String>,
    pub requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperInfo {
    pub id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub abstract_text: String,
    pub arxiv_id: Option<String>,
    pub published_date: DateTime<Utc>,
    pub keywords: Vec<String>,
    pub repository_urls: Vec<String>,
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub id: String,
    pub tool_name: String,
    pub reason: String,
    pub confidence: f32,
    pub use_case: String,
    pub installation_difficulty: RecommendationDifficulty,
    pub prerequisites: Vec<String>,
    pub estimated_setup_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationDifficulty {
    Easy,
    Medium,
    Hard,
    Expert,
}

pub struct Agent {
    db: Db,
    client: Client,
    github_token: Option<String>,
    is_running: bool,
}

impl Agent {
    pub fn new(db: Db) -> Result<Self> {
        let client = Client::new();
        
        Ok(Self {
            db,
            client,
            github_token: std::env::var("GITHUB_TOKEN").ok(),
            is_running: false,
        })
    }
    
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing agentic component...");
        
        // Create agent tables in database
        self.db.insert(b"agent_initialized", b"true")?;
        self.db.insert(b"agent_last_run", Utc::now().to_rfc3339().as_bytes())?;
        
        // Initialize with known repositories
        self.initialize_known_repositories().await?;
        
        Ok(())
    }
    
    async fn initialize_known_repositories(&mut self) -> Result<()> {
        let known_repos = vec![
            ("gaussian_splatting", "https://github.com/graphdeco-inria/gaussian-splatting"),
            ("seasplat", "https://github.com/seasplat/seasplat"),
            ("skysplat_blender", "https://github.com/kyjohnso/skysplat_blender"),
            ("dynamic_3dgs", "https://github.com/jonathsch/dynamic3dgaussians"),
            ("four_d_gaussians", "https://github.com/hustvl/4DGaussians"),
            ("brush_app", "https://github.com/ArthurBrussee/brush"),
            ("nerfstudio", "https://github.com/nerfstudio-project/nerfstudio"),
            ("threestudio", "https://github.com/threestudio-project/threestudio"),
            ("gaussian_splatting_lightning", "https://github.com/yzslab/gaussian-splatting-lightning"),
            ("taichi_3d_gaussian_splatting", "https://github.com/wanmeihuali/taichi_3d_gaussian_splatting"),
        ];
        
        for (name, url) in known_repos {
            if let Ok(repo_info) = self.fetch_repository_info(url).await {
                self.store_repository_info(&repo_info)?;
                info!("Stored repository info: {}", name);
            }
        }
        
        Ok(())
    }
    
    pub async fn start_daemon(&mut self) -> Result<()> {
        info!("Starting agent daemon...");
        self.is_running = true;
        
        // In a real implementation, this would run in a separate thread
        // For now, we'll just run once
        self.run_once().await?;
        
        Ok(())
    }
    
    pub async fn run_once(&mut self) -> Result<()> {
        info!("Running agent update cycle...");
        
        // Update repository information
        self.update_repositories().await?;
        
        // Search for new papers
        self.search_arxiv_papers().await?;
        
        // Generate recommendations
        self.generate_recommendations().await?;
        
        // Update last run time
        self.db.insert(b"agent_last_run", Utc::now().to_rfc3339().as_bytes())?;
        
        info!("Agent update cycle completed");
        Ok(())
    }
    
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping agent...");
        self.is_running = false;
        Ok(())
    }
    
    pub async fn show_status(&self) -> Result<()> {
        println!("Agent Status:");
        println!("  Running: {}", self.is_running);
        
        if let Ok(Some(last_run)) = self.db.get(b"agent_last_run") {
            let last_run_str = String::from_utf8_lossy(&last_run);
            println!("  Last run: {}", last_run_str);
        }
        
        // Count repositories
        let repo_count = self.db.scan_prefix(b"repo:").count();
        println!("  Tracked repositories: {}", repo_count);
        
        // Count papers
        let paper_count = self.db.scan_prefix(b"paper:").count();
        println!("  Tracked papers: {}", paper_count);
        
        // Count recommendations
        let rec_count = self.db.scan_prefix(b"recommendation:").count();
        println!("  Active recommendations: {}", rec_count);
        
        Ok(())
    }
    
    pub async fn update_database(&mut self) -> Result<()> {
        info!("Updating agent database...");
        self.run_once().await?;
        Ok(())
    }
    
    async fn update_repositories(&mut self) -> Result<()> {
        info!("Updating repository information...");
        
        let repos: Vec<RepositoryInfo> = self.db.scan_prefix(b"repo:")
            .filter_map(|item| item.ok())
            .filter_map(|(_, value)| serde_json::from_slice(&value).ok())
            .collect();
        
        for repo in repos {
            if let Ok(updated_repo) = self.fetch_repository_info(&repo.url).await {
                self.store_repository_info(&updated_repo)?;
                debug!("Updated repository: {}", repo.name);
            }
        }
        
        Ok(())
    }
    
    async fn fetch_repository_info(&self, url: &str) -> Result<RepositoryInfo> {
        // Extract owner and repo from GitHub URL
        let parts: Vec<&str> = url.split('/').collect();
        if parts.len() < 2 {
            return Err(HylaeanError::InvalidPath { path: url.to_string() });
        }
        
        let owner = parts[parts.len() - 2];
        let repo = parts[parts.len() - 1];
        
        let api_url = format!("https://api.github.com/repos/{}/{}", owner, repo);
        
        let mut request = self.client.get(&api_url);
        
        if let Some(token) = &self.github_token {
            request = request.header("Authorization", format!("token {}", token));
        }
        
        let response = request.send().await?;
        
        if !response.status().is_success() {
            return Err(HylaeanError::HttpError(reqwest::Error::from(response.error_for_status().unwrap_err())));
        }
        
        let repo_data: serde_json::Value = response.json().await?;
        
        let repo_info = RepositoryInfo {
            id: repo_data["id"].as_u64().unwrap_or(0).to_string(),
            name: repo_data["name"].as_str().unwrap_or("unknown").to_string(),
            url: url.to_string(),
            description: repo_data["description"].as_str().map(|s| s.to_string()),
            stars: repo_data["stargazers_count"].as_u64().unwrap_or(0) as u32,
            forks: repo_data["forks_count"].as_u64().unwrap_or(0) as u32,
            language: repo_data["language"].as_str().map(|s| s.to_string()),
            last_updated: Utc::now(),
            topics: repo_data["topics"].as_array()
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect())
                .unwrap_or_default(),
            installation_detected: false,
            installation_method: None,
            requirements: Vec::new(),
        };
        
        Ok(repo_info)
    }
    
    fn store_repository_info(&self, repo: &RepositoryInfo) -> Result<()> {
        let key = format!("repo:{}", repo.id);
        let value = serde_json::to_vec(repo)?;
        self.db.insert(key.as_bytes(), value)?;
        Ok(())
    }
    
    async fn search_arxiv_papers(&mut self) -> Result<()> {
        info!("Searching arXiv for new papers...");
        
        let search_terms = vec![
            "3d gaussian splatting",
            "gaussian splatting",
            "neural radiance fields",
            "nerf",
            "3d reconstruction",
        ];
        
        for term in search_terms {
            if let Ok(papers) = self.search_arxiv_by_term(term).await {
                for paper in papers {
                    self.store_paper_info(&paper)?;
                }
            }
        }
        
        Ok(())
    }
    
    async fn search_arxiv_by_term(&self, term: &str) -> Result<Vec<PaperInfo>> {
        let query = format!("search_query=all:{}&max_results=10", term.replace(' ', "+"));
        let url = format!("http://export.arxiv.org/api/query?{}", query);
        
        let response = self.client.get(&url).send().await?;
        let content = response.text().await?;
        
        // Parse XML response (simplified)
        let papers = self.parse_arxiv_response(&content)?;
        
        Ok(papers)
    }
    
    fn parse_arxiv_response(&self, xml: &str) -> Result<Vec<PaperInfo>> {
        // This is a simplified parser - in a real implementation, you'd use a proper XML parser
        let mut papers = Vec::new();
        
        // For now, return empty vector
        // TODO: Implement proper XML parsing for arXiv API response
        
        Ok(papers)
    }
    
    fn store_paper_info(&self, paper: &PaperInfo) -> Result<()> {
        let key = format!("paper:{}", paper.id);
        let value = serde_json::to_vec(paper)?;
        self.db.insert(key.as_bytes(), value)?;
        Ok(())
    }
    
    async fn generate_recommendations(&mut self) -> Result<()> {
        info!("Generating recommendations...");
        
        // Get all repositories
        let repos: Vec<RepositoryInfo> = self.db.scan_prefix(b"repo:")
            .filter_map(|item| item.ok())
            .filter_map(|(_, value)| serde_json::from_slice(&value).ok())
            .collect();
        
        // Generate recommendations based on repository popularity and relevance
        let mut recommendations = Vec::new();
        
        for repo in repos {
            if repo.stars > 50 && repo.language.as_deref() == Some("Python") {
                let recommendation = Recommendation {
                    id: format!("rec_{}", repo.id),
                    tool_name: repo.name.clone(),
                    reason: format!("Popular repository with {} stars, actively maintained", repo.stars),
                    confidence: self.calculate_confidence(&repo),
                    use_case: self.determine_use_case(&repo),
                    installation_difficulty: self.assess_difficulty(&repo),
                    prerequisites: self.determine_prerequisites(&repo),
                    estimated_setup_time: self.estimate_setup_time(&repo),
                };
                
                recommendations.push(recommendation);
            }
        }
        
        // Store recommendations
        for rec in recommendations {
            self.store_recommendation(&rec)?;
        }
        
        Ok(())
    }
    
    fn calculate_confidence(&self, repo: &RepositoryInfo) -> f32 {
        let star_score = (repo.stars as f32 / 1000.0).min(1.0);
        let fork_score = (repo.forks as f32 / 500.0).min(1.0);
        let topic_score = if repo.topics.iter().any(|t| t.contains("gaussian") || t.contains("splatting")) {
            1.0
        } else {
            0.5
        };
        
        (star_score + fork_score + topic_score) / 3.0
    }
    
    fn determine_use_case(&self, repo: &RepositoryInfo) -> String {
        if repo.name.to_lowercase().contains("render") {
            "Rendering and Visualization".to_string()
        } else if repo.name.to_lowercase().contains("train") {
            "Training and Optimization".to_string()
        } else if repo.name.to_lowercase().contains("dynamic") {
            "Dynamic Scene Reconstruction".to_string()
        } else {
            "General 3D Gaussian Splatting".to_string()
        }
    }
    
    fn assess_difficulty(&self, repo: &RepositoryInfo) -> RecommendationDifficulty {
        let has_requirements = repo.requirements.len() > 5;
        let complex_language = repo.language.as_deref() == Some("C++") || repo.language.as_deref() == Some("Rust");
        
        if has_requirements && complex_language {
            RecommendationDifficulty::Expert
        } else if has_requirements || complex_language {
            RecommendationDifficulty::Hard
        } else if repo.language.as_deref() == Some("Python") {
            RecommendationDifficulty::Medium
        } else {
            RecommendationDifficulty::Easy
        }
    }
    
    fn determine_prerequisites(&self, repo: &RepositoryInfo) -> Vec<String> {
        let mut prereqs = Vec::new();
        
        if let Some(lang) = &repo.language {
            prereqs.push(lang.clone());
        }
        
        if repo.topics.contains(&"pytorch".to_string()) {
            prereqs.push("PyTorch".to_string());
        }
        
        if repo.topics.contains(&"cuda".to_string()) {
            prereqs.push("CUDA".to_string());
        }
        
        prereqs
    }
    
    fn estimate_setup_time(&self, repo: &RepositoryInfo) -> String {
        match self.assess_difficulty(repo) {
            RecommendationDifficulty::Easy => "15-30 minutes".to_string(),
            RecommendationDifficulty::Medium => "1-2 hours".to_string(),
            RecommendationDifficulty::Hard => "3-5 hours".to_string(),
            RecommendationDifficulty::Expert => "1-2 days".to_string(),
        }
    }
    
    fn store_recommendation(&self, rec: &Recommendation) -> Result<()> {
        let key = format!("recommendation:{}", rec.id);
        let value = serde_json::to_vec(rec)?;
        self.db.insert(key.as_bytes(), value)?;
        Ok(())
    }
    
    pub async fn generate_install_script(&self, tool: String, output: Option<String>) -> Result<()> {
        info!("Generating installation script for: {}", tool);
        
        // Find the repository
        let repo_info = self.find_repository_by_name(&tool)?;
        
        let script = self.create_install_script(&repo_info)?;
        
        let output_path = output.unwrap_or_else(|| format!("install_{}.sh", tool));
        let mut file = File::create(&output_path)?;
        file.write_all(script.as_bytes())?;
        
        info!("Installation script generated: {}", output_path);
        Ok(())
    }
    
    fn find_repository_by_name(&self, name: &str) -> Result<RepositoryInfo> {
        for item in self.db.scan_prefix(b"repo:") {
            let (_, value) = item?;
            let repo: RepositoryInfo = serde_json::from_slice(&value)?;
            if repo.name.to_lowercase().contains(&name.to_lowercase()) {
                return Ok(repo);
            }
        }
        
        Err(HylaeanError::ToolNotFound { name: name.to_string() })
    }
    
    fn create_install_script(&self, repo: &RepositoryInfo) -> Result<String> {
        let mut script = String::new();
        
        script.push_str("#!/bin/bash\n");
        script.push_str(&format!("# Installation script for {}\n", repo.name));
        script.push_str(&format!("# Generated by Hylaean Splat\n\n"));
        
        script.push_str("set -e\n\n");
        
        script.push_str(&format!("echo \"Installing {}...\"\n", repo.name));
        script.push_str(&format!("git clone {} {}\n", repo.url, repo.name));
        script.push_str(&format!("cd {}\n", repo.name));
        
        // Add language-specific setup
        if let Some(lang) = &repo.language {
            match lang.as_str() {
                "Python" => {
                    script.push_str("echo \"Setting up Python environment...\"\n");
                    script.push_str("python -m venv venv\n");
                    script.push_str("source venv/bin/activate\n");
                    script.push_str("pip install -r requirements.txt\n");
                }
                "Rust" => {
                    script.push_str("echo \"Building Rust project...\"\n");
                    script.push_str("cargo build --release\n");
                }
                _ => {
                    script.push_str(&format!("echo \"Language {} detected - manual setup may be required\"\n", lang));
                }
            }
        }
        
        script.push_str(&format!("echo \"Installation of {} completed!\"\n", repo.name));
        
        Ok(script)
    }
    
    pub async fn show_recommendations(&self, use_case: Option<String>) -> Result<()> {
        info!("Showing recommendations...");
        
        let mut recommendations: Vec<Recommendation> = self.db.scan_prefix(b"recommendation:")
            .filter_map(|item| item.ok())
            .filter_map(|(_, value)| serde_json::from_slice(&value).ok())
            .collect();
        
        // Filter by use case if specified
        if let Some(ref case) = use_case {
            recommendations.retain(|r| r.use_case.to_lowercase().contains(&case.to_lowercase()));
        }
        
        // Sort by confidence
        recommendations.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        
        println!("Recommendations:");
        for rec in recommendations.iter().take(10) {
            println!("  {} (Confidence: {:.2})", rec.tool_name, rec.confidence);
            println!("    Use case: {}", rec.use_case);
            println!("    Reason: {}", rec.reason);
            println!("    Difficulty: {:?}", rec.installation_difficulty);
            println!("    Setup time: {}", rec.estimated_setup_time);
            if !rec.prerequisites.is_empty() {
                println!("    Prerequisites: {}", rec.prerequisites.join(", "));
            }
            println!();
        }
        
        Ok(())
    }
}