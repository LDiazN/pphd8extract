use std::fmt::Display;
use std::fs::{self, File};
use std::io::Write;
use std::mem::size_of;
use std::os::windows::fs::FileExt;
use std::path::Path;

/// All the data that we know how to extract from a PPHD8 file.
///
/// PPHD8 files have the following structure:
/// - A **metadata section**, where it specifies other sections in the same file
/// - An **index section**, where it lists all files contained by the PPHD8
/// - The actual **raw data** for several VAG files
pub struct PPHD8FileData {
    start_of_index: u32, // Start of the index section, extracted from the word at 0x38
    vag_entries: Vec<VAGFileEntry>, // All entries inside the file
    n_files: usize,      // Not extracted but can be computed from the index
    start_of_data: u32,  // Start of the data section, extracted from the word at 0xC
    file: File,          // The file we are reading
}

/// A VAG file entry in a PPHD8 file, as it comes from the index section.
struct VAGFileEntry {
    frequency: u32,
    size: u32,
    //how far from the start of the data section in the pphd8 file is this file
    offset_from_data_start: usize,
}

/// A VAG file extracted from the PPHD8 file
pub struct VAGFile {
    frequency: u32,
    size: u32,
    channels: u32,
    filename: [u8; 32],
    body: Vec<u8>,
}

/// Possible errors that could happen when parsing a VAG file
pub enum ParseError {
    IOError(std::io::Error),
    IncompleteVag {
        entry_index: usize,
        expected_size: u32,
        actual_size: u32,
    },
}

macro_rules! read_from_file {
    ($file_variable:ident, $type_name:ident, $offset:expr) => {
        unsafe {
            let mut buffer = [0u8; std::mem::size_of::<$type_name>()];
            let result = $file_variable.seek_read(&mut buffer, $offset)?;
            assert_eq!(result, std::mem::size_of::<$type_name>());

            *(buffer.as_mut_ptr().cast::<u32>())
        }
    };
}

impl PPHD8FileData {
    /// Parse a VAG file from a file.
    pub fn parse_from_file(filename: &Path) -> Result<PPHD8FileData, ParseError> {
        // Try to open file:
        let file = fs::File::open(filename)?;
        let start_of_index = read_from_file!(file, u32, 0x38) + 16 * 4;
        let start_of_data = read_from_file!(file, u32, 0xC);

        // Try to parse vag file entries
        let mut vag_entries = vec![];
        let mut n_files = 0;
        let mut index_iterator = start_of_index;

        while index_iterator + ((size_of::<u32>() * 3) as u32) < start_of_data {
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

            vag_entries.push(VAGFileEntry {
                frequency,
                size,
                offset_from_data_start,
            });
            n_files += 1;
        }

        Ok(PPHD8FileData {
            start_of_index,
            start_of_data,
            vag_entries,
            n_files,
            file,
        })
    }

    /// Get all VAG files inside this PPHD8File
    pub fn get_vag_files(&self) -> Result<Vec<VAGFile>, ParseError> {
        let mut results = vec![];

        for (i, vag_entry) in self.vag_entries.iter().enumerate() {
            let mut buff = vec![0u8; vag_entry.size as usize];
            let offset = vag_entry.offset_from_data_start + (self.start_of_data as usize);

            // Read vag file from open file.
            let result = self.file.seek_read(&mut buff, offset as u64)?;
            if result != vag_entry.size as usize {
                return Err(ParseError::IncompleteVag {
                    entry_index: i,
                    expected_size: vag_entry.size,
                    actual_size: result as u32,
                });
            }

            let mut filename = [0u8; 32];
            filename[0] = b'L';
            filename[1] = b'D';
            results.push(VAGFile {
                frequency: vag_entry.frequency,
                size: vag_entry.size,
                channels: 0x00000003,
                filename,
                body: buff,
            });
        }

        Ok(results)
    }
}

impl VAGFile {
    /// Writes this VAG file to the specified file
    pub fn write_to_file(&self, filepath: &Path) -> Result<(), std::io::Error> {
        let mut new_file = fs::File::create(filepath)?;
        let file_format_buff = [b'V', b'A', b'G', b'p'];
        let mut channels_buff = get_buff_for_num(self.channels);
        channels_buff.reverse();

        let zero_buff = [0u8; 4];
        let mut len_buff = get_buff_for_num(self.size);
        len_buff.reverse();
        let mut freq_buff = get_buff_for_num(self.frequency);
        freq_buff.reverse();

        // Actually write the file
        let _written = new_file.write(&file_format_buff)?;
        let _written = new_file.write(&channels_buff)?;
        let _written = new_file.write(&zero_buff)?;
        let _written = new_file.write(&len_buff)?;
        let _written = new_file.write(&freq_buff)?;
        let _written = new_file.write(&[0u8; 12])?;
        let _written = new_file.write(&self.filename)?;
        let _written = new_file.write(self.body.as_slice())?;

        Ok(())
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::IOError(error) => write!(f, "Could not operate file. Error: {}", error)?,
            ParseError::IncompleteVag {
                entry_index,
                expected_size,
                actual_size } =>
                    write!(f,
                        "Unable to read full VAG body from file, incomplete body. VAG entry: {entry_index}. Expected size: {expected_size}, Actual size: {actual_size}")?
        }
        Ok(())
    }
}

impl Display for PPHD8FileData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "start_of_index: {}", self.start_of_index)?;
        writeln!(f, "start_of_data: {}", self.start_of_data)?;
        writeln!(f, "n_files: {}", self.n_files)?;
        writeln!(f, "vag_entries:")?;

        for (i, entry) in self.vag_entries.iter().enumerate() {
            writeln!(f, "\t- Entry: {i}")?;
            writeln!(f, "\t\t+ frequency: {}", entry.frequency)?;
            writeln!(f, "\t\t+ size: {}", entry.size)?;
            writeln!(
                f,
                "\t\t+ offset_from_data_start: {}",
                entry.offset_from_data_start
            )?;
        }

        Ok(())
    }
}

impl From<std::io::Error> for ParseError {
    fn from(value: std::io::Error) -> Self {
        ParseError::IOError(value)
    }
}

/// Get a buffer (byte array) from a number
fn get_buff_for_num(num: u32) -> [u8; 4] {
    unsafe {
        let mut buff = [0u8; std::mem::size_of::<u32>()];
        buff.as_mut_ptr().cast::<u32>().write(num);
        buff
    }
}
