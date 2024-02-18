use itertools::{iproduct, Itertools};
use std::collections::HashMap;

use common_lib::{
    calc_liberty, extract_surrounded, find_minimal_rectangle, gather_canonical_arrangements,
    HotBitIter,
};
use tokyodoves::{
    analysis::{compare_board_value, BoardValue},
    collections::BoardSet,
    game::GameRule,
    Board, BoardBuilder, Color,
};

fn may_be_win1(other_boss_bit: u16, mut all_bits: u16) -> bool {
    all_bits |= other_boss_bit; // 防御
    let liberty = calc_liberty(other_boss_bit, all_bits);
    if liberty == 1 {
        return true;
    }
    if liberty != 2 {
        return false;
    }
    let Some(rect) = find_minimal_rectangle(all_bits) else {
        return false;
    };
    let rect_size = rect.size();
    rect_size.vsize == 3 && rect_size.hsize == 3
}

fn is_win1(board: Board) -> bool {
    let rule = GameRule::new(true);
    let win1 = BoardValue::win(1).unwrap();
    matches!(
        compare_board_value(board, win1, Color::Red, rule),
        Ok(std::cmp::Ordering::Equal)
    )
}

fn gather_win1(other_boss_bit: u16, mut all_bits: u16) -> BoardSet {
    let mut win1_set = BoardSet::new();

    all_bits |= other_boss_bit; // 防御
    let sur = extract_surrounded(all_bits);
    let boss_candidates = all_bits & !sur & !other_boss_bit;

    for boss_bit in HotBitIter::new(boss_candidates) {
        let pos_bosses = [[boss_bit, 0, 0, 0, 0, 0], [other_boss_bit, 0, 0, 0, 0, 0]];
        let not_boss_bits = all_bits & !boss_bit & !other_boss_bit;
        let pos_vec: Vec<u16> = HotBitIter::new(not_boss_bits).collect();
        for cd_vec in iproduct!(0..2, 1..6).permutations(pos_vec.len()) {
            let mut positions = pos_bosses;
            for ((color, dove), &pos) in cd_vec.into_iter().zip(pos_vec.iter()) {
                positions[color][dove] = pos;
            }
            let board = BoardBuilder::from_u16_bits(positions).build_unchecked();
            if is_win1(board) {
                // win1_set.insert(board);
                win1_set
                    .raw_mut()
                    .insert(board.to_invariant_u64(Color::Red));
            }
        }
    }
    win1_set
}

fn count_win1() -> HashMap<usize, usize> {
    let mut total_win1_map: HashMap<usize, usize> = (2..=12).map(|n| (n, 0)).collect();

    for arr in gather_canonical_arrangements() {
        if arr.count_ones() >= 7 {
            continue;
        }
        let mut pool = BoardSet::new();
        let sur = extract_surrounded(arr);
        let free = arr & !sur;
        for other_boss_bit in HotBitIter::new(free) {
            if !may_be_win1(other_boss_bit, arr) {
                continue;
            }
            pool.absorb(gather_win1(other_boss_bit, arr));
        }

        let num_doves = arr.count_ones() as usize;
        let count = total_win1_map.get_mut(&num_doves).unwrap();
        *count += pool.len();
    }

    total_win1_map
}

fn main() {
    let count = count_win1();
    for n in 2..=12 {
        if let Some(c) = count.get(&n) {
            println!("{}: {}", n, c);
        }
    }
}
