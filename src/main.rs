mod game;

use game::Game;

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
