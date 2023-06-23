extern crate oxidian;
use clap::Parser;
use oxidian::oxidianlib::{
    errors,//::{self, IndexError},
    constants::INDEX_FILE, 
    note::create_note 
};
use std::path::Path;
use oxidian::oxidianlib::utils::filter_markdown_html_files;

type MissingDirectory<'a> = errors::MissingDirectoryError<&'a Path>; 
type MissingIndex<'a> = errors::MissingIndexError<&'a Path>;
type ExistingOutput<'a> = errors::DirExistsError<&'a Path>;
type InitializeError<'a> = errors::InitializationError<&'a Path>;
use std::time::Instant;

// -------------------------------------------
// CLI
// -------------------------------------------

/// Compile the notes in a given directory into a static website.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The directory containing the notes
    #[arg(short, long)]
    dir: String,

    /// The output directory
    #[arg(short, long)]
    out: String,
}


fn main() {
    let args = Args::parse();
    if let Err(e) = validate_args(&args){
        println!("{}", e);
    };
    
    for path in filter_markdown_html_files(&args.dir){
        println!("{:?}", path.unwrap());
    }


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

fn validate_args(args: &Args) -> Result<(), InitializeError>{
    let input_path = Path::new(&args.dir);
    check_exists(input_path).map_err(
        |e| errors::InitializationError::<&Path>::MissingDirectory(e)
    )?;
    check_contains_index(input_path).map_err(
        |e| errors::InitializationError::<&Path>::MissingIndexError(e)
    )?;
    let output_path = Path::new(&args.out);
    check_output_available(output_path).map_err(
        |e| errors::InitializationError::<&Path>::OutputDirExists(e)
    )?;
    Ok(())
}

fn check_exists(input_path: &Path) -> Result<(), MissingDirectory> {
    if !input_path.exists() {
        return Err(errors::MissingDirectoryError(input_path));
    }
    Ok(())
}

fn check_contains_index(input_path: &Path) -> Result<(), MissingIndex> {
    if !input_path.join(Path::new(INDEX_FILE)).exists() {
       return Err(errors::MissingIndexError(input_path)); 
    } 
    Ok(())
}

fn check_output_available(output_path: &Path) -> Result<(), ExistingOutput> {
    if output_path.exists() {
        return Err(errors::DirExistsError(output_path));
    }
    Ok(())
}
