use std::io::Cursor;
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt};

const HEADER_BYTE_SIZE: usize = 12;
const DIRECTORY_ENTRY_BYTE_SIZE: usize = 16;

#[derive(Debug, Copy, Clone)]
pub enum Kind {
    IWad,
    PWad,
}

#[derive(Debug)]
pub enum ParseError {
    Invalid,
    InvalidLength,
    InvalidHeader,
}

#[derive(Debug)]
pub enum LoadError {
    ParseError(ParseError),
    IoError(std::io::Error),
}

pub struct Wad {
    kind: Kind,
    data: Vec<u8>,
    directory_offset: usize,
    n_entries: usize,
}

impl Wad {
    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub fn len(&self) -> usize {
        self.n_entries
    }

    pub fn entry(&self, index: usize) -> (&str, &[u8]) {
        assert!(index < self.len());

        let dir_entry_start = self.directory_offset +
            DIRECTORY_ENTRY_BYTE_SIZE * index;

        let directory_entry = &self.data[
            dir_entry_start
            ..
            dir_entry_start + DIRECTORY_ENTRY_BYTE_SIZE
        ];

        let mut rdr = Cursor::new(directory_entry);
        let start = rdr.read_i32::<LittleEndian>().expect("Invariant");
        let length = rdr.read_i32::<LittleEndian>().expect("Invariant");

        assert!(length >= 0);
        let length = length as usize;

        assert!(start >= 0);
        let mut start = start as usize;

        // If length == 0, start doesn't matter. Some directory entries in
        // official doom wads have start == 0, which is really too early.
        if length == 0 {
            start = HEADER_BYTE_SIZE;
        }

        assert!(start >= HEADER_BYTE_SIZE);

        let end = start.checked_add(length).unwrap();
        assert!(end <= self.directory_offset);

        let lump = &self.data[start..end];

        let name = &directory_entry[8..16];
        let name = name.split(|&x| x == 0).next().unwrap();
        assert!(name.len() > 0);
        assert!(name.iter().all(|&x| x & 0x80 == 0)); // ASCII
        let name = std::str::from_utf8(name).unwrap();

        (name, lump)
    }

    pub fn iter(&self) -> WadIterator {
        WadIterator::new(self)
    }
}

pub struct WadIterator<'a> {
    index: usize,
    wad: &'a Wad,
}

impl<'a> WadIterator<'a> {
    fn new<'b>(wad: &'b Wad) -> WadIterator<'b> {
        WadIterator {
            index: 0,
            wad,
        }
    }
}

impl<'a> Iterator for WadIterator<'a> {
    type Item = (&'a str, &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.wad.len() {
            self.index += 1;
            return Some(self.wad.entry(self.index - 1));
        } else {
            return None;
        }
    }
}

impl std::ops::Index<usize> for Wad {
    type Output = [u8];

    fn index(&self, index: usize) -> &Self::Output {
        self.entry(index).1
    }
}

pub fn parse_wad(mut data: Vec<u8>) -> Result<Wad, ParseError> {
    if data.len() < HEADER_BYTE_SIZE {
        return Err(ParseError::InvalidLength);
    }

    let kind = match &data[0..4] {
        b"IWAD" => Ok(Kind::IWad),
        b"PWAD" => Ok(Kind::PWad),
        _ => Err(ParseError::InvalidHeader),
    }?;

    let mut rdr = Cursor::new(&data[4..12]);
    let n_entries = rdr.read_i32::<LittleEndian>().expect("Checked by guard at top");
    let directory_offset = rdr.read_i32::<LittleEndian>().expect("Checked by guard at top");

    if n_entries < 0 || directory_offset < 0 {
        return Err(ParseError::Invalid);
    }

    let n_entries = n_entries as usize;
    let directory_offset = directory_offset as usize;

    let expected_directory_length = n_entries
        .checked_mul(DIRECTORY_ENTRY_BYTE_SIZE)
        .ok_or(ParseError::Invalid)?;

    let expected_binary_length = directory_offset
        .checked_add(expected_directory_length)
        .ok_or(ParseError::Invalid)?;

    if data.len() < expected_binary_length {
        return Err(ParseError::InvalidLength);
    }
    data.truncate(expected_binary_length);

    Ok(Wad {
        kind,
        data,
        directory_offset,
        n_entries,
    })
}

pub fn load_wad_file(filename: impl AsRef<Path>)
    -> Result<Wad, LoadError>
{
    let data = std::fs::read(filename).map_err(LoadError::IoError)?;
    parse_wad(data).map_err(LoadError::ParseError)
}
