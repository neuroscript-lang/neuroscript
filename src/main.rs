//! NeuroScript CLI - Neural architecture composition language compiler

use clap::{Parser, Subcommand};
use miette::{IntoDiagnostic, NamedSource, WrapErr};
use neuroscript::{generate_pytorch, parse, stdlib, validate, NeuronBody};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "neuroscript")]
#[command(about = "Neural architecture composition language compiler", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new NeuroScript package
    Init {
        /// Package name
        #[arg(value_name = "NAME")]
        name: String,

        /// Create in this directory (defaults to NAME)
        #[arg(long, value_name = "PATH")]
        path: Option<PathBuf>,

        /// Package version
        #[arg(long, default_value = "0.1.0")]
        version: String,

        /// Author name and email
        #[arg(long)]
        author: Option<String>,

        /// License
        #[arg(long, default_value = "MIT")]
        license: String,

        /// Create a binary package (with examples)
        #[arg(long)]
        bin: bool,
    },

    /// Add a dependency to Axon.toml
    Add {
        /// Package name
        #[arg(value_name = "PACKAGE")]
        package: String,

        /// Version requirement (e.g., "1.0", "^1.2.3")
        #[arg(long)]
        version: Option<String>,

        /// Git repository URL
        #[arg(long)]
        git: Option<String>,

        /// Git branch
        #[arg(long)]
        branch: Option<String>,

        /// Git tag
        #[arg(long)]
        tag: Option<String>,

        /// Git revision (commit hash)
        #[arg(long)]
        rev: Option<String>,

        /// Local filesystem path
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// Fetch all dependencies
    Fetch {
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,

        /// Update dependencies to latest compatible versions
        #[arg(long)]
        update: bool,
    },

    /// Parse a NeuroScript file and show its structure
    Parse {
        /// Input NeuroScript file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Show detailed IR structure
        #[arg(short, long)]
        verbose: bool,
    },

    /// Validate a NeuroScript file
    Validate {
        /// Input NeuroScript file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Show IR structure and validation details
        #[arg(short, long)]
        verbose: bool,

        /// Skip loading standard library
        #[arg(long)]
        no_stdlib: bool,
    },

    /// Compile NeuroScript to PyTorch
    Compile {
        /// Input NeuroScript file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Neuron to compile (defaults to file name in PascalCase)
        #[arg(short = 'n', long, value_name = "NEURON")]
        neuron: Option<String>,

        /// Write output to file instead of stdout
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// Disable all optimizations
        #[arg(long)]
        no_optimize: bool,

        /// Disable dead branch elimination only
        #[arg(long)]
        no_dead_elim: bool,

        /// Show optimization details and timing
        #[arg(short, long)]
        verbose: bool,

        /// Skip loading standard library
        #[arg(long)]
        no_stdlib: bool,
    },

    /// List all neurons in a file
    List {
        /// Input NeuroScript file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Show additional details (connections, match expressions)
        #[arg(short, long)]
        verbose: bool,
    },
}

fn main() -> miette::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init {
            name,
            path,
            version,
            author,
            license,
            bin,
        } => cmd_init(name, path, version, author, license, bin),
        Commands::Add {
            package,
            version,
            git,
            branch,
            tag,
            rev,
            path,
        } => cmd_add(package, version, git, branch, tag, rev, path),
        Commands::Fetch { verbose, update } => cmd_fetch(verbose, update),
        Commands::Parse { file, verbose } => cmd_parse(file, verbose),
        Commands::Validate {
            file,
            verbose,
            no_stdlib,
        } => cmd_validate(file, verbose, no_stdlib),
        Commands::Compile {
            file,
            neuron,
            output,
            no_optimize,
            no_dead_elim,
            verbose,
            no_stdlib,
        } => cmd_compile(
            file,
            neuron,
            output,
            no_optimize,
            no_dead_elim,
            verbose,
            no_stdlib,
        ),
        Commands::List { file, verbose } => cmd_list(file, verbose),
    }
}

/// Init command: Create a new NeuroScript package
fn cmd_init(
    name: String,
    path: Option<PathBuf>,
    version: String,
    author: Option<String>,
    license: String,
    bin: bool,
) -> miette::Result<()> {
    use neuroscript::package::{init_package, InitOptions};

    let options = InitOptions {
        name: name.clone(),
        path,
        version,
        author,
        license: Some(license),
        bin,
    };

    match init_package(&options) {
        Ok(package_dir) => {
            println!("✓ Created new package '{}' at {}", name, package_dir.display());
            println!("\nNext steps:");
            println!("  cd {}", package_dir.display());
            println!("  # Edit src/*.ns with your neuron definitions");
            println!("  neuroscript build");
            Ok(())
        }
        Err(e) => {
            eprintln!("✗ Failed to initialize package: {}", e);
            std::process::exit(1);
        }
    }
}

/// Add command: Add a dependency to Axon.toml
fn cmd_add(
    package: String,
    version: Option<String>,
    git: Option<String>,
    branch: Option<String>,
    tag: Option<String>,
    rev: Option<String>,
    path: Option<PathBuf>,
) -> miette::Result<()> {
    use neuroscript::package::{Dependency, DependencyDetail, Manifest};

    // Find Axon.toml in current directory or parents
    let manifest_path = Manifest::find_in_directory(".")
        .map_err(|e| miette::miette!("No Axon.toml found in current directory or parents: {}", e))?;

    // Load manifest
    let mut manifest = Manifest::from_path(&manifest_path)
        .into_diagnostic()
        .wrap_err("Failed to load Axon.toml")?;

    // Create dependency specification
    let dep = if let Some(git_url) = git {
        Dependency::Detailed(DependencyDetail {
            version: version.clone(),
            git: Some(git_url),
            branch,
            tag,
            rev,
            path: None,
            optional: false,
        })
    } else if let Some(local_path) = path {
        Dependency::Detailed(DependencyDetail {
            version: None,
            git: None,
            branch: None,
            tag: None,
            rev: None,
            path: Some(local_path),
            optional: false,
        })
    } else if let Some(ver) = version {
        Dependency::Simple(ver)
    } else {
        // Default to latest version
        Dependency::Simple("*".to_string())
    };

    // Add to manifest
    manifest.dependencies.insert(package.clone(), dep);

    // Save manifest
    let toml_string = toml::to_string_pretty(&manifest)
        .into_diagnostic()
        .wrap_err("Failed to serialize manifest")?;

    fs::write(&manifest_path, toml_string)
        .into_diagnostic()
        .wrap_err("Failed to write Axon.toml")?;

    println!("✓ Added dependency: {}", package);
    println!("  Updated: {}", manifest_path.display());
    println!("\nRun `neuroscript fetch` to download dependencies");

    Ok(())
}

/// Fetch command: Fetch all dependencies
fn cmd_fetch(verbose: bool, update: bool) -> miette::Result<()> {
    use neuroscript::package::{Lockfile, Manifest, Registry};

    // Find Axon.toml
    let manifest_path = Manifest::find_in_directory(".")
        .map_err(|e| miette::miette!("No Axon.toml found in current directory or parents: {}", e))?;

    let manifest_dir = manifest_path
        .parent()
        .ok_or_else(|| miette::miette!("Invalid manifest path"))?;

    // Load manifest
    let manifest = Manifest::from_path(&manifest_path)
        .into_diagnostic()
        .wrap_err("Failed to load Axon.toml")?;

    if verbose {
        println!("Loading manifest from {}", manifest_path.display());
        println!("Package: {} v{}", manifest.package.name, manifest.package.version);
    }

    // Check if we have dependencies
    if manifest.dependencies.is_empty() {
        println!("No dependencies to fetch");
        return Ok(());
    }

    // Initialize registry
    let registry = Registry::new()
        .into_diagnostic()
        .wrap_err("Failed to initialize registry")?;

    registry
        .init()
        .into_diagnostic()
        .wrap_err("Failed to initialize cache")?;

    if verbose {
        let cache_dir = Registry::default_cache_dir()
            .into_diagnostic()
            .wrap_err("Failed to get cache directory")?;
        println!("Cache directory: {}", cache_dir.display());
    }

    // Check lockfile
    let lockfile_path = manifest_dir.join("Axon.lock");
    let needs_update = if lockfile_path.exists() && !update {
        let lockfile = Lockfile::from_path(&lockfile_path)
            .into_diagnostic()
            .wrap_err("Failed to load Axon.lock")?;

        if lockfile.is_up_to_date(&manifest) {
            if verbose {
                println!("Lockfile is up-to-date, using existing resolutions");
            }
            false
        } else {
            if verbose {
                println!("Lockfile is outdated, resolving dependencies");
            }
            true
        }
    } else {
        true
    };

    // Fetch dependencies
    println!("Fetching {} dependencies...", manifest.dependencies.len());

    let fetched = registry
        .fetch_dependencies(&manifest)
        .into_diagnostic()
        .wrap_err("Failed to fetch dependencies")?;

    for (name, path) in &fetched {
        println!("  ✓ {} -> {}", name, path.display());
    }

    // Generate/update lockfile if needed
    if needs_update {
        // For now, create a simple lockfile with fetched dependencies
        // Full resolution will be implemented when we have a registry
        let mut lockfile = Lockfile::new();

        for (name, path) in &fetched {
            // Load the dependency's manifest to get version
            let dep_manifest_path = path.join("Axon.toml");
            if let Ok(dep_manifest) = Manifest::from_path(dep_manifest_path) {
                let locked = neuroscript::package::LockedPackage::from_path(
                    name.clone(),
                    dep_manifest.package.version.clone(),
                    path.clone(),
                );
                lockfile.add_package(locked);
            }
        }

        lockfile
            .save(&lockfile_path)
            .into_diagnostic()
            .wrap_err("Failed to save Axon.lock")?;

        if verbose {
            println!("\n✓ Generated Axon.lock");
        }
    }

    println!("\n✓ All dependencies fetched successfully");

    Ok(())
}

/// Parse command: Read and display the IR structure
fn cmd_parse(file: PathBuf, verbose: bool) -> miette::Result<()> {
    let source = read_source(&file)?;
    let program = parse(&source).map_err(|e| {
        let source_named = NamedSource::new(file.to_string_lossy(), source);
        miette::Report::from(e).with_source_code(source_named)
    })?;

    if verbose {
        println!(
            "Parsed {} imports and {} neurons:\n",
            program.uses.len(),
            program.neurons.len()
        );

        for use_stmt in &program.uses {
            println!("  use {},{}", use_stmt.source, use_stmt.path.join("/"));
        }

        if !program.uses.is_empty() {
            println!();
        }
    } else {
        println!("✓ Successfully parsed {}", file.display());
    }

    print_neuron_summary(&program, verbose);

    Ok(())
}

/// Validate command: Parse and validate the program
fn cmd_validate(file: PathBuf, verbose: bool, no_stdlib: bool) -> miette::Result<()> {
    let source = read_source(&file)?;
    let user_program = parse(&source).map_err(|e| {
        let source_named = NamedSource::new(file.to_string_lossy(), source);
        miette::Report::from(e).with_source_code(source_named)
    })?;

    // Load and merge stdlib if not disabled
    let mut program = if no_stdlib {
        if verbose {
            println!("Skipping stdlib loading (--no-stdlib)");
        }
        user_program
    } else {
        if verbose {
            println!("Loading standard library...");
        }
        match stdlib::load_stdlib() {
            Ok(stdlib_program) => {
                if verbose {
                    println!("✓ Loaded {} stdlib neurons", stdlib_program.neurons.len());
                }
                stdlib::merge_programs(stdlib_program, user_program)
            }
            Err(e) => {
                eprintln!("Warning: Failed to load stdlib: {}", e);
                eprintln!("Continuing without stdlib...");
                user_program
            }
        }
    };

    if verbose {
        println!(
            "Parsed {} imports and {} neurons total\n",
            program.uses.len(),
            program.neurons.len()
        );
    }

    if verbose {
        println!("Running validation...");
    }

    match validate(&mut program) {
        Ok(()) => {
            if verbose {
                println!("✓ Program is valid!");
            } else {
                println!("✓ Valid");
            }
            Ok(())
        }
        Err(errors) => {
            println!("✗ Validation failed with {} error(s):", errors.len());
            for error in errors {
                println!("  {}", error);
            }
            std::process::exit(1);
        }
    }
}

/// Compile command: Full pipeline - parse, validate, optimize, codegen
fn cmd_compile(
    file: PathBuf,
    neuron: Option<String>,
    output: Option<PathBuf>,
    no_optimize: bool,
    no_dead_elim: bool,
    verbose: bool,
    no_stdlib: bool,
) -> miette::Result<()> {
    let source = read_source(&file)?;
    let user_program = parse(&source).map_err(|e| {
        let source_named = NamedSource::new(file.to_string_lossy(), source);
        miette::Report::from(e).with_source_code(source_named)
    })?;

    // Load and merge stdlib if not disabled
    let mut program = if no_stdlib {
        if verbose {
            println!("Skipping stdlib loading (--no-stdlib)");
        }
        user_program
    } else {
        if verbose {
            println!("Loading standard library...");
        }
        match stdlib::load_stdlib() {
            Ok(stdlib_program) => {
                if verbose {
                    println!("✓ Loaded {} stdlib neurons", stdlib_program.neurons.len());
                }
                stdlib::merge_programs(stdlib_program, user_program)
            }
            Err(e) => {
                eprintln!("Warning: Failed to load stdlib: {}", e);
                eprintln!("Continuing without stdlib...");
                user_program
            }
        }
    };

    if verbose {
        println!(
            "Parsed {} imports and {} neurons total",
            program.uses.len(),
            program.neurons.len()
        );
    }

    // Validate
    if let Err(errors) = validate(&mut program) {
        println!("✗ Validation failed with {} error(s):", errors.len());
        for error in errors {
            println!("  {}", error);
        }
        std::process::exit(1);
    }
    if verbose {
        println!("✓ Validation passed");
    }

    // Infer neuron name if not provided
    let neuron_name = if let Some(n) = neuron {
        n
    } else {
        infer_neuron_name(&file, &program)?
    };

    // Check neuron exists
    if !program.neurons.contains_key(&neuron_name) {
        let available: Vec<&str> = program.neurons.keys().map(|s| s.as_str()).collect();
        eprintln!("✗ Neuron '{}' not found", neuron_name);
        eprintln!("  Available neurons: {}", available.join(", "));
        std::process::exit(1);
    }

    // Optimize
    if !no_optimize {
        let reordered = neuroscript::optimizer::reorder_match_arms(&mut program);
        let pruned = neuroscript::optimizer::optimize_matches(&mut program, !no_dead_elim);
        if verbose {
            if reordered > 0 {
                println!(
                    "  Pattern reordering: optimized {} match expressions",
                    reordered
                );
            }
            if pruned > 0 {
                println!("  Dead branch elimination: pruned {} arms", pruned);
            }
        }
    } else if verbose {
        println!("  Optimizations disabled");
    }

    // Codegen
    match generate_pytorch(&program, &neuron_name) {
        Ok(python_code) => {
            if let Some(output_path) = output {
                fs::write(&output_path, python_code)
                    .into_diagnostic()
                    .wrap_err_with(|| format!("Failed to write to {}", output_path.display()))?;
                if verbose {
                    println!(
                        "✓ Generated PyTorch code for '{}' → {}",
                        neuron_name,
                        output_path.display()
                    );
                } else {
                    println!("✓ Compiled to {}", output_path.display());
                }
            } else {
                println!("# Generated PyTorch code for '{}'", neuron_name);
                println!("{}", python_code);
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("✗ Codegen failed: {}", e);
            std::process::exit(1);
        }
    }
}

/// List command: Show all neurons and their signatures
fn cmd_list(file: PathBuf, verbose: bool) -> miette::Result<()> {
    let source = read_source(&file)?;
    let program = parse(&source).map_err(|e| {
        let source_named = NamedSource::new(file.to_string_lossy(), source);
        miette::Report::from(e).with_source_code(source_named)
    })?;

    if program.neurons.is_empty() {
        println!("No neurons found in {}", file.display());
        return Ok(());
    }

    println!(
        "Neurons in {} ({} total):\n",
        file.display(),
        program.neurons.len()
    );

    for (name, neuron) in &program.neurons {
        let kind = match &neuron.body {
            NeuronBody::Primitive(_) => "primitive",
            NeuronBody::Graph { .. } => "composite",
        };

        let inputs: Vec<String> = neuron
            .inputs
            .iter()
            .map(|p| format!("{}: {}", p.name, p.shape))
            .collect();

        let outputs: Vec<String> = neuron
            .outputs
            .iter()
            .map(|p| format!("{}: {}", p.name, p.shape))
            .collect();

        println!("  {} ({})", name, kind);
        println!("    inputs:  {}", inputs.join(", "));
        println!("    outputs: {}", outputs.join(", "));

        if verbose {
            if let NeuronBody::Graph { connections, .. } = &neuron.body {
                println!("    connections: {} ", connections.len());
                for conn in connections.iter().take(3) {
                    println!("      - {:?}", conn);
                }
                if connections.len() > 3 {
                    println!("      ... and {} more", connections.len() - 3);
                }
            }
        }

        println!();
    }

    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Read source file with error handling
fn read_source(file: &PathBuf) -> miette::Result<String> {
    fs::read_to_string(file)
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to read {}", file.display()))
}

/// Print a summary of all neurons
fn print_neuron_summary(program: &neuroscript::Program, verbose: bool) {
    for (name, neuron) in &program.neurons {
        let kind = match &neuron.body {
            NeuronBody::Primitive(_) => "primitive",
            NeuronBody::Graph { .. } => "composite",
        };

        let inputs: Vec<String> = neuron
            .inputs
            .iter()
            .map(|p| format!("{}: {}", p.name, p.shape))
            .collect();

        let outputs: Vec<String> = neuron
            .outputs
            .iter()
            .map(|p| format!("{}: {}", p.name, p.shape))
            .collect();

        println!("  {} ({})", name, kind);
        println!("    in:  {}", inputs.join(", "));
        println!("    out: {}", outputs.join(", "));

        if verbose {
            if let NeuronBody::Graph {
                connections: conns, ..
            } = &neuron.body
            {
                println!("    connections: {:?}", conns);
            }
        }

        println!();
    }
}

/// Infer neuron name from filename, or fail with helpful message
fn infer_neuron_name(file: &Path, program: &neuroscript::Program) -> miette::Result<String> {
    // Extract filename without extension
    let filename = file
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| miette::miette!("Invalid file path: {}", file.display()))?;

    // Convert snake_case or kebab-case to PascalCase
    let neuron_name = filename
        .split(['_', '-'])
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<String>();

    // Check if this neuron exists
    if program.neurons.contains_key(&neuron_name) {
        Ok(neuron_name)
    } else {
        // Provide helpful message with available neurons
        let available: Vec<&str> = program.neurons.keys().map(|s| s.as_str()).collect();
        eprintln!(
            "✗ No neuron matching filename '{}' found (tried: '{}')",
            file.display(),
            neuron_name
        );
        if !available.is_empty() {
            eprintln!("  Available neurons: {}", available.join(", "));
        }
        eprintln!("  Use --neuron <NAME> to specify explicitly");
        std::process::exit(1);
    }
}
