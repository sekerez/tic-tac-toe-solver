mod calc;
mod model;

use anyhow::Result;
use calc::Cache;
use itertools::Itertools;
use model::{Board, Coord, Outcome, Piece, Player, CELLS};
use rand::seq::SliceRandom;
use rand::Rng;
use std::io;

static SEQUENCES: [[Coord; 3]; 8] = [
    [(0, 0), (1, 0), (2, 0)],
    [(0, 1), (1, 1), (2, 1)],
    [(0, 2), (1, 2), (2, 2)], // Verticals
    [(0, 0), (0, 1), (0, 2)],
    [(1, 0), (1, 1), (1, 2)],
    [(2, 0), (2, 1), (2, 2)], // Horizontals
    [(0, 0), (1, 1), (2, 2)],
    [(0, 2), (1, 1), (2, 0)], // Diagonals
];

#[derive(Clone)]
pub struct Game {
    pub board: Board,
    first: Player,
    cache: Cache,
}

impl Game {
    pub fn new() -> Result<Self, String> {
        use Player as P;
        let mut rng = rand::thread_rng();
        let first = if rng.gen::<bool>() {
            P::Computer
        } else {
            P::Human
        };
        Ok(Game {
            board: Board::new(),
            cache: Cache::new(),
            first,
        })
    }
    pub fn play_board_move(&mut self) -> Result<(), String> {
        let (coord, piece) = self.calculate_move()?;
        self.board.place(coord, piece)?;
        Ok(())
    }
    pub fn winner(&self) -> Option<Player> {
        SEQUENCES
            .iter()
            .map(|seq| seq.map(|coord| self.board.get(coord).unwrap()))
            .find(|pieces| {
                pieces[0] != Piece::Blank && pieces.iter().all(|&cell| cell == pieces[0])
            })
            .map(|pieces| self.player_by_piece(pieces[0]))
    }
    fn calculate_move(&self) -> Result<(Coord, Piece), String> {
        use Player as P;
        let (current_player, current_piece) = self.current_player();

        let coord = match current_player {
            P::Human => self.human_move(),
            P::Computer => self
                .computer_move()
                .ok_or("No moves left to make".to_string()),
        }?;
        Ok((coord, current_piece))
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
                    let coord = (x, y);
                    match self.board.get(coord) {
                        Err(mess) => {
                            println!("{}", mess);
                        }
                        Ok(Piece::Circle) | Ok(Piece::Cross) => {
                            println!("There's already a piece...")
                        }
                        Ok(Piece::Blank) => {
                            break (x, y);
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
        let (_, current_piece) = self.current_player();
        println!("Computer calculating move...");
        let mut game_clone = self.clone();
        game_clone.best_move(current_piece).map(|mov| mov.1)
    }
    fn best_move(&mut self, piece: Piece) -> Option<(Outcome, Coord)> {
        let mut outcomes = vec![];
        for &coord in CELLS.iter() {
            if self.board.get(coord).unwrap() != Piece::Blank {
                continue;
            }
            self.board.place(coord, piece).unwrap();
            let outcome = match self.cache.check(self.board) {
                Some(res) => res.1,
                None => {
                    let outcome = match self.winner() {
                        Some(_) => Outcome::Win,
                        None => self
                            .best_move(piece.opposite())
                            .map_or(Outcome::Tie, |res| res.0.opposite()),
                    };
                    self.cache.add(&self.board, (coord, outcome));
                    outcome
                }
            };
            self.board.reset(coord).unwrap();
            outcomes.push((outcome, coord));
        }
        outcomes
            .into_iter()
            .sorted_by(|a, b| Ord::cmp(&a.0, &b.0))
            .group_by(|r| r.0)
            .into_iter()
            .map(|(outcome, group)| {
                let rand_coord = group
                    .into_iter()
                    .collect::<Vec<_>>()
                    .choose(&mut rand::thread_rng())
                    .unwrap()
                    .1;
                (outcome, rand_coord)
            })
            .next()
    }
    fn current_player(&self) -> (Player, Piece) {
        if self.board.used_cells() % 2 == 0 {
            (self.first, Piece::Cross)
        } else {
            (self.first.opposite(), Piece::Circle)
        }
    }
    fn player_by_piece(&self, piece: Piece) -> Player {
        use Player as P;
        if (self.first == P::Computer) == (piece == Piece::Cross) {
            P::Computer
        } else {
            P::Human
        }
    }
}

#[allow(dead_code)]
mod tests {
    use super::*;

    fn make_game(board: Board) -> Game {
        Game {
            board,
            first: Player::Computer,
            cache: Cache::new(),
        }
    }

    #[test]
    fn winner_one() {
        let game = make_game(Board::from(
            r#"
            X|O| 
             |O| 
            X|O|X"#,
        ));
        let winner = game.winner();
        assert_eq!(winner, Some(Player::Human));
    }
    #[test]
    fn winner_two() {
        let game = make_game(Board::from(
            r#"
            X|O|X
            X|X|O
            O| |O"#,
        ));
        let winner = game.winner();
        assert_eq!(winner, None);
    }
    #[test]
    fn make_best_move() {
        let boards: [(&str, Coord); 2] = [
            (
                r#"
            X|O|X
             | | 
             | |O"#,
                (2, 0),
            ),
            (
                r#"
            X|O|X
            X| |O
            O|X|O"#,
                (1, 1),
            ),
        ];
        for (sketch, expected) in boards {
            let game = make_game(Board::from(sketch));
            let best_move = game.computer_move();
            assert_eq!(best_move, Some(expected));
        }
    }
}
