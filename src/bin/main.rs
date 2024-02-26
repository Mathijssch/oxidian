extern crate oxidian;
extern crate pretty_env_logger;
#[macro_use] extern crate log;

use clap::{Parser, Subcommand};
use oxidian::exporting::{config, exporter};
use oxidian::utils::constants::INDEX_FILE;

use oxidian::core::errors;

use std::path::{Path, PathBuf};

type MissingDirectory<'a> = errors::MissingDirectoryError<&'a Path>;
type MissingIndex<'a> = errors::MissingIndexError<&'a Path>;
type ExistingOutput<'a> = errors::DirExistsError<&'a Path>;
type InitializeError<'a> = errors::InitializationError<&'a Path>;

// -------------------------------------------
// CLI
// -------------------------------------------
//

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "oxidian")]
#[command(about = "Tools for Obsidian-style markdown notes.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Builds the webpage
    #[command(arg_required_else_help = true)]
    Build {
        /// The directory containing the notes
        dir: PathBuf,

        /// The output directory
        #[arg(short, long)]
        out: Option<PathBuf>,

        /// Path to the index file
        #[arg(short, long)]
        index: Option<PathBuf>,
        
        /// Path to the config file. Uses `[dir]/config.toml` by default.
        #[arg(short, long)]
        cfg: Option<PathBuf>,
    },
    #[command(arg_required_else_help=true)]
    Watch {
        /// The directory containing the notes
        dir: PathBuf,

        /// The output directory
        #[arg(short, long)]
        out: Option<PathBuf>,
            
        /// Path to the index file
        #[arg(short, long)]
        index: Option<PathBuf>,
        
        /// Path to the config file. Uses `[dir]/config.toml` by default.
        #[arg(short, long)]
        cfg: Option<PathBuf>
    },

    /// Launches a server
    #[command()]
    Serve {
        #[arg(short, long)]
        port: Option<u32>,
    },
}

fn serve(port: u32) {
    println!("Serving on port {}", port);
}

fn main() {
    pretty_env_logger::init();

    let args = Cli::parse();

    match args.command {
        Commands::Build {
            dir,
            out,
            index, 
            cfg
        } => {
            trace!("Running build command.");
            let index = index.unwrap_or(PathBuf::from(INDEX_FILE));
            debug!("index file: {:?}", index);
            let out = out.unwrap_or_else(|| {
                let mut out = dir.clone();
                if let Some(main_dir) = out.file_name() { 
                    let mut filename = main_dir.to_owned(); 
                    filename.push(std::ffi::OsString::from("_out"));
                    out.set_file_name(filename);
                } else {
                    out.set_file_name("notebook_out");
                }
                out
            });
            debug!("output directory: {:?}", out);
            build_vault(dir, out, index, cfg);
        }

        Commands::Watch {
            dir,
            out,
            index, 
            cfg
        } => {
            trace!("Running watch command.");
            let index = index.unwrap_or(PathBuf::from(INDEX_FILE));
            debug!("index file: {:?}", index);
            let out = out.unwrap_or_else(|| {
                let mut out = dir.clone();
                if let Some(main_dir) = out.file_name() { 
                    let mut filename = main_dir.to_owned(); 
                    filename.push(std::ffi::OsString::from("_out"));
                    out.set_file_name(filename);
                } else {
                    out.set_file_name("notebook_out");
                }
                out
            });
            debug!("output directory: {:?}", out);
            watch(dir, out, index, cfg);
        }
        Commands::Serve { port } => {
            let port_nb = port.unwrap_or(8080);
            serve(port_nb);
        }
    }
}

fn build_vault(
    input_dir: PathBuf,
    output_dir: PathBuf,
    index_file: PathBuf,
    config_file: Option<PathBuf>
) {
    // Prepare
    // --------------------
    if let Err(e) = validate_build_args(&input_dir, &output_dir, &index_file) {
        log::warn!("{}", e);
    };

    let default_config_path = input_dir.join("config.toml");
    let config_file = config_file
        .unwrap_or(default_config_path); 

    let export_config = config::ExportConfig::from_file(config_file)
        .unwrap_or_default();

    let mut builder = exporter::Exporter::new(&input_dir, &output_dir, &export_config);

    // Do the export
    builder.export();

    // Print outputs
    // ----------------------
    info!("{}", builder.stats);
}

fn watch(
    input_dir: PathBuf,
    output_dir: PathBuf,
    index_file: PathBuf,
    config_file: Option<PathBuf>
) {
    use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
    let (tx, rx) = std::sync::mpsc::channel();

    // Prepare
    // --------------------
    if let Err(e) = validate_build_args(&input_dir, &output_dir, &index_file) {
        log::warn!("{}", e);
    };

    let default_config_path = input_dir.join("config.toml");
    let config_file = config_file
        .unwrap_or(default_config_path); 

    let export_config = config::ExportConfig::from_file(config_file)
        .unwrap_or_default();

    let mut builder = exporter::Exporter::new(input_dir.as_ref(), 
        output_dir.as_ref(), &export_config);

    //todo store cache files to allow a true incremental build.
    info!("Running initial build.");
    let mut backlinks = builder.export();
    let line = "-".repeat(70);
    info!("Initial build finished.\n\n{}{}{}\n", line, builder.stats, line);
    info!("Watching for file changes.");

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(&input_dir, RecursiveMode::Recursive).unwrap();

    for res in rx {
        match res {
            Ok(event) => builder.handle_event(event, &mut backlinks),
            Err(error) => log::error!("Error: {error:?}"),
        }
    }
}

fn validate_build_args<'a>(
    input_dir: &'a Path,
    output_dir: &'a Path,
    index_file: &'a Path,
) -> Result<(), InitializeError<'a>> {
    check_exists(input_dir)
        .map_err(|e| errors::InitializationError::<&Path>::MissingDirectory(e))?;
    check_output_available(output_dir)
        .map_err(|e| errors::InitializationError::<&Path>::OutputDirExists(e))?;
    check_contains_index(&input_dir, &index_file)?;
    Ok(())
}

fn check_exists(input_path: &Path) -> Result<(), MissingDirectory> {
    if !input_path.exists() {
        return Err(errors::MissingDirectoryError(input_path));
    }
    Ok(())
}

fn check_contains_index<'a>(
    input_path: &'a Path,
    index_file: &'a Path,
) -> Result<(), MissingIndex<'a>> {
    if !input_path.join(index_file).exists() {
        return Err(errors::MissingIndexError(input_path, index_file));
    }
    Ok(())
}

fn check_output_available(output_path: &Path) -> Result<(), ExistingOutput> {
    if output_path.exists() {
        return Err(errors::DirExistsError(output_path));
    }
    Ok(())
}
