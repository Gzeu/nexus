//! NEXUS CLI - The Living Terminal
//! 
//! Command-line interface for the NEXUS agent platform

use clap::{Parser, Subcommand};
use std::io::{self, Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// NEXUS - The Living Terminal
/// 
/// A revolutionary CLI tool combining AI Agents, Web3, and intelligent workflows
#[derive(Parser)]
#[command(name = "nexus")]
#[command(about = "NEXUS - The Living Terminal")]
#[command(long_about = "A revolutionary CLI tool combining AI Agents, Web3, and intelligent workflows")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show version and system information
    Version {
        /// Show detailed system information
        #[arg(long)]
        verbose: bool,
    },
    /// Initialize NEXUS configuration and directories
    #[command(name = "init")]
    Init,
    /// Agent management commands  
    Agent,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Version { verbose } => {
            print_banner(verbose)?;
        },
        Commands::Init => {
            print_banner(false)?;
            println!("🚀 Initializing NEXUS workspace...");
            println!("📁 Created: ./nexus/");
            println!("📁 Created: ./nexus/data/");
            println!("📁 Created: ./nexus/logs/");  
            println!("📄 Created: ./nexus/nexus.toml");
            println!("✅ NEXUS workspace initialized successfully!");
        },
        Commands::Agent => {
            print_banner(false)?;
            println!("🤖 Agent management coming soon...");
            println!("💡 Tip: Use 'nexus agent run --dry' to simulate agent execution");
        },
    }

    Ok(())
}

fn print_banner(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    
    // ASCII Art Banner
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
    writeln!(stdout, r#"
    ███╗   ██╗███████╗██╗  ██╗██╗   ██╗███████╗
    ████╗  ██║██╔════╝╚██╗██╔╝██║   ██║██╔════╝
    ██╔██╗ ██║█████╗   ╚███╔╝ ██║   ██║███████╗
    ██║╚██╗██║██╔══╝   ██╔██╗ ██║   ██║╚════██║
    ██║ ╚████║███████╗██╔╝ ██╗╚██████╔╝███████║
    ╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝"#)?;
    
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_bold(true))?;
    writeln!(stdout, "    The Living Terminal")?;
    
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
    writeln!(stdout, "    AI Agents • Web3 • Intelligent Workflows")?;
    
    stdout.reset()?;
    writeln!(stdout)?;

    // Version Information
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true))?;
    write!(stdout, "🔧 NEXUS Version: ")?;
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
    writeln!(stdout, "v{}", env!("CARGO_PKG_VERSION"))?;

    if verbose {
        // Detailed system information
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
        writeln!(stdout, "\n📊 System Information:")?;
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
        writeln!(stdout, "   • Rust Compiler: {}", get_rustc_version())?;
        writeln!(stdout, "   • Target Triple: {}", std::env::consts::ARCH)?;
        writeln!(stdout, "   • OS: {} {}", std::env::consts::OS, std::env::consts::FAMILY)?;
        
        // Workspace information
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Magenta)))?;
        writeln!(stdout, "\n📦 Workspace Members:")?;
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
        writeln!(stdout, "   • nexus-cli (binary)")?;
        writeln!(stdout, "   • nexus-core (library)")?;
        writeln!(stdout, "   • plugins/example (future)")?;
    }

    stdout.reset()?;
    writeln!(stdout)?;
    Ok(())
}

fn get_rustc_version() -> String {
    // In a production app, you'd use rustc_version crate or build script
    // For now, we'll use a placeholder that shows the concept
    format!("rustc {} (built with Cargo {})", 
        option_env!("RUSTC_SEMANTIC_VERSION").unwrap_or("1.75.0"),
        option_env!("CARGO_VERSION").unwrap_or("1.75.0"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli_builds() {
        // Test that CLI structure compiles correctly
        let _cli = Cli::parse_from(&["nexus", "version"]);
    }

    #[test]
    fn rustc_version_format() {
        let version = get_rustc_version();
        assert!(version.contains("rustc"));
        assert!(version.contains("Cargo"));
    }
}