use std::path::Path;
use std::process::exit;
use std::{fs, os::windows::fs::FileExt, io::Write};
extern crate pphd8extract;
use pphd8extract::pphd8parser::PPHD8FileData;
use std::env;

macro_rules! read_from_file {
    ($file_variable:ident, $type_name:ident, $offset:expr) => {
        unsafe{
            let mut buffer = [0u8; std::mem::size_of::<$type_name>()];
            let result = $file_variable.seek_read(&mut buffer, $offset).expect("Couldn't read for some reason");
            assert_eq!(result, std::mem::size_of::<$type_name>());

            *(buffer.as_mut_ptr().cast::<u32>())
        }
    };
}

fn read_line_of_file(file : &fs::File, position : u64) -> [u8; 16]
{
    let mut buff = [0u8; 16];
    file.seek_read(&mut buff, position).expect("Should be able to read file");
    return buff;
}

fn line_is_all_zeros(line : &[u8] )-> bool
{
    for &b in line
    {
        if b != 0
        {
            return false;
        }
    }
    return true;
}

fn get_buff_for_num(num : u32) -> [u8; 4]
{
    unsafe 
    {
        let mut buff = [0u8; std::mem::size_of::<u32>()];
        buff.as_mut_ptr().cast::<u32>().write(num);
        buff
    }
}

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
    for (i, vag) in vags.iter().enumerate() {
        let output_vag_filepath = output_dir_path
                                            .join(
                                                format!("extracted_{i}.vag")
                                            );
        let output_vag_filepath = output_vag_filepath.as_path();

        println!("Extracting file {i} to {}...", output_vag_filepath.display());
        let result = vag.write_to_file(&output_vag_filepath.to_str().unwrap().to_string());

        match result 
        {
            Err(e) => {eprintln!("Could not write vag file! Error: {e}"); exit(1)},
            _ => {}
        }
    }

    println!("All files successfully extracted!");
}