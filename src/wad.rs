use std::path::Path;

use byteorder::{LittleEndian, ByteOrder};

use crate::entry::Entry;
use crate::error::{Error, LoadError};
use crate::iterator::*;

const HEADER_BYTE_SIZE: usize = 12;
const DIRECTORY_ENTRY_BYTE_SIZE: usize = 16;

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

pub type RawEntry = [u8; DIRECTORY_ENTRY_BYTE_SIZE];

impl Wad {
    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub fn len(&self) -> usize {
        self.n_entries
    }

    pub unsafe fn raw_entry_unchecked(&self, index: usize) -> &RawEntry {
        debug_assert!(index < self.len());

        let dir_entry_start = self.directory_offset + DIRECTORY_ENTRY_BYTE_SIZE * index;

        let directory_entry =
            &self.data[dir_entry_start..dir_entry_start + DIRECTORY_ENTRY_BYTE_SIZE];
        debug_assert!(directory_entry.len() == DIRECTORY_ENTRY_BYTE_SIZE);

        // This is safe because the bounds of the entry table were
        // verified in parse_wad
        &*(directory_entry.as_ptr() as *const _)
    }

    pub fn raw_entry(&self, index: usize) -> Result<&RawEntry, Error> {
        verify!(index < self.len(), Error::OutOfBounds);

        Ok(unsafe {
            // This is safe because raw_entry_unchecked only requires us to
            // do bounds checking
            self.raw_entry_unchecked(index)
        })
    }

    pub fn entry_id_from_raw_entry(raw_entry: &RawEntry) -> u64 {
        LittleEndian::read_u64(&raw_entry[8..16])
    }

    pub unsafe fn entry_id_unchecked(&self, index: usize) -> u64 {
        let directory_entry = self.raw_entry_unchecked(index);
        Self::entry_id_from_raw_entry(directory_entry)
    }

    pub fn entry_id(&self, index: usize) -> Result<u64, Error> {
        let directory_entry = self.raw_entry(index)?;
        Ok(Self::entry_id_from_raw_entry(directory_entry))
    }

    pub fn id_iter(&self) -> IdIterator {
        IdIterator::new(self)
    }

    pub fn entry_from_raw_entry(&self, raw_entry: &RawEntry) -> Result<Entry, Error> {
        let start = LittleEndian::read_i32(&raw_entry[0..4]);
        let length = LittleEndian::read_i32(&raw_entry[4..8]);
        let id = LittleEndian::read_u64(&raw_entry[8..16]);

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

        Ok(Entry { id, lump })
    }

    pub unsafe fn entry_unchecked(&self, index: usize) -> Result<Entry, Error> {
        let raw_entry = self.raw_entry_unchecked(index);
        self.entry_from_raw_entry(raw_entry)
    }

    pub fn entry(&self, index: usize) -> Result<Entry, Error> {
        let raw_entry = self.raw_entry(index)?;
        self.entry_from_raw_entry(raw_entry)
    }

    pub fn entry_iter(&self) -> EntryIterator {
        EntryIterator::new(self)
    }
}

impl std::ops::Index<usize> for Wad {
    type Output = [u8];

    fn index(&self, index: usize) -> &Self::Output {
        self.entry(index).unwrap().lump
    }
}

pub fn parse_wad(mut data: Vec<u8>) -> Result<Wad, Error> {
    if data.len() < HEADER_BYTE_SIZE {
        return Err(Error::InvalidLength);
    }

    let kind = match &data[0..4] {
        b"IWAD" => Ok(Kind::IWad),
        b"PWAD" => Ok(Kind::PWad),
        _ => Err(Error::InvalidHeader),
    }?;

    let n_entries = LittleEndian::read_i32(&data[4..8]);
    let directory_offset = LittleEndian::read_i32(&data[8..12]);

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

pub fn load_wad_file(filename: impl AsRef<Path>) -> Result<Wad, LoadError> {
    let data = std::fs::read(filename).map_err(LoadError::IoError)?;
    parse_wad(data).map_err(LoadError::Error)
}
