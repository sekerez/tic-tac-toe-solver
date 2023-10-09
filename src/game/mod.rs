mod calc;
mod model;

use anyhow::Result;
use calc::Cache;
use model::{Board, Coord, Outcome, Piece, Player, CELLS};
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
        game_clone.best_move(current_piece).map(|mov| mov.0)
    }
    fn best_move(&mut self, piece: Piece) -> Option<(Coord, Outcome)> {
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
                            .map_or(Outcome::Tie, |res| res.1.opposite()),
                    };
                    self.cache.add(&self.board, (coord, outcome));
                    outcome
                }
            };
            self.board.reset(coord).unwrap();
            if outcome == Outcome::Win {
                return Some((coord, outcome));
            }
            outcomes.push((coord, outcome));
        }
        outcomes
            .iter()
            .find(|(_, outcome)| *outcome == Outcome::Tie)
            .or(outcomes.first())
            .map(|r| r.to_owned())
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
    use Piece as P;

    fn make_game(board: Board, computer_first: bool) -> Game {
        use Player as Pl;

        let first = if computer_first {
            Pl::Computer
        } else {
            Pl::Human
        };
        Game {
            board,
            first,
            cache: Cache::new(),
        }
    }

    #[test]
    fn winner_one() {
        let game = make_game(
            Board([
                [P::Cross, P::Circle, P::Blank],
                [P::Blank, P::Circle, P::Blank],
                [P::Cross, P::Circle, P::Cross],
            ]),
            false,
        );
        let winner = game.winner();
        assert_eq!(winner, Some(Player::Computer));
    }
    #[test]
    fn winner_two() {
        let game = make_game(
            Board([
                [P::Cross, P::Circle, P::Cross],
                [P::Cross, P::Cross, P::Circle],
                [P::Circle, P::Blank, P::Circle],
            ]),
            true,
        );
        let winner = game.winner();
        assert_eq!(winner, None);
    }
    #[test]
    fn make_best_move() {
        let boards: [(Board, Coord); 2] = [
            (
                Board([
                    [P::Cross, P::Circle, P::Cross],
                    [P::Blank, P::Blank, P::Blank],
                    [P::Blank, P::Blank, P::Circle],
                ]),
                (2, 0),
            ),
            (
                Board([
                    [P::Cross, P::Circle, P::Cross],
                    [P::Cross, P::Blank, P::Circle],
                    [P::Circle, P::Cross, P::Circle],
                ]),
                (1, 1),
            ),
        ];
        for (board, expected) in boards {
            let game = make_game(board, true);
            let best_move = game.computer_move();
            assert_eq!(best_move, Some(expected));
        }
    }
}
