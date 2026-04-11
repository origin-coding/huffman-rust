use crate::core::error::CoreError;
use crate::format::FrequencyTable;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};

#[derive(Debug)]
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

    pub fn new_internal(
        left: Box<HuffmanTree>,
        right: Box<HuffmanTree>,
    ) -> Result<Self, CoreError> {
        let left_frequency = left.frequency();
        let right_frequency = right.frequency();
        let merged_frequency =
            left_frequency
                .checked_add(right_frequency)
                .ok_or(CoreError::FrequencyOverflow {
                    left: left_frequency,
                    right: right_frequency,
                })?;

        Ok(Self::Internal {
            frequency: merged_frequency,
            min_symbol: left.min_symbol().min(right.min_symbol()),
            left,
            right,
        })
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
    type Error = CoreError;

    fn try_from(value: &FrequencyTable) -> Result<Self, Self::Error> {
        if value.count == 0 || value.entries.is_empty() {
            return Err(CoreError::EmptyFrequencyTable);
        }

        let actual_count = value.entries.len();
        if actual_count != usize::from(value.count) {
            return Err(CoreError::FrequencyCountMismatch {
                declared: value.count,
                actual: actual_count,
            });
        }

        let mut seen_symbols = HashSet::with_capacity(actual_count);
        for entry in &value.entries {
            if entry.frequency == 0 {
                return Err(CoreError::ZeroFrequency {
                    symbol: entry.symbol,
                });
            }

            if !seen_symbols.insert(entry.symbol) {
                return Err(CoreError::DuplicateSymbol {
                    symbol: entry.symbol,
                });
            }
        }

        let mut nodes = BinaryHeap::new();
        let iter = value
            .entries
            .iter()
            .map(|entry| Self::new_leaf(entry.symbol, entry.frequency));
        nodes.extend(iter);

        while nodes.len() > 1 {
            let left = nodes.pop().expect("nodes.len() > 1");
            let right = nodes.pop().expect("nodes.len() > 1");
            nodes.push(Self::new_internal(Box::new(left), Box::new(right))?);
        }

        nodes.pop().ok_or(CoreError::EmptyFrequencyTable)
    }
}
