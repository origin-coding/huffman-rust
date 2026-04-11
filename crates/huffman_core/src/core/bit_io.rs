use std::io::Write;
use crate::core::error::Result;
use bitvec::prelude::*;

/// 支持位级写入的包装器
#[must_use = "BitWriter must be explicitly closed via finalize() to handle trailing bits and confirm IO errors."]
pub struct BitWriter<W: Write> {
    inner: W,
    buffer: u8,
    used: u8,
}

impl<W: Write> BitWriter<W> {
    /// 创建一个新的 BitWriter
    pub fn new(inner: W) -> Self {
        Self {
            inner,
            buffer: 0,
            used: 0,
        }
    }

    /// 写入单个比特位
    /// 采用 MSB-first (最高位优先) 策略
    pub fn write_bit(&mut self, bit: bool) -> Result<()> {
        if bit {
            // 将对应位置为 1
            // 第一次写入 (used=0) 对应 1 << 7 (0x80)
            self.buffer |= 1 << (7 - self.used);
        }
        self.used += 1;

        // 如果凑够一个字节，执行写入
        if self.used == 8 {
            self.inner.write_all(&[self.buffer])?;
            self.buffer = 0;
            self.used = 0;
        }
        Ok(())
    }

    /// 批量写入比特切片
    pub fn write_bits(&mut self, bits: &BitSlice<u8, Msb0>) -> Result<()> {
        for bit in bits {
            self.write_bit(*bit)?;
        }
        Ok(())
    }

    /// 完成写入，将剩余不足一个字节的数据刷入底层写入器
    /// 返回填充的比特数 (0-7)
    /// 该方法会消耗 BitWriter
    pub fn finalize(mut self) -> Result<u8> {
        if self.used == 0 {
            return Ok(0);
        }

        let padding = 8 - self.used;
        self.inner.write_all(&[self.buffer])?;
        Ok(padding)
    }

    /// 获取底层的写入器
    pub fn into_inner(self) -> W {
        self.inner
    }
}
