use std::slice::SliceIndex;
use byteorder::{ByteOrder, LittleEndian};

use crate::entry::Entry;
use crate::entry_id::EntryId;
use crate::error::Error;
use crate::iterator::*;
use crate::wad::*;

pub struct WadSlice<'a> {
    data: &'a [u8],
    directory: &'a [RawEntry],
}

impl<'a> WadSlice<'a> {
    pub(crate) fn new<'n>(data: &'n [u8], directory: &'n [RawEntry]) -> WadSlice<'n> {
        WadSlice { data, directory }
    }

    pub fn len(&self) -> usize {
        self.directory.len()
    }

    pub fn entry_id_from_raw_entry(raw_entry: &RawEntry) -> EntryId {
        // This is safe because the static size of RawEntry is bigger than
        // the size of the requested slice:
        let id = unsafe { &*(raw_entry[8..16].as_ptr() as *const _) };
        EntryId::from_bytes(id)
    }

    pub unsafe fn entry_id_unchecked(&self, index: usize) -> EntryId {
        let directory_entry = self.directory.get_unchecked(index);
        Self::entry_id_from_raw_entry(directory_entry)
    }

    pub fn entry_id(&self, index: usize) -> Option<EntryId> {
        let directory_entry = self.directory.get(index)?;
        Some(Self::entry_id_from_raw_entry(directory_entry))
    }

    pub fn id_iter(&self) -> SliceIdIterator {
        SliceIdIterator::new(self)
    }

    pub fn entry_from_raw_entry(&self, raw_entry: &RawEntry) -> Result<Entry, Error> {
        let start = LittleEndian::read_i32(&raw_entry[0..4]);
        let length = LittleEndian::read_i32(&raw_entry[4..8]);
        let id = Self::entry_id_from_raw_entry(raw_entry);

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
        verify!(end <= self.data.len(), Error::InvalidEntry);

        let lump = &self.data[start..end];

        Ok(Entry { id, lump })
    }

    pub unsafe fn entry_unchecked(&self, index: usize) -> Result<Entry, Error> {
        let raw_entry = self.directory.get_unchecked(index);
        self.entry_from_raw_entry(raw_entry)
    }

    pub fn entry(&self, index: usize) -> Result<Entry, Error> {
        let raw_entry = self.directory.get(index).ok_or(Error::OutOfBounds)?;
        self.entry_from_raw_entry(raw_entry)
    }

    pub fn entry_iter(&self) -> SliceEntryIterator {
        SliceEntryIterator::new(self)
    }

    pub fn slice(&self, slice_index: impl SliceIndex<[RawEntry], Output = [RawEntry]>) -> WadSlice<'a> {
        WadSlice::new(
            self.data,
            &self.directory[slice_index],
        )
    }
}

impl<'a> std::ops::Index<usize> for WadSlice<'a> {
    type Output = [u8];

    fn index(&self, index: usize) -> &Self::Output {
        self.entry(index).unwrap().lump
    }
}

impl<'a> From<&'a Wad> for WadSlice<'a> {
    fn from(wad: &'a Wad) -> WadSlice<'a> {
        wad.as_slice()
    }
}
