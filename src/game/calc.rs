use super::model::{Board, Coord, Outcome};
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
impl Board {
    fn rotate(&self) -> u32 {
        let mut rotations: Vec<u32> = vec![self.clone().into()];
        for _ in 0..3 {
            rotations.push(Self::right_rotate(*rotations.last().unwrap()));
        }
        *rotations.iter().min().unwrap()
    }
    fn right_rotate(board: u32) -> u32 {
        (0..9).fold(0, |acc, cur| {
            let new_position = match cur {
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
            };
            acc | (MASK & (board >> cur)) << new_position
        })
    }
}
static MASK: u32 = u32::MAX << 2;

#[derive(Clone)]
pub struct Cache(HashMap<u32, (Coord, Outcome)>);
impl Cache {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn check(&self, board: Board) -> Option<(Coord, Outcome)> {
        let rotated = board.rotate();
        self.0.get(&rotated).map(|r| r.to_owned())
    }
    pub fn add(&mut self, board: &Board, res: (Coord, Outcome)) {
        let rotated = board.rotate();
        self.0.insert(rotated, res);
    }
}
