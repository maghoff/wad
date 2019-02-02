use crate::error::{Error, LoadError};
use crate::entry::Entry;
use crate::wad_iterator::WadIterator;

#[derive(Debug, Copy, Clone)]
pub enum Kind {
    IWad,
    PWad,
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

    pub fn entry(&self, index: usize) -> Result<Entry, Error> {
        verify!(index < self.len(), Error::OutOfBounds);

        let dir_entry_start = self.directory_offset +
            DIRECTORY_ENTRY_BYTE_SIZE * index;

        let directory_entry = &self.data[
            dir_entry_start
            ..
            dir_entry_start + DIRECTORY_ENTRY_BYTE_SIZE
        ];

        let mut rdr = Cursor::new(directory_entry);
        let start = rdr.read_i32::<LittleEndian>().expect("Struct invariant");
        let length = rdr.read_i32::<LittleEndian>().expect("Struct invariant");
        let id = rdr.read_u64::<LittleEndian>().expect("Struct invariant");

        verify!(length >= 0, Error::InvalidEntry);
        let length = length as usize;

        verify!(start >= 0, Error::InvalidEntry);
        let mut start = start as usize;

        // If length == 0, start doesn't matter. Some directory entries in
        // official doom wads have start == 0, which is really too early.
        if length == 0 {
            start = HEADER_BYTE_SIZE;
        }

        verify!(start >= HEADER_BYTE_SIZE, Error::InvalidEntry);

        let end = start.checked_add(length).ok_or(Error::InvalidEntry)?;
        verify!(end <= self.directory_offset, Error::InvalidEntry);

        let lump = &self.data[start..end];

        Ok(Entry {
            id,
            lump,
        })
    }

    pub fn iter(&self) -> WadIterator {
        WadIterator::new(self)
    }
}

impl std::ops::Index<usize> for Wad {
    type Output = [u8];

    fn index(&self, index: usize) -> &Self::Output {
        self.entry(index).unwrap().lump
    }
}
use std::io::Cursor;
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt};

const HEADER_BYTE_SIZE: usize = 12;
const DIRECTORY_ENTRY_BYTE_SIZE: usize = 16;

pub fn parse_wad(mut data: Vec<u8>) -> Result<Wad, Error> {
    if data.len() < HEADER_BYTE_SIZE {
        return Err(Error::InvalidLength);
    }

    let kind = match &data[0..4] {
        b"IWAD" => Ok(Kind::IWad),
        b"PWAD" => Ok(Kind::PWad),
        _ => Err(Error::InvalidHeader),
    }?;

    let mut rdr = Cursor::new(&data[4..12]);
    let n_entries = rdr.read_i32::<LittleEndian>().expect("Checked by guard at top");
    let directory_offset = rdr.read_i32::<LittleEndian>().expect("Checked by guard at top");

    if n_entries < 0 || directory_offset < 0 {
        return Err(Error::Invalid);
    }

    let n_entries = n_entries as usize;
    let directory_offset = directory_offset as usize;

    let expected_directory_length = n_entries
        .checked_mul(DIRECTORY_ENTRY_BYTE_SIZE)
        .ok_or(Error::Invalid)?;

    let expected_binary_length = directory_offset
        .checked_add(expected_directory_length)
        .ok_or(Error::Invalid)?;

    if data.len() < expected_binary_length {
        return Err(Error::InvalidLength);
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
    parse_wad(data).map_err(LoadError::Error)
}
