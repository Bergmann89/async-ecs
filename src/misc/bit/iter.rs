use hibitset::BitSetLike;

#[derive(Debug, Clone)]
pub struct BitIter<T> {
    set: T,
    masks: [usize; LAYERS],
    prefix: [u32; LAYERS - 1],
}

impl<T> BitIter<T>
where
    T: BitSetLike,
{
    pub fn new(set: T) -> Self {
        Self {
            masks: [0, 0, 0, set.layer3()],
            prefix: [0; 3],
            set,
        }
    }

    fn handle_next(&mut self, level: usize) -> State {
        use self::State::*;

        if self.masks[level] == 0 {
            Empty
        } else {
            let first_bit = self.masks[level].trailing_zeros();
            self.masks[level] &= !(1 << first_bit);

            let idx = self.prefix.get(level).cloned().unwrap_or(0) | first_bit;

            if level == 0 {
                Value(idx)
            } else {
                self.masks[level - 1] = self.set.get_from_layer(level - 1, idx as usize);
                self.prefix[level - 1] = idx << BITS;

                Continue
            }
        }
    }
}

impl<T> BitIter<T>
where
    T: BitSetLike + Copy,
{
    pub fn split(mut self) -> (Self, Option<Self>) {
        let other = self
            .handle_split(3)
            .or_else(|| self.handle_split(2))
            .or_else(|| self.handle_split(1));

        (self, other)
    }

    fn handle_split(&mut self, level: usize) -> Option<Self> {
        if self.masks[level] == 0 {
            None
        } else {
            let level_prefix = self.prefix.get(level).cloned().unwrap_or(0);
            let first_bit = self.masks[level].trailing_zeros();

            bit_average(self.masks[level])
                .map(|average_bit| {
                    let mask = (1 << average_bit) - 1;
                    let mut other = BitIter {
                        set: self.set,
                        masks: [0; LAYERS],
                        prefix: [0; LAYERS - 1],
                    };

                    other.masks[level] = self.masks[level] & !mask;
                    other.prefix[level - 1] = (level_prefix | average_bit as u32) << BITS;
                    other.prefix[level..].copy_from_slice(&self.prefix[level..]);

                    self.masks[level] &= mask;
                    self.prefix[level - 1] = (level_prefix | first_bit) << BITS;

                    other
                })
                .or_else(|| {
                    let idx = level_prefix as usize | first_bit as usize;

                    self.prefix[level - 1] = (idx as u32) << BITS;
                    self.masks[level] = 0;
                    self.masks[level - 1] = self.set.get_from_layer(level - 1, idx);

                    None
                })
        }
    }
}

impl<T: BitSetLike> BitIter<T> {
    pub fn contains(&self, i: u32) -> bool {
        self.set.contains(i)
    }
}

#[derive(PartialEq)]
pub(crate) enum State {
    Empty,
    Continue,
    Value(u32),
}

impl<T> Iterator for BitIter<T>
where
    T: BitSetLike,
{
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        use self::State::*;

        'find: loop {
            for level in 0..LAYERS {
                match self.handle_next(level) {
                    Value(v) => return Some(v),
                    Continue => continue 'find,
                    Empty => {}
                }
            }

            return None;
        }
    }
}

impl<T: BitSetLike> BitIter<T> {}

pub fn bit_average(n: usize) -> Option<usize> {
    #[cfg(target_pointer_width = "64")]
    let average = bit_average_u64(n as u64).map(|n| n as usize);

    #[cfg(target_pointer_width = "32")]
    let average = bit_average_u32(n as u32).map(|n| n as usize);

    average
}

#[allow(clippy::many_single_char_names)]
#[cfg(any(test, target_pointer_width = "32"))]
fn bit_average_u32(n: u32) -> Option<u32> {
    const PAR: [u32; 5] = [!0 / 0x3, !0 / 0x5, !0 / 0x11, !0 / 0x101, !0 / 0x10001];

    let a = n - ((n >> 1) & PAR[0]);
    let b = (a & PAR[1]) + ((a >> 2) & PAR[1]);
    let c = (b + (b >> 4)) & PAR[2];
    let d = (c + (c >> 8)) & PAR[3];

    let mut cur = d >> 16;
    let count = (d + cur) & PAR[4];

    if count <= 1 {
        return None;
    }

    let mut target = count / 2;
    let mut result = 32;

    {
        let mut descend = |child, child_stride, child_mask| {
            if cur < target {
                result -= 2 * child_stride;
                target -= cur;
            }

            cur = (child >> (result - child_stride)) & child_mask;
        };

        descend(c, 8, 16 - 1); // PAR[3]
        descend(b, 4, 8 - 1); // PAR[2]
        descend(a, 2, 4 - 1); // PAR[1]
        descend(n, 1, 2 - 1); // PAR[0]
    }

    if cur < target {
        result -= 1;
    }

    Some(result - 1)
}

#[allow(clippy::many_single_char_names)]
#[cfg(any(test, target_pointer_width = "64"))]
fn bit_average_u64(n: u64) -> Option<u64> {
    const PAR: [u64; 6] = [
        !0 / 0x3,
        !0 / 0x5,
        !0 / 0x11,
        !0 / 0x101,
        !0 / 0x10001,
        !0 / 0x100000001,
    ];

    let a = n - ((n >> 1) & PAR[0]);
    let b = (a & PAR[1]) + ((a >> 2) & PAR[1]);
    let c = (b + (b >> 4)) & PAR[2];
    let d = (c + (c >> 8)) & PAR[3];
    let e = (d + (d >> 16)) & PAR[4];

    let mut cur = e >> 32;
    let count = (e + cur) & PAR[5];

    if count <= 1 {
        return None;
    }

    let mut target = count / 2;
    let mut result = 64;

    {
        let mut descend = |child, child_stride, child_mask| {
            if cur < target {
                result -= 2 * child_stride;
                target -= cur;
            }

            cur = (child >> (result - child_stride)) & child_mask;
        };

        descend(d, 16, 256 - 1); // PAR[4]
        descend(c, 8, 16 - 1); // PAR[3]
        descend(b, 4, 8 - 1); // PAR[2]
        descend(a, 2, 4 - 1); // PAR[1]
        descend(n, 1, 2 - 1); // PAR[0]
    }

    if cur < target {
        result -= 1;
    }

    Some(result - 1)
}

const LAYERS: usize = 4;

#[cfg(target_pointer_width = "64")]
pub const BITS: usize = 6;

#[cfg(target_pointer_width = "32")]
pub const BITS: usize = 5;

#[cfg(test)]
mod test_bit_average {
    use hibitset::{BitSet, BitSetLike};

    use super::*;

    #[test]
    fn iterator_clone() {
        let mut set = BitSet::new();

        set.add(1);
        set.add(3);

        let iter = set.iter().skip(1);
        for (a, b) in iter.clone().zip(iter) {
            assert_eq!(a, b);
        }
    }

    #[test]
    fn parity_0_bit_average_u32() {
        struct EvenParity(u32);

        impl Iterator for EvenParity {
            type Item = u32;
            fn next(&mut self) -> Option<Self::Item> {
                if self.0 == u32::max_value() {
                    return None;
                }
                self.0 += 1;
                while self.0.count_ones() & 1 != 0 {
                    if self.0 == u32::max_value() {
                        return None;
                    }
                    self.0 += 1;
                }
                Some(self.0)
            }
        }

        let steps = 1000;
        for i in 0..steps {
            let pos = i * (u32::max_value() / steps);
            for i in EvenParity(pos).take(steps as usize) {
                let mask = (1 << bit_average_u32(i).unwrap_or(31)) - 1;
                assert_eq!((i & mask).count_ones(), (i & !mask).count_ones(), "{:x}", i);
            }
        }
    }

    #[test]
    fn parity_1_bit_average_u32() {
        struct OddParity(u32);

        impl Iterator for OddParity {
            type Item = u32;
            fn next(&mut self) -> Option<Self::Item> {
                if self.0 == u32::max_value() {
                    return None;
                }
                self.0 += 1;
                while self.0.count_ones() & 1 == 0 {
                    if self.0 == u32::max_value() {
                        return None;
                    }
                    self.0 += 1;
                }
                Some(self.0)
            }
        }

        let steps = 1000;
        for i in 0..steps {
            let pos = i * (u32::max_value() / steps);
            for i in OddParity(pos).take(steps as usize) {
                let mask = (1 << bit_average_u32(i).unwrap_or(31)) - 1;
                let a = (i & mask).count_ones();
                let b = (i & !mask).count_ones();
                if a < b {
                    assert_eq!(a + 1, b, "{:x}", i);
                } else if b < a {
                    assert_eq!(a, b + 1, "{:x}", i);
                } else {
                    panic!("Odd parity shouldn't split in exactly half");
                }
            }
        }
    }

    #[test]
    fn empty_bit_average_u32() {
        assert_eq!(None, bit_average_u32(0));
    }

    #[test]
    fn singleton_bit_average_u32() {
        for i in 0..32 {
            assert_eq!(None, bit_average_u32(1 << i), "{:x}", i);
        }
    }

    #[test]
    fn parity_0_bit_average_u64() {
        struct EvenParity(u64);

        impl Iterator for EvenParity {
            type Item = u64;
            fn next(&mut self) -> Option<Self::Item> {
                if self.0 == u64::max_value() {
                    return None;
                }
                self.0 += 1;
                while self.0.count_ones() & 1 != 0 {
                    if self.0 == u64::max_value() {
                        return None;
                    }
                    self.0 += 1;
                }
                Some(self.0)
            }
        }

        let steps = 1000;
        for i in 0..steps {
            let pos = i * (u64::max_value() / steps);
            for i in EvenParity(pos).take(steps as usize) {
                let mask = (1 << bit_average_u64(i).unwrap_or(63)) - 1;
                assert_eq!((i & mask).count_ones(), (i & !mask).count_ones(), "{:x}", i);
            }
        }
    }

    #[test]
    fn parity_1_bit_average_u64() {
        struct OddParity(u64);

        impl Iterator for OddParity {
            type Item = u64;
            fn next(&mut self) -> Option<Self::Item> {
                if self.0 == u64::max_value() {
                    return None;
                }
                self.0 += 1;
                while self.0.count_ones() & 1 == 0 {
                    if self.0 == u64::max_value() {
                        return None;
                    }
                    self.0 += 1;
                }
                Some(self.0)
            }
        }

        let steps = 1000;
        for i in 0..steps {
            let pos = i * (u64::max_value() / steps);
            for i in OddParity(pos).take(steps as usize) {
                let mask = (1 << bit_average_u64(i).unwrap_or(63)) - 1;
                let a = (i & mask).count_ones();
                let b = (i & !mask).count_ones();
                if a < b {
                    assert_eq!(a + 1, b, "{:x}", i);
                } else if b < a {
                    assert_eq!(a, b + 1, "{:x}", i);
                } else {
                    panic!("Odd parity shouldn't split in exactly half");
                }
            }
        }
    }

    #[test]
    fn empty_bit_average_u64() {
        assert_eq!(None, bit_average_u64(0));
    }

    #[test]
    fn singleton_bit_average_u64() {
        for i in 0..64 {
            assert_eq!(None, bit_average_u64(1 << i), "{:x}", i);
        }
    }

    #[test]
    fn bit_average_agree_u32_u64() {
        let steps = 1000;
        for i in 0..steps {
            let pos = i * (u32::max_value() / steps);
            for i in pos..steps {
                assert_eq!(
                    bit_average_u32(i),
                    bit_average_u64(i as u64).map(|n| n as u32),
                    "{:x}",
                    i
                );
            }
        }
    }

    #[test]
    fn specific_values() {
        assert_eq!(Some(4), bit_average_u32(0b10110));
        assert_eq!(Some(5), bit_average_u32(0b100010));
        assert_eq!(None, bit_average_u32(0));
        assert_eq!(None, bit_average_u32(1));

        assert_eq!(Some(4), bit_average_u64(0b10110));
        assert_eq!(Some(5), bit_average_u64(0b100010));
        assert_eq!(None, bit_average_u64(0));
        assert_eq!(None, bit_average_u64(1));
    }
}
