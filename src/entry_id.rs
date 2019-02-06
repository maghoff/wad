use byteorder::{ByteOrder, NativeEndian};

#[derive(PartialEq, Eq, Hash)]
pub struct EntryId(u64);

impl EntryId {
    pub fn from_bytes(bytes: &[u8; 8]) -> Option<EntryId> {
        let is_ascii = bytes.iter().all(u8::is_ascii);
        if !is_ascii {
            return None;
        }

        Some(EntryId(NativeEndian::read_u64(bytes)))
    }

    // None if s is too long or if s contains non-ASCII characters.
    // Normalized to uppercase and zero-padded.
    pub fn from_str(s: impl AsRef<str>) -> Option<EntryId> {
        let buf = s.as_ref().as_bytes();

        if buf.len() > 8 {
            return None;
        }

        let mut padded = [0u8; 8];
        for i in 0..buf.len() {
            padded[i] = buf[i].to_ascii_uppercase();
        }

        Self::from_bytes(&padded)
    }
}

impl std::fmt::Debug for EntryId {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "EntryId({:?})", self.as_ref())
    }
}

impl std::fmt::Display for EntryId {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.as_ref(), fmt)
    }
}

impl AsRef<str> for EntryId {
    fn as_ref<'a>(&'a self) -> &'a str {
        let buf: &'a [u8; 8] = unsafe {
            // I believe this is safe because the target type does not
            // have any particular alignment requirements

            // Also, due to use of NativeEndian, this should work equally
            // well regardless of endianness.

            std::mem::transmute(&self.0)
        };
        let name = buf.split(|&x| x == 0).next().expect("split returns at least one item");

        std::str::from_utf8(name).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_bytes() {
        let id = b"E1M1\0\0\0\0";
        let entry_id = EntryId::from_bytes(&id);
        assert!(entry_id.is_some());
    }

    #[test]
    fn from_bytes_refuses_non_ascii() {
        let id = [196, 0, 0, 0, 0, 0, 0, 0];
        let entry_id = EntryId::from_bytes(&id);
        assert!(entry_id.is_none());
    }

    #[test]
    fn from_str() {
        let id = "E1M1";
        let entry_id = EntryId::from_str(&id);
        assert!(entry_id.is_some());
    }

    #[test]
    fn from_str_eq_from_bytes() {
        assert_eq!(
            EntryId::from_bytes(&b"E1M1\0\0\0\0").unwrap(),
            EntryId::from_str("E1M1").unwrap()
        );
    }

    #[test]
    fn from_str_same_is_eq() {
        assert_eq!(
            EntryId::from_str("E1M1").unwrap(),
            EntryId::from_str("E1M1").unwrap()
        );
    }

    #[test]
    fn from_str_different_is_eq() {
        assert!(
            EntryId::from_str("E1M2").unwrap() != EntryId::from_str("E1M1").unwrap()
        );
    }

    #[test]
    fn from_str_eq_to_string() {
        let id = "E1M1";
        let entry_id = EntryId::from_str(&id).unwrap();
        let string = entry_id.as_ref().to_string();
        assert_eq!(id, string);
    }

    #[test]
    fn from_str_transforms_to_uppercase() {
        assert_eq!(
            EntryId::from_str("e1m1").unwrap().as_ref().to_string(),
            "E1M1"
        );
    }

    #[test]
    fn from_str_same_is_eq_case_insensitive() {
        assert_eq!(
            EntryId::from_str("e1m1").unwrap(),
            EntryId::from_str("E1M1").unwrap()
        );
    }
}
