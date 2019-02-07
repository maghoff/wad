use crate::entry_id::EntryId;

pub struct Entry<'a> {
    pub id: EntryId,
    pub lump: &'a [u8],
}

impl<'a> Entry<'a> {
    /// Lossy display representation. If the ID contains non-ASCII characters,
    /// this function will return "?".
    pub fn display_name(&self) -> &str {
        self.id.display()
    }
}
