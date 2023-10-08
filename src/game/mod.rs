mod calc;
mod model;

use anyhow::Result;
use calc::Cache;
use itertools::Itertools;
use model::{Board, Coord, Opponent, Outcome, Piece, Player, CELLS};
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
        self.board.place(coord, current_player.piece)?;
        Ok(())
    }
    pub fn winner(&self) -> Option<Player> {
        SEQUENCES
            .iter()
            .map(|seq| seq.map(|coord| self.board.get(coord).unwrap()))
            .filter(|pieces| {
                pieces[0] != Piece::Blank && pieces.iter().all(|&cell| cell == pieces[0])
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
        println!("Computer calculating move...");
        let mut game_clone = self.clone();
        game_clone.best_move(self.players[1].piece).map(|mov| mov.0)
    }
    fn best_move(&mut self, piece: Piece) -> Option<(Coord, Outcome)> {
        // Problem: When playing second, sometimes the computer fucks up
        // such as with this:
        // State of the board:
        // X|O|
        //  |O|
        // X| |X
        //
        // Computer calculating move...
        // State of the board:
        // X|O|
        //  |O|O
        // X| |X
        let mut outcomes = vec![];
        for &coord in CELLS.iter() {
            if self.board.get(coord).unwrap() != Piece::Blank {
                continue;
            }
            self.board.place(coord, piece).unwrap();
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
            self.board.reset(coord).unwrap();
            if outcome == Outcome::Win {
                dbg!(outcomes);
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

mod tests {
    use super::*;
    use Piece as P;

    fn make_game(board: Board) -> Game {
        Game {
            board,
            players: [
                Player::new(Opponent::Human, P::Cross).unwrap(),
                Player::new(Opponent::Computer, P::Circle).unwrap(),
            ],
            cache: Cache::new(),
        }
    }

    #[test]
    fn winner() {
        let game = make_game(Board([
            [P::Cross, P::Circle, P::Blank],
            [P::Blank, P::Circle, P::Blank],
            [P::Cross, P::Circle, P::Cross],
        ]));
        let winner = game.winner();
        assert_eq!(winner, Some(game.players[1]));
    }
    #[test]
    fn make_best_move() {
        let game = make_game(Board([
            [P::Cross, P::Circle, P::Blank],
            [P::Blank, P::Circle, P::Blank],
            [P::Cross, P::Blank, P::Cross],
        ]));
        let best_move = game.computer_move();
        assert_eq!(best_move, Some((2, 1)));
    }
}
