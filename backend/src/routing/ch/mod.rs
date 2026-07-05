use crate::graphs::EdgeDescriptor;

pub mod contraction;
pub mod query;

pub type Rank = usize;

type ShortcutBreakdown<const N: usize = 2> = [EdgeDescriptor; N];
