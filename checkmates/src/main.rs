use std::collections::HashMap;

use common_lib::{
    calc_liberty, extract_surrounded, gather_canonical_arrangements, is_isolated, HotBitIter,
    D_WALL, L_WALL, R_WALL, U_WALL,
};
use itertools::{iproduct, Itertools};
use tokyodoves::{
    analysis::{compare_board_value, BoardValue},
    collections::BoardSet,
    game::GameRule,
    Board, BoardBuilder, Color,
};

fn make_space_if_possible(mut piece_bit: u16, mut all_bits: u16) -> (u16, u16) {
    all_bits |= piece_bit; // 防御
    if piece_bit & U_WALL != 0 && all_bits & D_WALL == 0 {
        piece_bit >>= 4;
        all_bits >>= 4;
    }
    if piece_bit & D_WALL != 0 && all_bits & U_WALL == 0 {
        piece_bit <<= 4;
        all_bits <<= 4;
    }
    if piece_bit & L_WALL != 0 && all_bits & R_WALL == 0 {
        piece_bit >>= 1;
        all_bits >>= 1;
    }
    if piece_bit & R_WALL != 0 && all_bits & L_WALL == 0 {
        piece_bit <<= 1;
        all_bits <<= 1;
    }
    (piece_bit, all_bits)
}

fn may_be_checkmate(boss_bit: u16, all_bits: u16) -> bool {
    fn _8moves(bit: u16) -> [u16; 8] {
        let u = bit << 4;
        let d = bit >> 4;
        let l = (bit & 0x7777) << 1;
        let r = (bit & 0xeeee) >> 1;
        let lu = l << 4;
        let ru = r << 4;
        let ld = l >> 4;
        let rd = r >> 4;
        [u, d, l, r, lu, ru, ld, rd]
    }

    let others = all_bits & !boss_bit;
    _8moves(boss_bit)
        .into_iter()
        .filter(|&b| b != 0 && b & others == 0 && !is_isolated(b | others)) // 合法手に限定
        .all(|b| calc_liberty(b, b | others) < 2)
}

fn is_checkmate(board: Board) -> bool {
    let rule = GameRule::new(true);
    let lose2 = BoardValue::lose(2).unwrap();
    matches!(
        compare_board_value(board, lose2, Color::Red, rule),
        Ok(std::cmp::Ordering::Equal),
    )
}

fn gather_checkmates(boss_bit: u16, mut other_bits: u16) -> BoardSet {
    other_bits &= !boss_bit; // 防御
    let mut checkmates = BoardSet::new();
    let sur = extract_surrounded(boss_bit | other_bits);
    let other_boss_candidates = other_bits & !sur;
    for other_boss_bit in HotBitIter::new(other_boss_candidates) {
        let not_boss_bits = other_bits & !other_boss_bit;
        let pos_vec: Vec<u16> = HotBitIter::new(not_boss_bits).collect();

        let pos_bosses = [[boss_bit, 0, 0, 0, 0, 0], [other_boss_bit, 0, 0, 0, 0, 0]];
        for cd_vec in iproduct!(0..2, 1..6).permutations(pos_vec.len()) {
            let mut positions = pos_bosses;
            for ((color, dove), &pos) in cd_vec.into_iter().zip(pos_vec.iter()) {
                positions[color][dove] = pos;
            }
            let board = BoardBuilder::from_u16_bits(positions).build_unchecked();
            if is_checkmate(board) {
                checkmates
                    .raw_mut()
                    .insert(board.to_invariant_u64(Color::Red));
            }
        }
    }
    checkmates
}

fn create_checkmates_map(show_progress: bool) -> HashMap<usize, BoardSet> {
    let mut checkmates_map = HashMap::new();
    for n in 2..=12 {
        checkmates_map.insert(n, BoardSet::new());
    }

    let arr_vec = gather_canonical_arrangements();
    let num_arr = arr_vec.len();
    for (i, arr) in arr_vec.into_iter().enumerate() {
        if show_progress {
            println!(
                "{} / {} ({:.2} %)",
                i + 1,
                num_arr,
                (i as f32) / (num_arr as f32) * 100f32
            );
        }
        if arr.count_ones() > 8 {
            continue;
        }

        let checkmates_set = checkmates_map
            .get_mut(&(arr.count_ones() as usize))
            .unwrap();

        let sur = extract_surrounded(arr);
        let free = arr & !sur;
        for boss in HotBitIter::new(free) {
            let (boss_bit, all_bits) = make_space_if_possible(boss, arr);
            if !may_be_checkmate(boss_bit, all_bits) {
                continue;
            }
            checkmates_set.absorb(gather_checkmates(boss_bit, all_bits & !boss_bit));
        }
    }
    checkmates_map
}

fn main() -> anyhow::Result<()> {
    let checkmates_map = create_checkmates_map(true);
    let mut total_count = 0;
    for n in 2..=12 {
        let checkmates_set = checkmates_map.get(&n).unwrap();
        println!("{}: {}", n, checkmates_set.len());
        total_count += checkmates_set.len();
    }
    println!("total: {}", total_count);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_space_if_possible() {
        let (piece_bit, all_bits) = make_space_if_possible(0b01, 0b11);
        assert_eq!(piece_bit, 0b0010_0000);
        assert_eq!(all_bits, 0b0110_0000);
    }
}
