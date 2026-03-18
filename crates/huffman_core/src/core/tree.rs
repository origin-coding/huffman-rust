pub enum HuffmanTree {
    Leaf {
        symbol: u8,
        frequency: u64,
    },
    Internal {
        frequency: u64,
        left: Box<HuffmanTree>,
        right: Box<HuffmanTree>,
    },
}

impl HuffmanTree {
    pub fn frequency(&self) -> u64 {
        match self {
            Self::Leaf { frequency, .. } => *frequency,
            Self::Internal { frequency, .. } => *frequency,
        }
    }

    pub fn new_leaf(symbol: u8, frequency: u64) -> Self {
        Self::Leaf { symbol, frequency }
    }

    pub fn new_internal(left: Box<HuffmanTree>, right: Box<HuffmanTree>) -> Self {
        Self::Internal {
            frequency: left.frequency() + right.frequency(),
            left,
            right,
        }
    }
}
