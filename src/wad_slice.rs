use byteorder::{ByteOrder, LittleEndian};

use crate::entry::Entry;
use crate::error::Error;
use crate::iterator::*;
use crate::wad::*;

pub struct WadSlice<'a> {
    kind: Kind,
    data: &'a [u8],
    directory: &'a [u8], // FIXME Change to &[RawEntry]?
}

impl<'a> WadSlice<'a> {
    pub(crate) fn new<'n>(kind: Kind, data: &'n [u8], directory: &'n [u8]) -> WadSlice<'n> {
        WadSlice {
            kind,
            data,
            directory,
        }
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub fn len(&self) -> usize {
        self.directory.len() / DIRECTORY_ENTRY_BYTE_SIZE
    }

    pub unsafe fn raw_entry_unchecked(&self, index: usize) -> &RawEntry {
        debug_assert!(index < self.len());

        let dir_entry_start = DIRECTORY_ENTRY_BYTE_SIZE * index;

        let directory_entry =
            &self.directory[dir_entry_start..dir_entry_start + DIRECTORY_ENTRY_BYTE_SIZE];
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

    pub fn id_iter(&self) -> SliceIdIterator {
        SliceIdIterator::new(self)
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
        verify!(end <= self.data.len(), Error::InvalidEntry);

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

    pub fn entry_iter(&self) -> SliceEntryIterator {
        SliceEntryIterator::new(self)
    }
}

impl<'a> std::ops::Index<usize> for WadSlice<'a> {
    type Output = [u8];

    fn index(&self, index: usize) -> &Self::Output {
        self.entry(index).unwrap().lump
    }
}
