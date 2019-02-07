use byteorder::{ByteOrder, NativeEndian};

#[derive(PartialEq, Eq, Hash)]
pub struct EntryId(u64);

impl EntryId {
    pub fn from_bytes(bytes: &[u8; 8]) -> EntryId {
        EntryId(NativeEndian::read_u64(bytes))
    }

    /// None if s is too long or if s contains non-ASCII characters.
    /// Normalized to uppercase and zero-padded.
    pub fn from_str(s: impl AsRef<str>) -> Option<EntryId> {
        let buf = s.as_ref().as_bytes();

        if buf.len() > 8 {
            return None;
        }

        let mut padded = [0u8; 8];
        for i in 0..buf.len() {
            padded[i] = buf[i].to_ascii_uppercase();
        }

        let is_ascii = padded.iter().all(u8::is_ascii);
        if !is_ascii {
            return None;
        }

        Some(Self::from_bytes(&padded))
    }

    /// Lossy display representation. If this was created with from_bytes
    /// with a buffer containing non-ASCII characters, this function will
    /// return "?".
    pub fn display<'a>(&'a self) -> &'a str {
        let buf: &'a [u8; 8] = unsafe {
            // I believe this is safe because the target type does not
            // have any particular alignment requirements

            // Also, due to use of NativeEndian, this should work equally
            // well regardless of endianness.

            std::mem::transmute(&self.0)
        };

        let is_ascii = buf.iter().all(u8::is_ascii);
        if !is_ascii {
            return "?";
        }

        let name = buf.split(|&x| x == 0).next().expect("split returns at least one item");

        std::str::from_utf8(name).unwrap()
    }
}

impl std::fmt::Debug for EntryId {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "EntryId({:?})", self.display())
    }
}

impl std::fmt::Display for EntryId {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.display(), fmt)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_bytes_gives_good_display() {
        assert_eq!(
            EntryId::from_bytes(&b"E1M1\0\0\0\0").to_string(),
            "E1M1"
        );
    }

    #[test]
    fn from_bytes_with_non_ascii_gives_alternative_display() {
        assert_eq!(
            EntryId::from_bytes(&[196, 255, 150, 0, 0, 0, 0, 0]).to_string(),
            "?"
        );
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
            EntryId::from_bytes(&b"E1M1\0\0\0\0"),
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
    fn from_str_different_is_not_eq() {
        assert!(
            EntryId::from_str("E1M2").unwrap() != EntryId::from_str("E1M1").unwrap()
        );
    }

    #[test]
    fn from_str_eq_to_string() {
        let id = "E1M1";
        let entry_id = EntryId::from_str(&id).unwrap();
        let string = entry_id.to_string();
        assert_eq!(id, string);
    }

    #[test]
    fn from_str_transforms_to_uppercase() {
        assert_eq!(
            EntryId::from_str("e1m1").unwrap().to_string(),
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
