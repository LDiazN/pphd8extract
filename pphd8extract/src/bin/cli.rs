// Rust imports
use std::env;
use std::path::{Path, PathBuf};
use std::process::exit;

// Third party imports
use rayon::prelude::*;
use clap::Parser;

// Local imports
extern crate pphd8extract;
use pphd8extract::pphd8parser::PPHD8FileData;


/// Extract the content of a pphd8 file, getting the list of VAG files
#[derive(Parser, Debug)]
#[command(author = "Luis Diaz", version, about, long_about = None)]
struct CLI {
    /// file to decompress
    pphd8_file: PathBuf,

    /// Where to save resulting VAG files
    target_dir: PathBuf,

    /// Print more details about the extraction process. Defaults to false.
    #[arg(short, long, default_value_t = false)]
    verbose : bool
}

impl CLI {

    /// Checks if the arguments are consistent. 
    /// If not, handle errors and exit the process
    fn check(&self)
    {
        let pphd8_file = self.pphd8_file.as_path();
        let target_dir = self.target_dir.as_path();
        
        Self::check_file_exists(pphd8_file);
        Self::check_file_exists(target_dir);
        Self::check_file_is_dir(target_dir);
    }

    fn check_file_exists(path : &Path)
    {
        if path.exists()
        { // nothing to do
            return
        }

        eprintln!("Error: File '{}' does not exists", path.display());
        exit(1);
    }

    fn check_file_is_dir(path : &Path)
    {
        if path.is_dir()
        { // nothing to do
            return;
        }

        eprintln!("Error: File '{}' is not a directory", path.display());
        exit(1);
    }
}

fn main() {

    let args = CLI::parse();
    args.check();

    let pphd8_file = PPHD8FileData::parse_from_file(&args.pphd8_file);

    let file = match pphd8_file {
        Err(e) => {
            eprint!("{}", e);
            exit(1)
        }
        Ok(file) => file,
    };

    println!("Successfully extracted data from PPHD8File!");
    if args.verbose
    {
        println!("Result:");
        println!("{}", file);
    }

    println!("Extracting VAG file content...");
    let vags = match file.get_vag_files() {
        Err(e) => {
            eprint!("{}", e);
            exit(1);
        }
        Ok(vags) => vags,
    };
    println!("VAG Files successfully parsed!");
    println!("Extracting VAG files...");

    let output_dir_path = args.target_dir;

    let errors: Vec<(usize, std::io::Error)> = vags
        .par_iter()
        .enumerate()
        .map(|(i, vag)| {
            let output_vag_filepath = output_dir_path.join(format!("extracted_{i}.vag"));
            let output_vag_filepath = output_vag_filepath.as_path();
            println!(
                "Extracting file {i} to {}...",
                output_vag_filepath.display()
            );
            (
                i,
                vag.write_to_file(&output_vag_filepath.to_str().unwrap().to_string()),
            )
        })
        .filter_map(|(i, output)| match output {
            Err(e) => Some((i, e)),
            _ => None,
        })
        .collect();

    if !errors.is_empty() {
        eprintln!("Some files could not be extracted:");
        for (i, error) in errors {
            eprintln!("VAG File {i} could not be extracted. Error: {error}");
        }
    } else {
        println!("All files successfully extracted!");
    }
}
