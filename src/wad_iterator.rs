use crate::Wad;
use crate::Entry;

pub struct WadIterator<'a> {
    index: usize,
    wad: &'a Wad,
}

impl<'a> WadIterator<'a> {
    pub(crate) fn new<'b>(wad: &'b Wad) -> WadIterator<'b> {
        WadIterator {
            index: 0,
            wad,
        }
    }
}

impl<'a> Iterator for WadIterator<'a> {
    type Item = Entry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.wad.len() {
            self.index += 1;
            return Some(self.wad.entry(self.index - 1).unwrap());
        } else {
            return None;
        }
    }
}

