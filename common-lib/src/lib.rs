use tokyodoves::analysis::PositionMapper;
use tokyodoves::Rectangle;

#[derive(Debug, Clone)]
pub struct HotBitIter {
    bits: u16,
}

impl HotBitIter {
    pub fn new(bits: u16) -> Self {
        Self { bits }
    }
}

impl Iterator for HotBitIter {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            return None;
        }

        let trailing_zeros = self.bits.trailing_zeros();
        let bit = 1 << trailing_zeros;
        self.bits &= !bit;
        Some(bit)
    }
}

fn find_minimal_rectangle(bits: u16) -> Option<Rectangle> {
    if bits == 0 {
        return None;
    }
    let mut hmin = 3;
    let mut hmax = 0;
    let mut vmin = 3;
    let mut vmax = 0;
    for bit in HotBitIter::new(bits) {
        let (h, v) = (
            bit.trailing_zeros() as usize % 4,
            bit.trailing_zeros() as usize / 4,
        );
        hmin = hmin.min(h);
        hmax = hmax.max(h);
        vmin = vmin.min(v);
        vmax = vmax.max(v);
    }
    Some(Rectangle {
        hmin,
        hmax,
        vmin,
        vmax,
    })
}

pub fn gather_equivalents(mut bits: u16) -> [u16; 8] {
    let mut equivalents = [0; 8];

    let Some(rect) = find_minimal_rectangle(bits) else {
        return equivalents;
    };

    bits >>= rect.vmin * 4 + rect.hmin;
    let rect_size = rect.size();
    let mapper = PositionMapper::try_create(rect_size.vsize, rect_size.hsize).unwrap();
    let bits_indices: Vec<usize> = HotBitIter::new(bits)
        .map(|bit| bit.trailing_zeros() as usize)
        .collect();

    for i in 0..8 {
        let mut mapped_bits = 0;
        for &index in &bits_indices {
            let mapped_index = mapper.map(i, index);
            mapped_bits |= 1 << mapped_index;
        }
        equivalents[i] = mapped_bits;
    }

    equivalents
}

fn canonicalize(bits: u16) -> u16 {
    gather_equivalents(bits).iter().copied().min().unwrap()
}

pub fn extract_surrounded(bits: u16) -> u16 {
    let Some(rect) = find_minimal_rectangle(bits) else {
        return 0;
    };
    let rect_size = rect.size();
    let (u_wall, d_wall) = if rect_size.vsize == 4 {
        (0xf000, 0x000f)
    } else {
        (0x0000, 0x0000)
    };
    let (l_wall, r_wall) = if rect_size.hsize == 4 {
        (0x8888, 0x1111)
    } else {
        (0x0000, 0x0000)
    };

    let u_shifted = (bits << 4) | d_wall;
    let d_shifted = (bits >> 4) | u_wall;
    let l_shifted = ((bits & 0x7777) << 1) | r_wall;
    let r_shifted = ((bits & 0xeeee) >> 1) | l_wall;
    bits & u_shifted & d_shifted & l_shifted & r_shifted
}

fn is_isolated(bits: u16) -> bool {
    let u = bits << 4;
    let d = bits >> 4;
    let u_c_d = u | bits | d;
    let l_lu_ld = (u_c_d & 0x7777) << 1;
    let r_ru_rd = (u_c_d & 0xeeee) >> 1;
    let adj = l_lu_ld | u | d | r_ru_rd;
    bits & adj != bits
}

pub fn gather_canonical_arrangements() -> Vec<u16> {
    let mut canonicals = Vec::with_capacity(44825);
    for bits in 0..=u16::MAX {
        if !(2..=12).contains(&bits.count_ones()) {
            continue;
        }
        if is_isolated(bits) {
            continue;
        }
        let canonical = canonicalize(bits);
        canonicals.push(canonical);
    }
    canonicals.sort_unstable();
    canonicals.dedup();
    canonicals
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    // fn print_bits(bits: u16) {
    //     for v in 0..4 {
    //         for h in 0..4 {
    //             let bit = 1 << (v * 4 + h);
    //             print!("{}", if bits & bit != 0 { "1" } else { "0" });
    //         }
    //         println!();
    //     }
    // }

    #[test]
    fn test_canonicalize() {
        assert_eq!(canonicalize(0), 0);
        assert_eq!(canonicalize(0b10), 0b1);
        assert_eq!(canonicalize(0b0010_0001), canonicalize(0b0001_0010));
        assert_eq!(canonicalize(0b1000_0100), canonicalize(0b0001_0010));
        assert_eq!(canonicalize(0b1000_0100_0000), canonicalize(0b0001_0010));
    }

    #[test]
    fn test_extract_surrounded() {
        assert_eq!(extract_surrounded(0), 0);
        assert_eq!(extract_surrounded(0b10), 0);
        assert_eq!(extract_surrounded(0x111f), 0x0001);
        assert_eq!(extract_surrounded(0x011f), 0x0000);
    }

    #[test]
    fn test_gather_equivalents() {
        let bits = 0x6666;
        let equivalents = gather_equivalents(bits);
        let eq_set: HashSet<u16> = equivalents.into_iter().collect();
        let expect: HashSet<u16> = [0x3333, 0xff].into_iter().collect();
        assert_eq!(eq_set, expect);
    }

    #[test]
    fn test_gather_canonical_arrangements() {
        let canonicals = gather_canonical_arrangements();
        assert_eq!(canonicals.len(), 5171);
    }
}
