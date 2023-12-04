extern crate oxidian;
use clap::{Parser, Subcommand};
//use oxidian::oxidianlib::filesys::{get_all_notes, convert_path};
//use oxidian::oxidianlib::note;
use oxidian::oxidianlib::exporter;
use oxidian::oxidianlib::{
    constants::INDEX_FILE,
    errors, //::{self, IndexError},
};

use std::path::Path;

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
        #[arg(short, long)]
        dir: String,

        /// The output directory
        #[arg(short, long)]
        out: String,

        /// Path to the index file
        #[arg(short, long)]
        index_path: Option<String>,

        /// Path where the attachments are stored in the input directory
        #[arg(short, long)]
        attachment_dir: Option<String>,
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
    let args = Cli::parse();

    match args.command {
        Commands::Build {
            dir,
            out,
            index_path,
            attachment_dir,
        } => {
            let index = index_path.unwrap_or(INDEX_FILE.to_owned());
            let index_path = Path::new(&index);
            let dir = Path::new(&dir);
            let out = Path::new(&out);

            if let Some(attach) = attachment_dir {
                let attachments = Some(Path::new(&attach));
                build_vault(dir, out, index_path, attachments);
            } else {
                build_vault(dir, out, index_path, None);
            }
        }
        Commands::Serve { port } => {
            let port_nb = port.unwrap_or(8080);
            serve(port_nb);
        }
    }
}

fn build_vault(
    input_dir: &Path,
    output_dir: &Path,
    index_file: &Path,
    attachment_file: Option<&Path>,
) {
    // Prepare
    // --------------------
    if let Err(e) = validate_build_args(&input_dir, &output_dir, &index_file) {
        println!("{}", e);
    };

    let export_config = exporter::ExportConfig {
        export_all: true,
        attachment_dir: attachment_file,
    };

    let mut builder = exporter::Exporter::new(input_dir, output_dir, &export_config);

    // Do the export
    builder.export();

    // Print outputs
    // ----------------------
    println!("{}", builder.stats);
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
