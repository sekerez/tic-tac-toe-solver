use anyhow::Result;

use std::fmt;
use strum_macros;

pub type Coord = (usize, usize);

#[rustfmt::skip]
pub static CELLS: [Coord; 9] = [
    (0, 0), (0, 1), (0, 2),
    (1, 0), (1, 1), (1, 2),
    (2, 0), (2, 1), (2, 2),
];

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Piece {
    Blank,
    Circle,
    Cross,
}
impl Piece {
    pub fn opposite(self) -> Self {
        use Piece as P;
        match self {
            P::Blank => P::Blank,
            P::Circle => P::Cross,
            P::Cross => P::Circle,
        }
    }
}
impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Piece as P;
        let symbol = match self {
            P::Blank => ' ',
            P::Circle => 'O',
            P::Cross => 'X',
        };
        write!(f, "{}", symbol)
    }
}
impl From<String> for Piece {
    fn from(rep: String) -> Self {
        use Piece as P;
        match &rep[..] {
            "X" => P::Cross,
            "O" => P::Circle,
            " " => P::Blank,
            _ => panic!("{rep} is invalid as a piece representation."),
        }
    }
}
impl Into<String> for Piece {
    fn into(self) -> String {
        format!("{}", self)
    }
}
impl Into<u32> for Piece {
    fn into(self) -> u32 {
        use Piece as P;
        match self {
            P::Blank => 0,
            P::Circle => 1,
            P::Cross => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Board(pub [[Piece; 3]; 3]);
impl Board {
    pub fn new() -> Self {
        Self([[Piece::Blank; 3]; 3])
    }
    pub fn get(&self, coord: Coord) -> Result<Piece, String> {
        self.0
            .get(coord.0)
            .and_then(|arr| arr.get(coord.1))
            .ok_or(format!(
                "The following coordinates are out of bounds: {} {}",
                coord.0, coord.1
            ))
            .copied()
    }
    pub fn place(&mut self, coord: Coord, piece: Piece) -> Result<(), String> {
        use Piece as P;
        match self.get(coord)? {
            P::Blank => (),
            existing_piece @ (P::Cross | P::Circle) => {
                return Err(format!("Piece already present: {}", existing_piece));
            }
        };
        self.set(coord, piece);
        Ok(())
    }
    pub fn reset(&mut self, coord: Coord) -> Result<(), String> {
        self.get(coord)?;
        self.set(coord, Piece::Blank);
        Ok(())
    }
    fn set(&mut self, coord: Coord, piece: Piece) {
        self.0[coord.0][coord.1] = piece;
    }
    pub fn used_cells(&self) -> usize {
        self.0
            .iter()
            .map(|arr| arr.iter().filter(|&&p| p != Piece::Blank).count())
            .sum()
    }
}
impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cells = CELLS
            .chunks(3)
            .map(|row| {
                row.iter()
                    .map(|&coord| self.get(coord).unwrap().into())
                    .collect::<Vec<String>>()
                    .join("|")
            })
            .collect::<Vec<_>>()
            .join("\n");
        write!(f, "{}\n", cells)
    }
}
// impl From<String> for Board {
//     fn from(rep: String) -> Self {
//         Board(rep.split("\n").map(|s| s.split("|").map(parse)))
//     }
// }

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum_macros::Display)]
pub enum Player {
    Human,
    Computer,
}
impl Player {
    pub fn opposite(self) -> Self {
        use Player as P;
        match self {
            P::Human => P::Computer,
            P::Computer => P::Human,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum_macros::Display, PartialOrd, Ord)]
pub enum Outcome {
    // TODO delete
    Win,
    Tie,
    Loss,
}

impl Outcome {
    pub fn opposite(self) -> Self {
        use Outcome as O;
        match self {
            O::Win => O::Loss,
            O::Tie => O::Tie,
            O::Loss => O::Win,
        }
    }
}
