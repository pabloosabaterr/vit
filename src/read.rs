use crate::error::VitError;

pub struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
    file: &'static str,
}

impl<'a> Reader<'a> {
    pub fn new(buf: &'a [u8], file: &'static str) -> Self {
        Self { buf, pos: 0, file }
    }

    pub fn read_bytes(&mut self, n: usize) -> crate::error::Result<&'a [u8]> {
        let end = self.pos + n;
        if end > self.buf.len() {
            return Err(VitError(format!(
                "corrupt {}: unexpected end at byte {}",
                self.file, self.pos
            )));
        }
        let slice = &self.buf[self.pos..end];
        self.pos = end;
        Ok(slice)
    }

    pub fn read_u32(&mut self) -> crate::error::Result<u32> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
    }

    pub fn read_f64(&mut self) -> crate::error::Result<f64> {
        let bytes = self.read_bytes(8)?;
        Ok(f64::from_le_bytes(bytes.try_into().unwrap()))
    }

    pub fn read_string(&mut self, len: usize) -> crate::error::Result<String> {
        let bytes = self.read_bytes(len)?;
        String::from_utf8(bytes.to_vec())
            .map_err(|_| VitError(format!("corrupt {}: invalid utf-8", self.file)))
    }

    pub(crate) fn expect_version(
        &mut self,
        expected: [u8; 4],
    ) -> crate::error::Result<()> {
        let bytes = self.read_bytes(4)?;
        if bytes[0] != expected[0] || bytes[1] != expected[1] {
            return Err(VitError(format!(
                "corrupt {}: version mismatch",
                self.file
            )));
        }
        Ok(())
    }
}

pub(crate) fn write_version(buf: &mut Vec<u8>, version: [u8; 4]) {
    buf.extend_from_slice(&version);
}
