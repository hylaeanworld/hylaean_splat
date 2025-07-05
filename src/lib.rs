//! Hylaean Splat - Operating system for 3D Gaussian splatting tools
//! 
//! This library provides a unified interface for managing, converting, and orchestrating
//! various 3D Gaussian splatting tools and workflows.

pub mod cli;
pub mod core;
pub mod formats;
pub mod integrations;
pub mod agentic;
pub mod errors;
pub mod config;

pub use crate::core::HylaeanSplat;