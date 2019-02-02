use crate::error::Error;

pub struct Lump<'a> {
    pub id: u64,
    pub data: &'a [u8],
}

impl<'a> Lump<'a> {
    pub fn name(&self) -> Result<&str, Error> {
        let buf: &'a [u8; 8] = unsafe { std::mem::transmute(&self.id) };
        let name = buf.split(|&x| x == 0).next().ok_or(Error::InvalidEntry)?;
        verify!(name.len() > 0, Error::InvalidEntry);
        verify!(name.iter().all(|&x| x & 0x80 == 0), Error::InvalidEntry); // ASCII

        Ok(std::str::from_utf8(name).unwrap())
    }
}
