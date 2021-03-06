use std::path::Path;
use std::slice::SliceIndex;

use byteorder::{ByteOrder, LittleEndian};

use crate::entry::Entry;
use crate::entry_id::EntryId;
use crate::error::{Error, LoadError};
use crate::iterator::*;
use crate::wad_slice::WadSlice;

pub(crate) const HEADER_BYTE_SIZE: usize = 12;
pub(crate) const DIRECTORY_ENTRY_BYTE_SIZE: usize = 16;

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

    fn directory(&self) -> &[RawEntry] {
        let directory = &self.data[self.directory_offset..];

        unsafe {
            // This is safe because the bounds and size of the entry table were
            // verified in parse_wad

            std::slice::from_raw_parts(
                std::mem::transmute(directory.as_ptr()),
                directory.len() / DIRECTORY_ENTRY_BYTE_SIZE,
            )
        }
    }

    pub fn entry_id_from_raw_entry(raw_entry: &RawEntry) -> EntryId {
        WadSlice::entry_id_from_raw_entry(raw_entry)
    }

    pub unsafe fn entry_id_unchecked(&self, index: usize) -> EntryId {
        self.as_slice().entry_id_unchecked(index)
    }

    pub fn entry_id(&self, index: usize) -> Option<EntryId> {
        self.as_slice().entry_id(index)
    }

    pub fn id_iter(&self) -> IdIterator {
        IdIterator::new(self)
    }

    pub fn index_of(&self, id: impl Into<EntryId>) -> Option<usize> {
        self.as_slice().index_of(id)
    }

    pub fn entry_from_raw_entry(&self, raw_entry: &RawEntry) -> Result<Entry, Error> {
        self.as_slice().entry_from_raw_entry(raw_entry)
    }

    pub unsafe fn entry_unchecked(&self, index: usize) -> Result<Entry, Error> {
        self.as_slice().entry_unchecked(index)
    }

    pub fn entry(&self, index: usize) -> Result<Entry, Error> {
        self.as_slice().entry(index)
    }

    pub fn entry_iter(&self) -> EntryIterator {
        EntryIterator::new(self)
    }

    pub fn by_id(&self, id: impl Into<EntryId>) -> Option<&[u8]> {
        self.as_slice().by_id(id)
    }

    pub fn slice(&self, slice_index: impl SliceIndex<[RawEntry], Output = [RawEntry]>) -> WadSlice {
        self.as_slice().slice(slice_index)
    }

    pub fn as_slice(&self) -> WadSlice {
        WadSlice::new(&self.data[0..self.directory_offset], self.directory())
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
