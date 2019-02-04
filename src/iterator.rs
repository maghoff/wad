use crate::Entry;
use crate::Wad;

pub struct EntryIterator<'a> {
    index: usize,
    wad: &'a Wad,
}

impl<'a> EntryIterator<'a> {
    pub(crate) fn new<'b>(wad: &'b Wad) -> EntryIterator<'b> {
        EntryIterator { index: 0, wad }
    }
}

impl<'a> Iterator for EntryIterator<'a> {
    type Item = Entry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.wad.len() {
            self.index += 1;
            return Some(unsafe {
                // This is safe because entry_unchecked only elides the bounds
                // check, and we do bounds checking in this function
                self.wad.entry_unchecked(self.index - 1).unwrap()
            });
        } else {
            return None;
        }
    }
}

pub struct IdIterator<'a> {
    index: usize,
    wad: &'a Wad,
}

impl<'a> IdIterator<'a> {
    pub(crate) fn new<'b>(wad: &'b Wad) -> IdIterator<'b> {
        IdIterator { index: 0, wad }
    }
}

impl<'a> Iterator for IdIterator<'a> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.wad.len() {
            self.index += 1;
            return Some(unsafe {
                // This is safe because entry_unchecked only elides the bounds
                // check, and we do bounds checking in this function
                self.wad.entry_id_unchecked(self.index - 1)
            });
        } else {
            return None;
        }
    }
}
