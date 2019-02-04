use crate::error::Error;

pub struct Entry<'a> {
    pub id: u64,
    pub lump: &'a [u8],
}

impl<'a> Entry<'a> {
    pub fn name(&self) -> Result<&str, Error> {
        let buf: &'a [u8; 8] = unsafe {
            // I believe this is safe because the target type does not
            // have any particular alignment requirements

            // FIXME However, this is incorrect on big-endian machines

            std::mem::transmute(&self.id)
        };
        let name = buf.split(|&x| x == 0).next().ok_or(Error::InvalidEntry)?;
        verify!(name.len() > 0, Error::InvalidEntry);
        verify!(name.iter().all(|&x| x & 0x80 == 0), Error::InvalidEntry); // ASCII

        Ok(std::str::from_utf8(name).unwrap())
    }
}
