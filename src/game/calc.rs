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
    fn rotate_min(self) -> (usize, u32) {
        (0..4)
            .map(|i| self.right_rotate(i).to_owned().into())
            .enumerate()
            .min_by(|a, b| Ord::cmp(&a.1, &b.1))
            .unwrap()
    }
    fn right_rotate(self, times: usize) -> Self {
        CELLS.iter().fold(Board::new(), |mut board, &coord| {
            let piece = self.get(coord).unwrap();
            let new_position = Self::right_rotate_coord(coord, times);
            board.place(new_position, piece).unwrap();
            board
        })
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
        self.0
            .get(&rotated)
            .map(|&(coord, outcome)| (Board::right_rotate_coord(coord, back_rotations), outcome))
    }
    pub fn add(&mut self, board: &Board, res: (Coord, Outcome)) {
        let (_, rotated) = board.rotate_min();
        self.0.insert(rotated, res);
    }
}

#[allow(dead_code)]
mod tests {
    use super::*;

    fn default_rotations() -> [Board; 4] {
        [
            Board::from(
                r#"
            X| | 
             | | 
             | | "#,
            ),
            Board::from(
                r#"
             | |X
             | | 
             | | "#,
            ),
            Board::from(
                r#"
             | | 
             | | 
             | |X"#,
            ),
            Board::from(
                r#"
             | | 
             | | 
            X| | "#,
            ),
        ]
    }

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
        for (&expected, times) in default_rotations().iter().zip(0..4) {
            assert_eq!(board.right_rotate(times), expected)
        }
    }

    #[test]
    fn rerotate() {
        fn test_rotation(expected_results: [(usize, usize); 4]) {
            let rotations = default_rotations();
            let mut cache = Cache::new();
            cache.add(&rotations[0], (expected_results[0], Outcome::Win));
            for (&rotation, expected) in rotations.iter().zip(expected_results) {
                assert_eq!(cache.check(rotation), Some((expected, Outcome::Win)));
            }
        }
        test_rotation([(2, 2), (2, 0), (0, 0), (0, 2)]);
        test_rotation([(1, 2), (2, 1), (1, 0), (0, 1)]);
        test_rotation([(1, 1), (1, 1), (1, 1), (1, 1)]);
    }
}
