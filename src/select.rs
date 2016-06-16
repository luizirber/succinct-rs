//! Data structure to support fast select queries.

use rank::Rank;
use storage::BitStore;

/// Interface for types that support select queries.
pub trait Select : BitStore {
    /// Returns the position of the `index`th 1 bit.
    fn select(&self, index: u64) -> Option<u64>;
}

/// Performs a select query by binary searching rank queries.
pub struct BinSearchSelect<'a, R: Rank + 'a> {
    rank_support: &'a R,
    max_rank: u64,
}

/// Creates a new binary search select support based on a rank support.
impl<'a, R: Rank + 'a> BinSearchSelect<'a, R> {
    /// Creates a new binary search selection support given a rank
    /// support.
    pub fn new(rank_support: &'a R) -> Self {
        let max_index = rank_support.bit_len() - 1;
        let max_rank = rank_support.rank(max_index);
        BinSearchSelect {
            rank_support: rank_support,
            max_rank: max_rank,
        }
    }
}

impl<'a, R: Rank + 'a> BitStore for BinSearchSelect<'a, R> {
    type Block = R::Block;

    fn block_len(&self) -> usize {
        self.rank_support.block_len()
    }

    fn bit_len(&self) -> u64 {
        self.rank_support.bit_len()
    }

    fn get_block(&self, index: usize) -> Self::Block {
        self.rank_support.get_block(index)
    }

    fn get_bit(&self, index: u64) -> bool {
        self.rank_support.get_bit(index)
    }
}

impl<'a, R: Rank + 'a> Rank for BinSearchSelect<'a, R> {
    fn rank(&self, index: u64) -> u64 {
        self.rank_support.rank(index)
    }
}

impl<'a, R: Rank + 'a> Select for BinSearchSelect<'a, R> {
    fn select(&self, index: u64) -> Option<u64> {
        // To find the `index`th 1, we find the position where
        // the rank goes to `index + 1`.
        let rank = index + 1;

        if rank > self.max_rank { return None; }

        let mut start = 0;
        let mut limit = self.bit_len();

        // Search in [start, limit):
        while start < limit {
            // Calculate average without risking overflow:
            let mid = start/2 + limit/2 + (start % 2 + limit % 2)/2;
            debug_assert!(start <= mid && mid < limit);

            let mid_rank = self.rank(mid);
            let pre_mid_rank = if mid == 0 {0} else {self.rank(mid - 1)};

            if mid_rank == rank && pre_mid_rank == rank - 1 {
                return Some(mid)
            } else if pre_mid_rank > rank {
                limit = mid - 1;
            } else if pre_mid_rank == rank {
                limit = mid;
            } else if mid_rank < rank {
                start = mid + 1;
            }
        }

        panic!("BinSearchSelect: broken invariant in rank support?");
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rank::JacobsonRank;
    use rank::Rank;

    #[test]
    fn select1() {
        let vec = vec![ 0b10000000000000001110000000000000u32; 1024 ];
        let rank = JacobsonRank::new(&*vec);
        let select = BinSearchSelect::new(&rank);

        assert_eq!(1, select.rank(0));
        assert_eq!(1, select.rank(1));
        assert_eq!(1, select.rank(2));
        assert_eq!(1, select.rank(15));
        assert_eq!(2, select.rank(16));
        assert_eq!(3, select.rank(17));
        assert_eq!(4, select.rank(18));
        assert_eq!(4, select.rank(19));
        assert_eq!(4, select.rank(20));
        assert_eq!(5, select.rank(32));

        assert_eq!(Some(0), select.select(0));
        assert_eq!(Some(16), select.select(1));
        assert_eq!(Some(17), select.select(2));
        assert_eq!(Some(18), select.select(3));
        assert_eq!(Some(32), select.select(4));
        assert_eq!(Some(3200), select.select(400));
        assert_eq!(Some(3216), select.select(401));

        assert_eq!(Some(8 * 4092), select.select(4092));
        assert_eq!(Some(8 * 4092 + 16), select.select(4093));
        assert_eq!(Some(8 * 4092 + 17), select.select(4094));
        assert_eq!(Some(8 * 4092 + 18), select.select(4095));
        assert_eq!(None, select.select(4096))
    }

    #[test]
    fn select2() {
        let vec = vec![ 0b01010101010101010101010101010101u32; 1024 ];
        let rank = JacobsonRank::new(&*vec);
        let select = BinSearchSelect::new(&rank);

        assert_eq!(Some(1), select.select(0));
        assert_eq!(Some(3), select.select(1));
        assert_eq!(Some(5), select.select(2));
        assert_eq!(Some(7), select.select(3));
        assert_eq!(Some(919), select.select(459));
    }

    #[test]
    fn select3() {
        let vec = vec![ 0b11111111111111111111111111111111u32; 1024 ];
        let rank = JacobsonRank::new(&*vec);
        let select = BinSearchSelect::new(&rank);

        assert_eq!(Some(0), select.select(0));
        assert_eq!(Some(1), select.select(1));
        assert_eq!(Some(2), select.select(2));
        assert_eq!(Some(32767), select.select(32767));
        assert_eq!(None, select.select(32768));
    }

}