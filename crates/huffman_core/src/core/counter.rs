use crate::core::error::Result;
use crate::format::{FrequencyEntry, FrequencyTable};
use std::io::{ErrorKind, Read};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ByteCounter {
    counts: [u64; 256],
}

impl ByteCounter {
    pub fn new() -> Self {
        Self { counts: [0; 256] }
    }

    /// 从指定的读取器中读取并统计字节频率
    pub fn from_reader(mut reader: impl Read) -> Result<Self> {
        let mut counter = Self::new();
        counter.count(&mut reader)?;
        Ok(counter)
    }

    /// 增量统计给定读取器中的字节频率
    pub fn count(&mut self, reader: &mut impl Read) -> Result<()> {
        let mut buffer = [0u8; 8192];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    for &b in &buffer[..n] {
                        self.counts[b as usize] += 1;
                    }
                }
                Err(e) if e.kind() == ErrorKind::Interrupted => {
                    continue;
                }
                Err(e) => return Err(e.into()),
            }
        }
        Ok(())
    }

    pub fn counts(&self) -> [u64; 256] {
        self.counts
    }
}

impl Default for ByteCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&ByteCounter> for FrequencyTable {
    fn from(counter: &ByteCounter) -> Self {
        let entries: Vec<FrequencyEntry> = counter
            .counts
            .iter()
            .enumerate()
            .filter(|&(_, &freq)| freq > 0)
            .map(|(symbol, &frequency)| FrequencyEntry {
                symbol: symbol as u8,
                frequency,
            })
            .collect();

        FrequencyTable {
            count: entries.len() as u16,
            entries,
        }
    }
}
