use core::{fmt, str};
use std::mem;

use crate::{Card64, CardCount, Rank, Rank16, Suit};

mod indexer;
mod mixed_radix;
mod per_suit_ranks;
mod rank_count;
mod ranks;
mod round_bits;
mod shifted_ranks;
mod suit_config;
mod suit_multiset;
mod util;

pub use indexer::*;
pub use mixed_radix::*;
pub use per_suit_ranks::*;
pub use rank_count::*;
pub use ranks::*;
pub use round_bits::*;
pub use shifted_ranks::*;
pub use suit_config::*;
pub use suit_multiset::*;
use util::{colex_decode, colex_encode, colex_multi_decode, colex_multi_encode};

type RoundBitsInner = u16;
type RoundIndex = u16;
pub type WaughIndex = u64;
