use {crate::error::Error, std::io::BufRead};

/// Helper extension trait to give a convenient way to iterate over
/// chunks sized to the size of the internal buffer of the reader.
pub(crate) trait ForEachBufFill: BufRead {
    /// Takes a function to apply to each buffer fill, and returns if any
    /// errors arose along with the number of bytes read in total.
    fn for_each_buf_fill(&mut self, f: impl FnMut(&[u8])) -> Result<usize, Error>;
}

impl<R: BufRead> ForEachBufFill for R {
    fn for_each_buf_fill(&mut self, mut f: impl FnMut(&[u8])) -> Result<usize, Error> {
        let mut total_read = 0;

        loop {
            let buffer = self.fill_buf()?;
            let amount_read = buffer.len();

            if amount_read == 0 {
                break;
            }

            f(buffer);

            self.consume(amount_read);
            total_read += amount_read;
        }

        Ok(total_read)
    }
}
