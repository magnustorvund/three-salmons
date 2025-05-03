use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum NodeType {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Debug, Clone)]
pub struct TranspositionEntry {
    pub hash: u64,
    pub depth: u32,
    pub score: i32,
    pub node_type: NodeType,
    pub best_move: Option<u64>,
}

pub struct TranspositionTable {
    table: HashMap<u64, TranspositionEntry>,
    size: usize,
}

impl TranspositionTable {
    pub fn new(size: usize) -> Self {
        Self {
            table: HashMap::with_capacity(size),
            size,
        }
    }

    pub fn store(&mut self, hash: u64, entry: TranspositionEntry) {
        if self.table.len() >= self.size {
            // Remove oldest entry if table is full
            let oldest_key = *self.table.keys().next().unwrap();
            self.table.remove(&oldest_key);
        }
        self.table.insert(hash, entry);
    }

    pub fn probe(&self, hash: u64, depth: u32, alpha: i32, beta: i32) -> Option<i32> {
        if let Some(entry) = self.table.get(&hash) {
            if entry.depth >= depth {
                match entry.node_type {
                    NodeType::Exact => return Some(entry.score),
                    NodeType::LowerBound => {
                        if entry.score >= beta {
                            return Some(entry.score);
                        }
                    }
                    NodeType::UpperBound => {
                        if entry.score <= alpha {
                            return Some(entry.score);
                        }
                    }
                }
            }
        }
        None
    }

    pub fn get_best_move(&self, hash: u64) -> Option<u64> {
        self.table.get(&hash).and_then(|entry| entry.best_move)
    }
} 