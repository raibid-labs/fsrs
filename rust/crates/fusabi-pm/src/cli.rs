//! Command-line interface for the Fusabi Package Manager (fpm).

use clap::{Parser, Subcommand};
use fusabi_pm::{
    install_dependencies, print_publish_instructions, publish_package, Dependency, Manifest,
    Package, PackageBuilder,
};
use std::fs;

#[derive(Parser)]
#[command(name = "fpm")]
#[command(about = "Fusabi Package Manager", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Fusabi package
    Init {
        /// Package name (defaults to current directory name)
        #[arg(long)]
        name: Option<String>,
    },
    /// Build the current package
    Build,
    /// Run the current package
    Run,
    /// Add a dependency to the current package
    Add {
        /// Package name to add
        package: String,

        /// Version requirement
        #[arg(long)]
        version: Option<String>,
    },
    /// Install dependencies from fusabi.toml
    Install,
    /// Publish the package to the registry
    Publish,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name } => {
            if let Err(e) = init_package(name) {
                eprintln!("Error initializing package: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Build => {
            if let Err(e) = build_package() {
                eprintln!("Error building package: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Run => {
            if let Err(e) = run_package() {
                eprintln!("Error running package: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Add { package, version } => {
            if let Err(e) = add_package(package, version) {
                eprintln!("Error adding package: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Install => {
            if let Err(e) = run_install() {
                eprintln!("Error installing dependencies: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Publish => {
            if let Err(e) = run_publish() {
                eprintln!("Error publishing package: {}", e);
                std::process::exit(1);
            }
        }
    }
}

/// Installs dependencies from fusabi.toml.
fn run_install() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    install_dependencies(&current_dir)?;
    Ok(())
}

/// Publishes the package to the registry.
fn run_publish() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    let result = publish_package(&current_dir)?;
    print_publish_instructions(&result);
    Ok(())
}

/// Initializes a new Fusabi package in the current directory.
fn init_package(name: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    let manifest_path = current_dir.join("fusabi.toml");
    let src_dir = current_dir.join("src");
    let main_file = src_dir.join("main.fsx");

    // Check if fusabi.toml already exists
    if manifest_path.exists() {
        return Err("fusabi.toml already exists in this directory".into());
    }

    // Determine package name
    let package_name = name.unwrap_or_else(|| {
        current_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("fusabi-package")
            .to_string()
    });

    // Create package metadata
    let package = Package {
        name: package_name.clone(),
        version: "0.1.0".to_string(),
        authors: vec![],
        description: None,
        license: Some("MIT".to_string()),
        repository: None,
    };

    // Create manifest
    let manifest = Manifest::new(package);
    let toml_content = manifest.to_toml()?;

    // Write fusabi.toml
    fs::write(&manifest_path, toml_content)?;
    println!("Created fusabi.toml");

    // Create src directory if it doesn't exist
    if !src_dir.exists() {
        fs::create_dir(&src_dir)?;
        println!("Created src/ directory");
    }

    // Create main.fsx if it doesn't exist
    if !main_file.exists() {
        let main_content = r#"// Main entry point for the Fusabi package

fn main() {
    println("Hello, Fusabi!")
}
"#;
        fs::write(&main_file, main_content)?;
        println!("Created src/main.fsx");
    }

    println!("\nPackage '{}' initialized successfully!", package_name);
    println!("\nNext steps:");
    println!("  - Edit fusabi.toml to configure your package");
    println!("  - Add your code to src/main.fsx");
    println!("  - Run 'fpm build' to build your package (coming soon)");

    Ok(())
}

/// Adds a dependency to the current package.
fn add_package(
    package: String,
    version: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    let manifest_path = current_dir.join("fusabi.toml");

    // Check if fusabi.toml exists
    if !manifest_path.exists() {
        return Err("fusabi.toml not found. Run 'fpm init' first.".into());
    }

    // Load existing manifest
    let mut manifest = Manifest::load(&manifest_path)?;

    // Create dependency
    let dependency = match version {
        Some(v) => Dependency::Simple(v),
        None => Dependency::Simple("*".to_string()),
    };

    // Add dependency
    manifest.add_dependency(package.clone(), dependency);

    // Write updated manifest
    let toml_content = manifest.to_toml()?;
    fs::write(&manifest_path, toml_content)?;

    println!("Added dependency: {}", package);
    println!("Updated fusabi.toml");

    Ok(())
}

/// Builds the current Fusabi package.
fn build_package() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;

    let builder = PackageBuilder::new(current_dir).verbose(true);
    let result = builder.build()?;

    println!("Output: {} ({} bytes)", result.output_path.display(), result.output_size);

    Ok(())
}

/// Runs the current Fusabi package.
fn run_package() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    let manifest_path = current_dir.join("fusabi.toml");

    // Check if fusabi.toml exists
    if !manifest_path.exists() {
        return Err("fusabi.toml not found. Run 'fpm init' first.".into());
    }

    // Load manifest
    let manifest = Manifest::load(&manifest_path)?;
    println!("Running {}...", manifest.package.name);

    // Find main entry point
    let main_path = current_dir.join("src").join("main.fsx");
    if !main_path.exists() {
        return Err("src/main.fsx not found".into());
    }

    // Read and execute source code
    let source = fs::read_to_string(&main_path)?;

    match fusabi::run_source(&source) {
        Ok(result) => {
            // Print result if not Unit
            if !matches!(result, fusabi_vm::Value::Unit) {
                println!("{}", result);
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("Runtime error: {}", e);
            Err(e.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_init_package_creates_files() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Initialize package
        let result = init_package(Some("test-package".to_string()));
        assert!(result.is_ok());

        // Check that files were created
        assert!(temp_dir.path().join("fusabi.toml").exists());
        assert!(temp_dir.path().join("src").exists());
        assert!(temp_dir.path().join("src/main.fsx").exists());

        // Verify manifest content
        let manifest = Manifest::load(temp_dir.path().join("fusabi.toml")).unwrap();
        assert_eq!(manifest.package.name, "test-package");
        assert_eq!(manifest.package.version, "0.1.0");

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_init_package_fails_if_manifest_exists() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Create existing fusabi.toml
        fs::write(temp_dir.path().join("fusabi.toml"), "").unwrap();

        // Try to initialize package
        let result = init_package(None);
        assert!(result.is_err());

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }
}
