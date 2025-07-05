//! Agentic components for intelligent 3DGS ecosystem management

// pub mod arxiv_monitor;
// pub mod github_monitor;
// pub mod recommendation_engine;

use crate::errors::Result;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub id: String,
    pub task_type: AgentTaskType,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentTaskType {
    RepositoryMonitor,
    PaperScraping,
    InstallationGeneration,
    RecommendationUpdate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

pub trait AgentComponent {
    fn name(&self) -> &str;
    fn is_enabled(&self) -> bool;
    fn run(&mut self) -> Result<()>;
    fn get_status(&self) -> TaskStatus;
}