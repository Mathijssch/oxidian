extern crate oxidian;
use clap::{Parser, Subcommand};
use oxidian::oxidianlib::filesys::{get_all_notes, convert_path};
use oxidian::oxidianlib::note;
use oxidian::oxidianlib::{
    constants::INDEX_FILE,
    errors, //::{self, IndexError},
};

use std::time::Instant;
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
        #[arg(short, long)]
        dir: String,

        /// The output directory
        #[arg(short, long)]
        out: String,

        /// Path to the index file
        #[arg(short, long)]
        index_path: Option<String>,
    },
    /// Launches a server
    #[command()]
    Serve {
        #[arg(short, long)]
        port: Option<u32>,
    },
}

fn serve( port: u32 ){
    println!("Serving on port {}", port);
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Build {
            dir,
            out,
            index_path,
        } => { 
            let index = index_path.unwrap_or(INDEX_FILE.to_owned());
            let index_path = Path::new(&index);
            let dir = Path::new(&dir);
            let out = Path::new(&out);
            build_vault(dir, out, index_path)
        }
        Commands::Serve { port } => { 
            let port_nb = port.unwrap_or(8080);
            serve(port_nb) ;
        },
    }

    //let mut notes = Vec::new();
    //for path in all_paths {
    //notes.push(note::Note::new(&path.unwrap()));
    //}

    //for path in get_all_notes(&args.dir){
    //    println!("{:?}", path.unwrap());
    //}

    //let start = Instant::now();
    //let idx_path = Path::new(&args.dir);
    //let path = idx_path.join(INDEX_FILE);
    //let note = create_note(path.to_str().unwrap());
    //let out_path = Path::new(&args.out).join("index.html");
    //note.to_html(&out_path).unwrap();
    //let duration = start.elapsed();
    //println!("Compiled notes in: {:?}", duration);
    //println!("{:#?}", note);
    //let index_note = read_index(&idx_path).unwrap();

    //let opts = Options::empty();
    //let html_string = note::parse_note(&index_note, opts);
    //write_note(&out_path, &html_string);
}


fn build_vault(input_dir: &Path, output_dir: &Path, index_file: &Path) { 
    let start = Instant::now();
    if let Err(e) = validate_build_args(&input_dir, &output_dir, &index_file) {
        println!("{}", e);
    };

    let all_paths = get_all_notes(input_dir);

    for note_path in all_paths {
        let path = note_path.unwrap();
        println!("Processing note {:?}", path);
        let note = note::Note::new(&path).unwrap();
        let output_file = convert_path(&path, Some("html")).expect("Could not convert the note path to a valid HTML path.");
        println!("First: {:?}", output_file); 
        let relative_path = output_file.strip_prefix(input_dir).unwrap();
        println!("After stripping: {:?}", relative_path);
        let output_path = output_dir.join(relative_path);
        println!("exporting to {:?}", output_path);
        note.to_html(&output_path).expect("Failed to export note");
    }
    let duration = start.elapsed();
    println!("Compiled notes in: {:?}", duration);
}

//fn write_note(path: &Path, content: &str) {
//    // Create the directories recursively if they don't exist
//    if let Some(parent_dir) = path.parent() {
//        std::fs::create_dir_all(parent_dir).expect("Could not create containing directory");
//    }
//    let mut file = File::create(path).expect("Could not create new file");
//    //let file = File::open(path).expect("Could not open file.");
//    file.write_all(content.as_bytes()).unwrap();
//}

//fn read_index(dir: &Path) -> Result<String, errors::IndexError> {
//    let path = dir.join(INDEX_FILE);
//    let mut file = File::open(path).map_err(|_| IndexError::IndexOpenError)?;
//    let mut contents = String::new();
//    file.read_to_string(&mut contents).map_err(|_| IndexError::IndexReadError)?;
//    return Ok(contents);
//}

fn validate_build_args<'a>(input_dir: &'a Path, output_dir: &'a Path, index_file: &'a Path) -> Result<(), InitializeError<'a>> {
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

fn check_contains_index<'a> (input_path: &'a Path, index_file: &'a Path) -> Result<(), MissingIndex<'a>> {
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
