mod calc;
mod model;

use anyhow::Result;
use calc::Cache;
use itertools::Itertools;
use model::{Board, Coord, Opponent, Outcome, Piece, Player, CELLS};
use rand::Rng;
use std::io;

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

#[derive(Clone)]
pub struct Game {
    pub board: Board,
    players: [Player; 2],
    cache: Cache,
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
            cache: Cache::new(),
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
        println!("Computer calculating move...");
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
            if let Some(cached) = self.cache.check(self.board) {
                return Some(cached);
            }
            let outcome = match self.winner() {
                Some(_) => Outcome::Win,
                None => self
                    .best_move(piece.opposite())
                    .map_or(Outcome::Tie, |res| res.1.opposite()),
            };
            self.cache.add(&self.board, (coord, outcome));
            self.board.set_to_blank(coord).unwrap();
            if outcome == Outcome::Win {
                return Some((coord, outcome));
            }
            outcomes.push((coord, outcome));
        }
        let res: Vec<_> = outcomes
            .iter()
            .sorted_by(|a, b| Ord::cmp(&a.1, &b.1))
            .collect();

        dbg!(&res);
        res.first().map(|res| res.to_owned().to_owned())
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
