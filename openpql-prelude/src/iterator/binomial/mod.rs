mod coefficient;
#[cfg(feature = "rayon")]
mod triangle;

pub(super) use coefficient::binom;
#[cfg(feature = "rayon")]
pub(super) use triangle::Triangle;
