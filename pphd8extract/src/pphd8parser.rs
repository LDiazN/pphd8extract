use std::fs::{self, File};
use std::mem::size_of;
use std::os::windows::fs::FileExt;

/// All the data that we know how to extract from a PPHD8 file
pub struct PPHD8FileData
{
    start_of_index : u32,
    vag_entries : Vec<VAGFileEntry>,
    n_files : usize, // still don't know if this exists
    start_of_data : u32,
    file : File
}

/// A VAG file entry in a PPHD8 file
struct VAGFileEntry
{
    frequency : u32,
    size : u32,
    //how far from the start of the data section in the pphd8 file is this file
    offset_from_data_start : usize, 
}

pub struct VAGFile
{
    frequency : u32,
    size : u32,
    channels : u32,
    filename : [u8; 32],
    body : Vec<u8>
}

pub enum ParseError 
{
    IOError(std::io::Error),
    IncompleteVag{entry_index : usize, expected_size : u32, actual_size : u32}
}

macro_rules! read_from_file {
    ($file_variable:ident, $type_name:ident, $offset:expr) => {
        unsafe{
            let mut buffer = [0u8; std::mem::size_of::<$type_name>()];
            let result = $file_variable.seek_read(&mut buffer, $offset)?;
            assert_eq!(result, std::mem::size_of::<$type_name>());

            *(buffer.as_mut_ptr().cast::<u32>())
        }
    };
}

impl PPHD8FileData 
{
    pub fn parse_from_file(filename : &String) -> Result<PPHD8FileData, ParseError>
    {
        // Try to open file:
        let file = fs::File::open(filename)?;
        let start_of_index = read_from_file!(file, u32, 0x38) + 16*4;
        let start_of_data = read_from_file!(file, u32, 0xC);

        // Try to parse vag file entries
        let mut vag_entries = vec![];
        let mut n_files = 0;
        let mut index_iterator = start_of_index;

        while index_iterator + ((size_of::<u32>() * 3) as u32) < start_of_data
        {
            let offset_from_data_start = read_from_file!(file, u32, index_iterator as u64) as usize;
            index_iterator += size_of::<u32>() as u32;
            let frequency = read_from_file!(file, u32, index_iterator as u64);
            index_iterator += size_of::<u32>() as u32;
            let size = read_from_file!(file, u32, index_iterator as u64);
            index_iterator += size_of::<u32>() as u32;

            // padding
            index_iterator += size_of::<u32>() as u32;

            // If all three values are 0xffffffff, it means this is a null entry, we don't count it
            if offset_from_data_start == 0xFFFFFFFF && frequency == 0xFFFFFFFF && size == 0xFFFFFFFF
            {
                continue;
            }

            vag_entries.push(VAGFileEntry { frequency, size, offset_from_data_start });
            n_files += 1;
        }

        Ok(
            PPHD8FileData {
                start_of_index,
                start_of_data,
                vag_entries,
                n_files,
                file
            }
        )
    }

    pub fn to_string(&self) -> String
    {
        let mut lines : Vec<String> = vec![];

        lines.push(format!("start_of_index: {}\n", self.start_of_index));
        lines.push(format!("start_of_data: {}\n", self.start_of_data));
        lines.push(format!("n_files: {}\n", self.n_files));
        lines.push(format!("vag_entries:\n"));

        for (i,entry) in self.vag_entries.iter().enumerate()
        {
            lines.push(format!("\t- Entry: {i}\n"));
            lines.push(format!("\t\t+ frequency: {}\n", entry.frequency));
            lines.push(format!("\t\t+ size: {}\n", entry.size));
            lines.push(format!("\t\t+ offset_from_data_start: {}\n", entry.offset_from_data_start));
        }

        lines.concat()
    }

    pub fn get_vag_files(&self) -> Result<Vec<VAGFile>, ParseError>
    {
        let mut results = vec![];

        for (i, vag_entry) in self.vag_entries.iter().enumerate()
        {
            let mut buff = vec![0u8; vag_entry.size as usize];
            let offset = vag_entry.offset_from_data_start + (self.start_of_data as usize);

            // Read vag file from open file.
            let result = self.file.seek_read(&mut buff, offset as u64)?;
            if result != vag_entry.size as usize
            {
                return Err(
                    ParseError::IncompleteVag { 
                        entry_index: i, 
                        expected_size: vag_entry.size, 
                        actual_size: result as u32
                    })
            }

            let mut filename = [0u8; 32];
            filename[0] = 'L' as u8;
            filename[1] = 'D' as u8;
            results.push(VAGFile{
                frequency : vag_entry.frequency,
                size : vag_entry.size,
                channels : 0x00000003,
                filename, 
                body: buff
            });
        }

        Ok(results)
    }

}

impl ParseError
{
    pub fn to_string(&self) -> String
    {
        match self {
            ParseError::IOError(error) => format!("Could not operate file. Error: {}", error.to_string()),
            ParseError::IncompleteVag { 
                entry_index, 
                expected_size, 
                actual_size } => 
                    format!(
                        "Unable to read full VAG body from file, incomplete body. VAG entry: {entry_index}. Expected size: {expected_size}, Actual size: {actual_size}")
        }
    }
}

impl From<std::io::Error> for ParseError
{
    fn from(value: std::io::Error) -> Self {
        ParseError::IOError(value)
    }
}

