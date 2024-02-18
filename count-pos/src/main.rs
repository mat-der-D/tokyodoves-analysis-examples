use std::collections::{HashMap, HashSet};

use common_lib::{extract_surrounded, gather_canonical_arrangements, gather_equivalents};

fn count_equivalents(bits: u16) -> usize {
    gather_equivalents(bits)
        .into_iter()
        .collect::<HashSet<u16>>()
        .len()
}

fn is_on_line(bits: u16) -> bool {
    fn _shift(x: u16, n: i16) -> u16 {
        if n > 0 {
            x << (n * 4)
        } else {
            x >> (-n * 4)
        }
    }

    let hs = (0..4).map(|i| 0x000f << (i * 4)); // 横一列
    let vs = (0..4).map(|i| 0x1111 << i); // 縦一列
    let ds1 = (-3..=3).map(|i| _shift(0x1248, i)); // ／ の一列
    let ds2 = (-3..=3).map(|i| _shift(0x8421, i)); // ＼ の一列
    hs.chain(vs)
        .chain(ds1)
        .chain(ds2)
        .any(|mask| bits & mask == bits)
}

fn perm(n: usize, r: usize) -> usize {
    let mut result = 1;
    for i in 0..r {
        result *= n - i;
    }
    result
}

fn count_positions(
    include_free: bool,
    include_one_surrounded: bool,
    include_both_surrounded: bool,
) -> HashMap<usize, usize> {
    let mut total_invariants: HashMap<usize, usize> = (2..=12).map(|n| (n, 0)).collect();

    for arr in gather_canonical_arrangements() {
        let num_doves = arr.count_ones() as usize;
        let num_surrounded = extract_surrounded(arr).count_ones() as usize;
        let num_not_surrounded = num_doves - num_surrounded;

        let mut invariant = 0;
        if include_free && num_not_surrounded >= 2 {
            invariant += perm(num_not_surrounded, 2) * perm(10, num_doves - 2);
        }
        if include_one_surrounded && num_surrounded >= 1 && num_not_surrounded >= 1 {
            invariant +=
                2 * perm(num_surrounded, 1) * perm(num_not_surrounded, 1) * perm(10, num_doves - 2);
        }
        if include_both_surrounded && num_surrounded >= 2 {
            invariant += perm(num_surrounded, 2) * perm(10, num_doves - 2);
        }

        *total_invariants.get_mut(&num_doves).unwrap() += if is_on_line(arr) {
            invariant * count_equivalents(arr) * 2
        } else {
            invariant * count_equivalents(arr)
        };
    }

    for n in 2..=12 {
        *total_invariants.get_mut(&n).unwrap() /= 8;
    }
    total_invariants
}

fn main() {
    let count = count_positions(
        true, // いずれのボスハトも囲まれていないケース
        true, // 一方のボスハトのみが囲まれているケース
        true, // 両方のボスハトが囲まれているケース
    );
    let mut total = 0;
    for n in 2..=12 {
        let each_count = *count.get(&n).unwrap();
        println!("{n:>2}: {each_count}");
        total += each_count;
    }
    println!("---------------");
    println!("Total: {total}");
}

// test
#[cfg(test)]
mod tests {
    use super::*;
    use common_lib::gather_canonical_arrangements;

    #[test]
    fn test_count_positions() {
        let count = count_positions(true, true, true);
        assert_eq!(count.get(&2).unwrap(), &2);
        assert_eq!(count.get(&3).unwrap(), &180);
        assert_eq!(count.get(&4).unwrap(), &28620);
    }

    #[test]
    fn test_gather_canonical_arrangements() {
        let canonicals = gather_canonical_arrangements();
        assert_eq!(canonicals.len(), 5171);
    }
}
