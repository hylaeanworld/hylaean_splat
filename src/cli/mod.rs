use clap::Subcommand;

pub mod commands;

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize Hylaean Splat configuration
    Init {
        /// Force initialization even if config exists
        #[arg(short, long)]
        force: bool,
    },
    
    /// Manage 3DGS tools
    Tool {
        #[command(subcommand)]
        action: ToolAction,
    },
    
    /// Convert between different data formats
    Convert {
        /// Input file path
        #[arg(short, long)]
        input: String,
        
        /// Output file path
        #[arg(short, long)]
        output: String,
        
        /// Input format (auto-detect if not specified)
        #[arg(long)]
        input_format: Option<String>,
        
        /// Output format
        #[arg(long)]
        output_format: String,
    },
    
    /// Run the agentic component
    Agent {
        #[command(subcommand)]
        action: AgentAction,
    },
    
    /// List available tools and their status
    List {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },
    
    /// Execute a workflow
    Workflow {
        /// Workflow name or path to workflow file
        name: String,
        
        /// Input data path
        #[arg(short, long)]
        input: Option<String>,
        
        /// Output directory
        #[arg(short, long)]
        output: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ToolAction {
    /// Discover and register installed tools
    Discover {
        /// Specific path to search
        #[arg(short, long)]
        path: Option<String>,
    },
    
    /// Install a tool
    Install {
        /// Tool name or repository URL
        name: String,
        
        /// Installation path
        #[arg(short, long)]
        path: Option<String>,
    },
    
    /// Remove a tool
    Remove {
        /// Tool name
        name: String,
    },
    
    /// Update a tool
    Update {
        /// Tool name (update all if not specified)
        name: Option<String>,
    },
    
    /// Show tool information
    Info {
        /// Tool name
        name: String,
    },
    
    /// Execute a tool with arguments
    Run {
        /// Tool name
        name: String,
        
        /// Arguments to pass to the tool
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum AgentAction {
    /// Start monitoring repositories and papers
    Start {
        /// Run in daemon mode
        #[arg(short, long)]
        daemon: bool,
    },
    
    /// Stop the agent
    Stop,
    
    /// Get agent status
    Status,
    
    /// Update repository database
    Update,
    
    /// Generate installation script for a tool
    Generate {
        /// Tool name
        tool: String,
        
        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },
    
    /// Get recommendations based on current setup
    Recommend {
        /// Specific use case
        #[arg(short, long)]
        use_case: Option<String>,
    },
}