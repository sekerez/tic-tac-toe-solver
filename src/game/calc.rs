use super::model::{Board, Coord, Outcome, Piece, CELLS};
use std::{collections::HashMap, ops::BitOr};

impl Into<u32> for Board {
    fn into(self) -> u32 {
        self.0
            .iter()
            .flatten()
            .enumerate()
            .map(|(i, &piece)| Into::<u32>::into(piece) << 2 * i)
            .reduce(u32::bitor)
            .unwrap()
    }
}
impl Board {
    fn rotate_min(&self) -> (usize, u32) {
        let rotations = self.rotations();
        rotations
            .iter()
            .map(|b| b.to_owned().into())
            .enumerate()
            .min_by(|a, b| Ord::cmp(&a.1, &b.1))
            .map(|(i, rot)| (i, rot))
            .unwrap()
    }
    fn rotations(&self) -> Vec<Board> {
        (0..3).fold(vec![self.clone()], |mut acc, _| {
            acc.push(acc.last().unwrap().right_rotate());
            acc
        })
    }
    fn right_rotate(self) -> Self {
        let mut new_board = self;
        CELLS.iter().for_each(|&coord| {
            let new_position = Self::right_rotate_coord(coord, 1);
            new_board.place(new_position, self.get(coord).unwrap());
        });
        new_board
    }
    pub fn right_rotate_coord(coord: Coord, times: usize) -> Coord {
        let actual_times = times % 4;
        (0..actual_times).fold(coord, |acc, _| (acc.1, 2 - acc.0))
    }
}

#[derive(Clone)]
pub struct Cache(HashMap<u32, (Coord, Outcome)>);
impl Cache {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn check(&self, board: Board) -> Option<(Coord, Outcome)> {
        let (rotations, rotated) = board.rotate_min();
        let back_rotations = 4 - rotations;
        self.0.get(&rotated).map(|(coord, outcome)| {
            (Board::right_rotate_coord(*coord, back_rotations), *outcome).to_owned()
        })
    }
    pub fn add(&mut self, board: &Board, res: (Coord, Outcome)) {
        let (_, rotated) = board.rotate_min();
        self.0.insert(rotated, res);
    }
}

mod tests {
    use super::Piece as P;
    use super::*;
    use rand;
    use rand::Rng;

    #[test]
    fn cached_result_is_rotated() {
        // let mut rng = rand::thread_rng();
        let mut cache = Cache::new();

        for &coord in CELLS.iter() {
            let mut board = Board::new();
            board.place(coord, Piece::Cross).unwrap();
            cache.add(&board, (coord, Outcome::Win));
        }
        assert_eq!(cache.0.keys().count(), 3);
    }
    #[test]
    fn board_rotate() {
        let mut board = Board::new();
        assert_eq!(board.rotate_min(), (0, 0));
        board.place((0, 0), Piece::Cross).unwrap();
        assert_eq!(board.rotate_min(), (0, 2));
        assert_eq!(
            board.right_rotate(),
            Board([
                [P::Blank, P::Blank, P::Cross],
                [P::Blank, P::Blank, P::Blank],
                [P::Blank, P::Blank, P::Blank],
            ])
        )
    }
}
