// Rust imports
use std::path::{Path, PathBuf};
use std::process::exit;

// Third party imports
use clap::Parser;
use rayon::prelude::*;

// Local imports
extern crate pphd8extract;
use pphd8extract::pphd8parser::{PPHD8FileData, VAGFile};

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
    verbose: bool,
}

impl CLI {
    /// Checks if the arguments are consistent.
    /// If not, handle errors and exit the process
    fn check(&self) {
        let pphd8_file = self.pphd8_file.as_path();
        let target_dir = self.target_dir.as_path();

        Self::check_file_exists(pphd8_file);
        Self::check_file_exists(target_dir);
        Self::check_file_is_dir(target_dir);
    }

    /// Runs the application
    fn run(&self) {
        let file = self.parse_pphd8();
        let vags = self.extract_vag_files(&file);
        self.save_vag_files(&vags);
    }

    fn parse_pphd8(&self) -> PPHD8FileData {
        let pphd8_file = PPHD8FileData::parse_from_file(&self.pphd8_file);
        let file = match pphd8_file {
            Err(e) => {
                eprint!("{}", e);
                exit(1)
            }
            Ok(file) => file,
        };

        println!("Successfully extracted data from PPHD8File!");
        if self.verbose {
            println!("Result:");
            println!("{}", file);
        }

        file
    }

    fn extract_vag_files(&self, file: &PPHD8FileData) -> Vec<VAGFile> {
        println!("Extracting VAG file content...");
        let vags = match file.get_vag_files() {
            Err(e) => {
                eprint!("{}", e);
                exit(1);
            }
            Ok(vags) => vags,
        };
        println!("VAG Files successfully parsed!");

        return vags;
    }

    fn save_vag_files(&self, vags: &Vec<VAGFile>) {
        println!("Saving VAG files...");

        let errors: Vec<(usize, std::io::Error)> = vags
            .par_iter()
            .enumerate()
            .map(|(i, vag)| {
                let output_vag_filepath = self.target_dir.join(format!("extracted_{i}.vag"));
                let output_vag_filepath = output_vag_filepath.as_path();
                println!("Saving file {i} to {}...", output_vag_filepath.display());
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

    fn check_file_exists(path: &Path) {
        if path.exists() {
            // nothing to do
            return;
        }

        eprintln!("Error: File '{}' does not exists", path.display());
        exit(1);
    }

    fn check_file_is_dir(path: &Path) {
        if path.is_dir() {
            // nothing to do
            return;
        }

        eprintln!("Error: File '{}' is not a directory", path.display());
        exit(1);
    }
}

fn main() {
    let cli = CLI::parse();
    cli.check();
    cli.run();
}
