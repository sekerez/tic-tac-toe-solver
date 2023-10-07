use anyhow::Result;
use itertools::{iproduct, Itertools};
use rand::Rng;

use std::fmt;
use std::io;
use strum_macros;

fn main() {
    let mut game = Game::new().unwrap();
    for i in 0..9 {
        println!("State of the board: \n{}", game.board);
        game.play_board_move(i).unwrap();
        if let Some(player) = game.winner() {
            println!("{player} won!");
            return;
        }
    }
    println!("The game ended in a tie!")
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, strum_macros::Display)]
enum Piece {
    Blank,
    Circle,
    Cross,
}

#[derive(Clone)]
struct Board([Piece; 9]);
impl Board {
    pub fn new() -> Self {
        Self([Piece::Blank; 9])
    }
}
impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}\n{:?}\n{:?}\n",
            &self.0[..3],
            &self.0[3..6],
            &self.0[6..]
        )
    }
}

static SEQUENCES: [[(usize, usize); 3]; 8] = [
    [(0, 0), (1, 0), (2, 0)],
    [(0, 1), (1, 1), (2, 1)],
    [(0, 2), (1, 2), (2, 2)], // Verticals
    [(0, 0), (0, 1), (0, 2)],
    [(1, 0), (1, 1), (1, 2)],
    [(2, 0), (2, 1), (2, 2)], // Horizontals
    [(0, 0), (1, 1), (2, 2)],
    [(0, 2), (1, 1), (2, 0)], // Diagonals
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
    pub fn play_board_move(&mut self, turn: usize) -> Result<(), String> {
        let current_player = self.current_player(turn);
        let (x, y) = self.calculate_move(current_player)?;
        self.set(x, y, current_player.piece)?;
        Ok(())
    }
    pub fn winner(&self) -> Option<Player> {
        SEQUENCES
            .iter()
            .map(|seq| seq.map(|(x, y)| self.get(x, y).unwrap()))
            .filter(|pieces| pieces.iter().all(|&cell| cell == pieces[0]))
            .next()
            .map(|pieces| self.player_by_piece(pieces[0]))
    }
    fn calculate_move(&self, player: Player) -> Result<(usize, usize), String> {
        use Opponent::*;

        match player.opponent {
            Human => Self::human_move(),
            Computer => Ok(self.computer_move()),
        }
    }
    fn human_move() -> Result<(usize, usize), String> {
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
                    if (0, 0) <= (x, y) && (x, y) <= (2, 2) {
                        break (x, y);
                    }
                    println!("{x} and {y} are not valid coordinates");
                }
                _ => {
                    println!("Input is invalid");
                }
            }
        };
        Ok(coords)
    }
    fn computer_move(&self) -> (usize, usize) {
        let mut game_clone = self.clone();
        iproduct!(0..3, 0..3)
            .map(|(i, j)| (i, j, self.best_move(self.players[1], i, j).unwrap()))
            .sorted_by(|a, b| Ord::cmp(&a.2, &b.2))
            .map(|(i, j, _)| (i, j))
            .next()
            .unwrap()
    }
    fn best_move(&mut self, player: Player, x: usize, y: usize) -> Result<Outcome, String> {
        for (i, j) in iproduct!(0..3, 0..3) {
            if self.get(i, j)? == Piece::Blank {
                continue;
            }
            self.set(i, j, player.piece)?;
            match self.winner() {
                Some(pl) if pl == player => {
                    return Ok((i, j));
                }
                Some(_) | None => (),
            };
        }
        Ok((0, 0))
    }
    fn get(&self, x: usize, y: usize) -> Result<Piece, String> {
        if x > 2 || y > 2 {
            return Err("The following coordinates are off: {x} {y}".to_string());
        }
        Ok(self.board.0[x * 3 + y])
    }
    fn set(&mut self, x: usize, y: usize, piece: Piece) -> Result<(), String> {
        use Piece::*;

        match self.get(x, y)? {
            Circle | Cross => Err("Piece already present".to_string()),
            Blank => {
                self.board.0[x * 3 + y] = piece;
                Ok(())
            }
        }
    }
    fn current_player(&self, turn: usize) -> Player {
        let current_piece = if turn % 2 == 0 {
            Piece::Circle
        } else {
            Piece::Cross
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
