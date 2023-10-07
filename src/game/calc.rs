use super::model::{Board, Coord, Outcome, Piece, CELLS};
use std::{collections::HashMap, ops::BitOr};

impl Into<u32> for Board {
    fn into(self) -> u32 {
        self.0
            .iter()
            .enumerate()
            .map(|(i, &piece)| Into::<u32>::into(piece) << 2 * i)
            .reduce(u32::bitor)
            .unwrap()
    }
}
impl Coord {
    fn right_rotate(&self, times: usize) -> Self {
        let result = (0..times).fold(self.flat_ind(), |acc, _| Self::right_rotate_cell(acc));
        Self::from_flat_ind(result)
    }
    fn from_flat_ind(ind: usize) -> Self {
        Self(ind / 3, ind % 3)
    }
    fn right_rotate_cell(cell: usize) -> usize {
        match cell {
            0 => 2,
            1 => 5,
            2 => 8,
            3 => 1,
            4 => 4,
            5 => 7,
            6 => 0,
            7 => 3,
            8 => 6,
            _ => panic!("uh oh"),
        }
    }
}
impl Board {
    fn rotate_min(&self) -> (usize, u32) {
        let rotations = self.rotations();
        dbg!(&rotations);
        rotations
            .iter()
            .enumerate()
            .min_by(|a, b| Ord::cmp(&a.1, &b.1))
            .map(|(i, &rot)| (i, rot))
            .unwrap()
    }
    fn rotations(&self) -> Vec<u32> {
        (0..3).fold(vec![self.clone().into()], |mut acc, _| {
            acc.push(Self::right_rotate(*acc.last().unwrap()));
            acc
        })
    }
    fn right_rotate(board: u32) -> u32 {
        (0..9).fold(0, |acc, cur| {
            let new_position = Coord::right_rotate_cell(cur as usize) as u32;
            acc | ((MASK & (board >> cur)) << new_position)
        })
    }
}
static MASK: u32 = 3;

#[derive(Clone)]
pub struct Cache(HashMap<u32, (Coord, Outcome)>);
impl Cache {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn check(&self, board: Board) -> Option<(Coord, Outcome)> {
        let (rotations, rotated) = board.rotate_min();
        let back_rotations = (4 - rotations) % 4;
        self.0
            .get(&rotated)
            .map(|(coord, outcome)| (coord.right_rotate(back_rotations), *outcome).to_owned())
    }
    pub fn add(&mut self, board: &Board, res: (Coord, Outcome)) {
        let (_, rotated) = board.rotate_min();
        dbg!(&rotated);
        self.0.insert(rotated, res);
    }
}

mod tests {
    use super::*;
    use rand;
    use rand::Rng;

    #[test]
    fn coord_ind_conversion() {
        for &coord in CELLS.iter() {
            assert_eq!(coord, Coord::from_flat_ind(coord.flat_ind()));
            assert_eq!(coord, coord.right_rotate(4));
        }
    }

    #[test]
    // fn cached_result_is_rotated() {
    //     // let mut rng = rand::thread_rng();
    //     let mut cache = Cache::new();
    //
    //     for &coord in CELLS.iter() {
    //         let mut board = Board::new();
    //         board.set(coord, Piece::Cross).unwrap();
    //         cache.add(&board, (coord, Outcome::Win));
    //     }
    //     assert_eq!(cache.0.keys().count(), 3);
    // }
    #[test]
    fn board_rotate() {
        let mut board = Board::new();
        assert_eq!(board.rotate_min(), (0, 0));
        board.set(Coord(0, 0), Piece::Cross).unwrap();
        assert_eq!(board.rotate_min(), (0, 2));
    }
}
