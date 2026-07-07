use serde::{Deserialize, Serialize};

use crate::graphs::EdgeDescriptor;

pub mod algos;
pub mod contraction;

pub type Rank = usize;

pub struct ShortcutBreakdown<const N: usize = 2> {
    descriptors: [EdgeDescriptor; 2],
    num_real_edges: usize,
}

pub type Priority = i64;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PriorityParts {
    pub e: Priority,
    pub s: Priority,
    pub d: Priority,
    pub o: Priority,
    pub q: Priority,
}

impl PriorityParts {
    fn dot(&self, other: &Self) -> Priority {
        self.e * other.e + self.s * other.s + self.d * other.d + self.o * other.o + self.q * other.q
    }
}

pub struct Config {
    pub allowed_lazy_updates_to_contractions_ratio: f64,
    pub allowed_time_between_global_updates: usize,
    pub coefficients: PriorityParts,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Stats {
    num_steps: usize,
    num_contractions: usize,
    num_shortcuts: usize,
    num_lazy_updates: usize,
    num_global_updates: usize,
}

impl Stats {
    fn lazy_updates_to_contractions_ratio(&self) -> f64 {
        (self.num_lazy_updates as f64) / ((self.num_contractions + 1) as f64)
    }

    fn reset(&mut self) {
        *self = Self::default();
    }
}
