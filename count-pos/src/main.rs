use std::collections::{HashMap, HashSet};

use common_lib::{extract_surrounded, gather_canonical_arrangements, gather_equivalents};

fn count_equivalents(bits: u16) -> usize {
    gather_equivalents(bits)
        .into_iter()
        .collect::<HashSet<u16>>()
        .len()
}

fn is_on_line(bits: u16) -> bool {
    let h = 0x000f;
    for i in 0..4 {
        let mask = h << (i * 4);
        if (bits & mask) == bits {
            return true;
        }
    }

    let v = 0x1111;
    for i in 0..4 {
        let mask = v << i;
        if (bits & mask) == bits {
            return true;
        }
    }

    let d1 = 0x1248;
    let d2 = 0x8421;
    for i in 0..4 {
        for d in [d1, d2] {
            let mask = d << (i * 4);
            if (bits & mask) == bits {
                return true;
            }
            let mask = d >> (i * 4);
            if (bits & mask) == bits {
                return true;
            }
        }
    }
    false
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
    let mut total_invariants = HashMap::new();
    for n in 2..=12 {
        total_invariants.insert(n, 0);
    }

    for arr in gather_canonical_arrangements() {
        let num_doves = arr.count_ones() as usize;
        let num_surrounded = extract_surrounded(arr).count_ones() as usize;
        let num_not_surrounded = num_doves - num_surrounded;
        // let invariant = perm(num_doves, 2) * perm(10, num_doves - 2);
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
    let count = count_positions(false, false, true);
    let mut total = 0;
    for n in 2..=12 {
        let each_count = *count.get(&n).unwrap();
        println!("{n}: {each_count}");
        total += each_count;
    }
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
