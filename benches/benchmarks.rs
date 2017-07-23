#![feature(test)]

extern crate test;
extern crate libgo;


#[cfg(test)]
mod tests {
    use test::Bencher;
    use libgo::game::Game;
    use libgo::game::board::{Board, Move, State};
    use libgo::game::matrix::Matrix;
    use libgo::game::player::Player;
    use libgo::game::vertex::Vertex;

    fn black_checkered_matrix(size: usize) -> Matrix<State> {
        let mut matrix = Matrix::with_size(size);
        for y in 0..matrix.size() {
            for x in 0..matrix.size() {
                if (y % 2 == 0 && x % 2 == 0) || (y % 2 != 0 && x % 2 != 0) {
                    matrix[&Vertex { x: x, y: y }] = State::Black;
                }
            }
        }
        matrix
    }

    #[bench]
    fn bench_first_move_genmove_random(b: &mut Bencher) {
        let mut game = Game::new();
        b.iter(|| {
            game.genmove_random(Player::Black);
            game.undo().unwrap();
        });
    }

    #[bench]
    fn bench_first_move_all_legal_moves(b: &mut Bencher) {
        let game = Game::new();
        b.iter(|| game.all_legal_moves(Player::Black));
    }

    #[bench]
    fn bench_first_move_play_in_game(b: &mut Bencher) {
        let mut game = Game::new();
        let center = game.board().center_point();
        let mov = Move {
            player: Player::Black,
            vertex: center,
        };
        b.iter(|| {
            game.play(&mov).unwrap();
            game.undo().unwrap();
        });
    }

    #[bench]
    fn bench_first_move_play_on_board(b: &mut Bencher) {
        let empty_board = Board::with_size(19).unwrap();
        let center = empty_board.center_point().unwrap();
        b.iter(|| {
            let mut board = empty_board.clone();
            board.place_stone(Player::Black, center);
        });
    }

    #[bench]
    fn bench_is_vacant(b: &mut Bencher) {
        let game = Game::new();
        let center = game.board().center_point().unwrap();
        b.iter(|| game.board().is_vacant(center));
    }

    #[bench]
    fn bench_split_by_black_empty_board(b: &mut Bencher) {
        let matrix: Matrix<State> = Matrix::with_size(19);
        b.iter(|| matrix.get_regions(|vertex| vertex != &State::Black));
    }

    #[bench]
    fn bench_split_by_black_checkered_board(b: &mut Bencher) {
        let matrix: Matrix<State> = black_checkered_matrix(19);
        b.iter(|| matrix.get_regions(|vertex| vertex != &State::Black));
    }
}
