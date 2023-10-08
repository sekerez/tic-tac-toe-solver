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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Board(pub [[Piece; 3]; 3]);
impl Board {
    pub fn new() -> Self {
        Self([[Piece::Blank; 3]; 3])
    }
    pub fn get(&self, coord: Coord) -> Result<Piece, &str> {
        self.get_by_coord(coord).map(|p| p.to_owned())
    }
    pub fn set(&mut self, coord: Coord, piece: Piece) -> Result<(), &str> {
        use Piece as P;
        let existing_piece = self.get_by_coord(coord).and_then(|p| match *p {
            P::Blank => Ok(p),
            P::Cross | P::Circle => Err(&format!("Piece already present: {p}")[..]),
        })?;
        *existing_piece = piece;
        Ok(())
    }
    pub fn set_to_blank(&mut self, coord: Coord) -> Result<(), String> {
        *self.get_by_coord(coord)? = Piece::Blank;
        Ok(())
    }
    pub fn used_cells(&self) -> usize {
        self.0
            .iter()
            .map(|arr| arr.iter().filter(|&&p| p != Piece::Blank).count())
            .sum()
    }
    fn get_by_coord(&mut self, coord: Coord) -> Result<&mut Piece, &str> {
        self.0
            .get_mut(coord.0)
            .and_then(|arr| arr.get_mut(coord.1))
            .ok_or(
                &format!(
                    "The following coordinates are out of bounds: {} {}",
                    coord.0, coord.1
                )[..],
            )
    }
}
impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}|{}|{}\n{}|{}|{}\n{}|{}|{}\n",
            self.0[0],
            self.0[1],
            self.0[2],
            self.0[3],
            self.0[4],
            self.0[5],
            self.0[6],
            self.0[7],
            self.0[8],
        )
    }
}
impl From<String> for Board {
    fn from(rep: String) -> Self {
        Board(rep.split("\n").map(|s| s.split("|").map(parse)))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum_macros::Display)]
pub enum Opponent {
    Human,
    Computer,
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
