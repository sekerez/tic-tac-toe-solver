use anyhow::Result;

use std::fmt;
use strum_macros;

pub static CELLS: [Coord; 9] = [
    Coord(0, 0),
    Coord(0, 1),
    Coord(0, 2),
    Coord(1, 0),
    Coord(1, 1),
    Coord(1, 2),
    Coord(2, 0),
    Coord(2, 1),
    Coord(2, 2),
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

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Coord(pub usize, pub usize);
impl Coord {
    pub fn out_of_bounds(self) -> Result<(), String> {
        if self.0 > 2 || self.1 > 2 {
            return Err(format!(
                "The following coordinates are out of bounds: {} {}",
                self.0, self.1
            )
            .to_string());
        }
        Ok(())
    }
    pub fn flat_ind(self) -> usize {
        self.0 * 3 + self.1
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Board(pub [Piece; 9]);
impl Board {
    pub fn new() -> Self {
        Self([Piece::Blank; 9])
    }
    pub fn get(&self, coord: Coord) -> Result<Piece, String> {
        coord.out_of_bounds()?;
        Ok(self.0[coord.flat_ind()])
    }
    pub fn set(&mut self, coord: Coord, piece: Piece) -> Result<(), String> {
        use Piece as P;

        coord.out_of_bounds()?;
        match self.get(coord)? {
            P::Circle | P::Cross => Err("Piece already present".to_string()),
            P::Blank => {
                self.0[coord.flat_ind()] = piece;
                Ok(())
            }
        }
    }
    pub fn set_to_blank(&mut self, coord: Coord) -> Result<(), String> {
        coord.out_of_bounds()?;
        self.0[coord.flat_ind()] = Piece::Blank;
        Ok(())
    }
    pub fn used_cells(&self) -> usize {
        self.0.iter().filter(|&&p| p != Piece::Blank).count()
    }
}
impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let formatted = self
            .0
            .chunks(3)
            .map(|chunk| format!("{}|{}|{}", chunk[0], chunk[1], chunk[2]))
            .collect::<Vec<_>>()
            .join("\n");
        write!(f, "{}\n", formatted)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum_macros::Display)]
pub enum Opponent {
    Human,
    Computer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum_macros::Display, PartialOrd, Ord)]
pub enum Outcome {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Player {
    pub opponent: Opponent,
    pub piece: Piece,
}
impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.opponent, self.piece)
    }
}

impl Player {
    pub fn new(opponent: Opponent, piece: Piece) -> Result<Self, String> {
        if let Piece::Blank = piece {
            return Err("Can't create a player with a blank piece".to_string());
        }
        Ok(Player { opponent, piece })
    }
}
