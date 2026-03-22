use crate::format::FrequencyTable;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

pub enum HuffmanTree {
    Leaf {
        symbol: u8,
        frequency: u64,
    },
    Internal {
        frequency: u64,
        min_symbol: u8,
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

    pub fn min_symbol(&self) -> u8 {
        match self {
            Self::Leaf { symbol, .. } => *symbol,
            Self::Internal { min_symbol, .. } => *min_symbol,
        }
    }

    pub fn new_leaf(symbol: u8, frequency: u64) -> Self {
        Self::Leaf { symbol, frequency }
    }

    pub fn new_internal(left: Box<HuffmanTree>, right: Box<HuffmanTree>) -> Self {
        Self::Internal {
            frequency: left.frequency() + right.frequency(),
            min_symbol: left.min_symbol().min(right.min_symbol()),
            left,
            right,
        }
    }
}

impl PartialEq<Self> for HuffmanTree {
    fn eq(&self, other: &Self) -> bool {
        self.frequency() == other.frequency()
    }
}

impl PartialOrd for HuffmanTree {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let mut ord = self.frequency().cmp(&other.frequency());
        if ord == Ordering::Equal {
            ord = self.min_symbol().cmp(&other.min_symbol());
        }

        Some(ord.reverse())
    }
}

impl Eq for HuffmanTree {}

impl Ord for HuffmanTree {
    fn cmp(&self, other: &Self) -> Ordering {
        Self::partial_cmp(self, other).unwrap()
    }
}

impl TryFrom<&FrequencyTable> for HuffmanTree {
    type Error = anyhow::Error;

    fn try_from(value: &FrequencyTable) -> Result<Self, Self::Error> {
        if value.count == 0 || value.entries.is_empty() {
            anyhow::bail!("频次表不能为空")
        }

        let mut nodes = BinaryHeap::new();
        let iter = value
            .entries
            .iter()
            .map(|entry| Self::new_leaf(entry.symbol, entry.frequency));
        nodes.extend(iter);

        while nodes.len() > 1 {
            let left = nodes.pop().unwrap();
            let right = nodes.pop().unwrap();
            nodes.push(Self::new_internal(Box::new(left), Box::new(right)));
        }

        Ok(nodes.pop().unwrap())
    }
}
