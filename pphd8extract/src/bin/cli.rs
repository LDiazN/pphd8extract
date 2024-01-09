use std::path::Path;
use std::process::exit;
use std::env;

use rayon::prelude::*;

extern crate pphd8extract;
use pphd8extract::pphd8parser::PPHD8FileData;

fn main()
{
    let args : Vec<String> = env::args().collect();

    if args.len() < 2
    {
        eprintln!("Error: Missing filepath argument");
        exit(1);
    }
    let filename = &args[1];

    if args.len() < 3
    {
        eprintln!("Error: Missing output directory argument");
        exit(1);
    }
    let output_dir = &args[2];

    let pphd8_file = PPHD8FileData::parse_from_file(filename);

    let file = match pphd8_file {
        Err(e) => {eprint!("{}", e.to_string()); exit(1)},
        Ok(file) => file
    };

    println!("Successfully extracted data from PPHD8File!");
    println!("Result:");
    println!("{}", file.to_string());

    println!("Extracting VAG file content...");
    let vags = match file.get_vag_files() 
    {
        Err(e) => {eprint!("{}", e.to_string()); exit(1);},
        Ok(vags) => vags
    };
    println!("VAG Files successfully parsed!");
    println!("Extracting VAG files...");

    let output_dir_path = Path::new(output_dir);

    let errors : Vec<(usize,std::io::Error)> = vags.par_iter()
        .enumerate()
        .map(|(i, vag)|{
            let output_vag_filepath = output_dir_path.join(format!("extracted_{i}.vag"));
            let output_vag_filepath = output_vag_filepath.as_path();
            println!("Extracting file {i} to {}...", output_vag_filepath.display());
            (i, vag.write_to_file(&output_vag_filepath.to_str().unwrap().to_string()))
    })  .filter_map(
        |(i, output)| 
                    match output { Err(e) => Some((i, e)), _ => None }
            )
        .collect();

    if !errors.is_empty()
    {
        eprintln!("Some files could not be extracted:");
        for (i, error) in errors
        {
            eprintln!("VAG File {i} could not be extracted. Error: {error}");
        }
    }
    else 
    {
        println!("All files successfully extracted!");
    }

    return
}