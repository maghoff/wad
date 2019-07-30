use byteorder::{ByteOrder, NativeEndian};

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
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

        if !padded.is_ascii() {
            return None;
        }

        Some(Self::from_bytes(&padded))
    }

    pub fn as_bytes<'a>(&'a self) -> &'a [u8; 8] {
        unsafe {
            // I believe this is safe because the target type does not
            // have any particular alignment requirements

            // Also, due to use of NativeEndian, this should work equally
            // well regardless of endianness.

            std::mem::transmute(&self.0)
        }
    }

    /// Lossy display representation. If this was created with from_bytes
    /// with a buffer containing non-ASCII characters, this function will
    /// return "?".
    pub fn display<'a>(&'a self) -> &'a str {
        let buf = self.as_bytes();

        let is_ascii = buf.iter().all(u8::is_ascii);
        if !is_ascii {
            return "?";
        }

        let name = buf
            .split(|&x| x == 0)
            .next()
            .expect("split returns at least one item");

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

impl From<&[u8; 8]> for EntryId {
    fn from(src: &[u8; 8]) -> Self {
        EntryId::from_bytes(src)
    }
}

macro_rules! widening_from_impl {
    ($n:expr) => {
        impl From<&[u8; $n]> for EntryId {
            fn from(src: &[u8; $n]) -> Self {
                let mut buf = [0; 8];
                for i in 0..$n {
                    buf[i] = src[i];
                }
                EntryId::from_bytes(&buf)
            }
        }
    };
}

widening_from_impl!(7);
widening_from_impl!(6);
widening_from_impl!(5);
widening_from_impl!(4);
widening_from_impl!(3);
widening_from_impl!(2);
widening_from_impl!(1);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_bytes_roundtrips_with_as_bytes() {
        let id = b"E1M1\0\0\0\0";
        assert_eq!(EntryId::from_bytes(id).as_bytes(), id);
    }

    #[test]
    fn from_bytes_gives_good_display() {
        assert_eq!(EntryId::from_bytes(b"E1M1\0\0\0\0").to_string(), "E1M1");
    }

    #[test]
    fn from_bytes_with_non_ascii_gives_alternative_display() {
        assert_eq!(
            EntryId::from_bytes(&[196, 255, 150, 0, 0, 0, 0, 0]).to_string(),
            "?"
        );
    }

    #[test]
    fn from_bytes_with_non_ascii_roundtrips_with_as_bytes() {
        let id = [196, 255, 150, 0, 0, 0, 0, 0];
        assert_eq!(EntryId::from_bytes(&id).as_bytes(), &id);
    }

    #[test]
    fn from_str() {
        let id = "E1M1";
        let entry_id = EntryId::from_str(&id);
        assert!(entry_id.is_some());
    }

    #[test]
    fn from_str_gives_correct_as_bytes() {
        let id = "E1M1";
        let entry_id = EntryId::from_str(&id).unwrap();
        assert_eq!(entry_id.as_bytes(), b"E1M1\0\0\0\0");
    }

    #[test]
    fn from_str_eq_from_bytes() {
        assert_eq!(
            EntryId::from_bytes(b"E1M1\0\0\0\0"),
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
        assert!(EntryId::from_str("E1M2").unwrap() != EntryId::from_str("E1M1").unwrap());
    }

    #[test]
    fn from_str_eq_to_string() {
        let id = "E1M1";
        let entry_id = EntryId::from_str(&id).unwrap();
        let string = entry_id.to_string();
        assert_eq!(id, string);
    }

    #[test]
    fn from_str_transforms_to_uppercase_display() {
        assert_eq!(EntryId::from_str("e1m1").unwrap().to_string(), "E1M1");
    }

    #[test]
    fn from_str_transforms_to_uppercase_as_bytes() {
        assert_eq!(
            EntryId::from_str("e1m1").unwrap().as_bytes(),
            b"E1M1\0\0\0\0"
        );
    }

    #[test]
    fn from_str_same_is_eq_case_insensitive() {
        assert_eq!(
            EntryId::from_str("e1m1").unwrap(),
            EntryId::from_str("E1M1").unwrap()
        );
    }

    #[test]
    fn from_array8_impl() {
        assert_eq!(EntryId::from_bytes(b"E1M1\0\0\0\0"), b"E1M1\0\0\0\0".into());
    }

    #[test]
    fn from_array7_impl() {
        assert_eq!(EntryId::from_bytes(b"E1M1\0\0\0\0"), b"E1M1\0\0\0".into());
    }

    #[test]
    fn from_array6_impl() {
        assert_eq!(EntryId::from_bytes(b"E1M1\0\0\0\0"), b"E1M1\0\0".into());
    }

    #[test]
    fn from_array5_impl() {
        assert_eq!(EntryId::from_bytes(b"E1M1\0\0\0\0"), b"E1M1\0".into());
    }

    #[test]
    fn from_array1_impl() {
        assert_eq!(EntryId::from_bytes(b"E\0\0\0\0\0\0\0"), b"E".into());
    }
}
