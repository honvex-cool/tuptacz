pub mod contraction;
pub mod query;

pub type Rank = usize;

struct EdgeBreakdown {
    index_within_start: usize,
    index_within_middle: usize,
}
