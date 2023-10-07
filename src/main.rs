use anyhow::Result;
use rand::Rng;

use std::fmt;
use std::io;
use strum_macros;

fn main() {
    let mut game = Game::new().unwrap();
    for _ in 0..9 {
        println!("State of the board: \n{}", game.board);
        game.play_board_move().unwrap();
        if let Some(player) = game.winner() {
            println!("{player} won!\n{}", game.board);
            return;
        }
    }
    println!("The game ended in a tie!\n{}", game.board);
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Piece {
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

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Coord(usize, usize);
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
#[derive(Clone)]
struct Board([Piece; 9]);
impl Board {
    pub fn new() -> Self {
        Self([Piece::Blank; 9])
    }
    fn get(&self, coord: Coord) -> Result<Piece, String> {
        coord.out_of_bounds()?;
        Ok(self.0[coord.flat_ind()])
    }
    fn set(&mut self, coord: Coord, piece: Piece) -> Result<(), String> {
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
    fn set_to_blank(&mut self, coord: Coord) -> Result<(), String> {
        coord.out_of_bounds()?;
        self.0[coord.flat_ind()] = Piece::Blank;
        Ok(())
    }
    fn used_cells(&self) -> usize {
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

static CELLS: [Coord; 9] = [
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
static SEQUENCES: [[Coord; 3]; 8] = [
    [Coord(0, 0), Coord(1, 0), Coord(2, 0)],
    [Coord(0, 1), Coord(1, 1), Coord(2, 1)],
    [Coord(0, 2), Coord(1, 2), Coord(2, 2)], // Verticals
    [Coord(0, 0), Coord(0, 1), Coord(0, 2)],
    [Coord(1, 0), Coord(1, 1), Coord(1, 2)],
    [Coord(2, 0), Coord(2, 1), Coord(2, 2)], // Horizontals
    [Coord(0, 0), Coord(1, 1), Coord(2, 2)],
    [Coord(0, 2), Coord(1, 1), Coord(2, 0)], // Diagonals
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum_macros::Display)]
enum Opponent {
    Human,
    Computer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum_macros::Display, PartialOrd, Ord)]
enum Outcome {
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
struct Player {
    opponent: Opponent,
    piece: Piece,
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

#[derive(Clone)]
struct Game {
    board: Board,
    players: [Player; 2],
}

impl Game {
    pub fn new() -> Result<Self, String> {
        let mut rng = rand::thread_rng();
        let mut pieces = [Piece::Cross, Piece::Circle];
        if rng.gen::<bool>() {
            pieces.reverse();
        }
        Ok(Game {
            board: Board::new(),
            players: [
                Player::new(Opponent::Human, pieces[0])?,
                Player::new(Opponent::Computer, pieces[1])?,
            ],
        })
    }
    pub fn play_board_move(&mut self) -> Result<(), String> {
        let current_player = self.current_player();
        let coord = self.calculate_move(current_player)?;
        self.board.set(coord, current_player.piece)?;
        Ok(())
    }
    pub fn winner(&self) -> Option<Player> {
        SEQUENCES
            .iter()
            .map(|seq| seq.map(|coord| self.board.get(coord).unwrap()))
            .filter(|pieces| {
                pieces
                    .iter()
                    .all(|&cell| cell != Piece::Blank && cell == pieces[0])
            })
            .next()
            .map(|pieces| self.player_by_piece(pieces[0]))
    }
    fn calculate_move(&self, player: Player) -> Result<Coord, String> {
        use Opponent::*;

        match player.opponent {
            Human => self.human_move(),
            Computer => self
                .computer_move()
                .ok_or("No moves left to make".to_string()),
        }
    }
    fn human_move(&self) -> Result<Coord, String> {
        let coords = loop {
            println!("Please enter two numbers between 0 and 2, separated by a space:");

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .map_err(|e| e.to_string())?;

            let numbers: Vec<usize> = input
                .trim()
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();

            match numbers[..] {
                [x, y] => {
                    let coord = Coord(x, y);
                    match self.board.get(coord) {
                        Err(mess) => {
                            println!("{}", mess);
                        }
                        Ok(Piece::Circle) | Ok(Piece::Cross) => {
                            println!("There's already a piece...")
                        }
                        Ok(Piece::Blank) => {
                            break Coord(x, y);
                        }
                    }
                }
                _ => {
                    println!("Input is invalid");
                }
            }
        };
        Ok(coords)
    }
    fn computer_move(&self) -> Option<Coord> {
        let mut game_clone = self.clone();
        game_clone.best_move(self.players[1].piece).map(|mov| mov.0)
    }
    fn best_move(&mut self, piece: Piece) -> Option<(Coord, Outcome)> {
        let mut outcomes = vec![];
        for &coord in CELLS.iter() {
            if self.board.get(coord).unwrap() != Piece::Blank {
                continue;
            }
            self.board.set(coord, piece).unwrap();
            let outcome = match self.winner() {
                Some(_) => Outcome::Win,
                None => self
                    .best_move(piece.opposite())
                    .map_or(Outcome::Tie, |res| res.1.opposite()),
            };
            outcomes.push((coord, outcome));
            self.board.set_to_blank(coord).unwrap();
        }
        outcomes
            .iter()
            .max_by(|a, b| Ord::cmp(&a.1, &b.1))
            .map(|res| res.to_owned())
    }
    fn current_player(&self) -> Player {
        let current_piece = if self.board.used_cells() % 2 == 0 {
            Piece::Cross
        } else {
            Piece::Circle
        };
        self.player_by_piece(current_piece)
    }
    fn player_by_piece(&self, piece: Piece) -> Player {
        if self.players[0].piece == piece {
            self.players[0]
        } else {
            self.players[1]
        }
    }
}
