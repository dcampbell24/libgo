use criterion::{criterion_group, criterion_main, Criterion};
// use std::hint::black_box;

use libgo::game::board::{Board, Move, State};
use libgo::game::matrix::Matrix;
use libgo::game::player::Player;
use libgo::game::vertex::Vertex;
use libgo::game::Game;

fn black_checkered_matrix(size: usize) -> Matrix<State> {
    let mut matrix = Matrix::with_size(size);
    for y in 0..matrix.size() {
        for x in 0..matrix.size() {
            if (y % 2 == 0 && x % 2 == 0) || (y % 2 != 0 && x % 2 != 0) {
                matrix[&Vertex { x, y }] = State::Black;
            }
        }
    }
    matrix
}

fn bench_first_move_genmove_random(c: &mut Criterion) {
    let mut game = Game::new();
    c.bench_function("bench_first_move_genmove_random", |b| {
        b.iter(|| {
            game.genmove_random(Player::Black);
            game.undo().unwrap();
        });
    });
}

fn bench_first_move_all_legal_moves(c: &mut Criterion) {
    let game = Game::new();
    c.bench_function("bench_first_move_all_legal_moves", |b| {
        b.iter(|| game.all_legal_moves(Player::Black));
    });
}

fn bench_first_move_play_in_game(c: &mut Criterion) {
    let mut game = Game::new();
    let center = game.board().center_point();
    let mov = Move {
        player: Player::Black,
        vertex: center,
    };
    c.bench_function("bench_first_move_play_in_game", |b| {
        b.iter(|| {
            game.play(&mov).unwrap();
            game.undo().unwrap();
        });
    });
}

fn bench_first_move_play_on_board(c: &mut Criterion) {
    let empty_board = Board::with_size(19).unwrap();
    let center = empty_board.center_point().unwrap();
    c.bench_function("bench_first_move_play_on_board", |b| {
        b.iter(|| {
            let mut board = empty_board.clone();
            board.place_stone(Player::Black, center);
        });
    });
}

fn bench_is_vacant(c: &mut Criterion) {
    let game = Game::new();
    let center = game.board().center_point().unwrap();
    c.bench_function("bench_is_vacant", |b| {
        b.iter(|| game.board().is_vacant(center));
    });
}

fn bench_not_black_regions_on_empty_board(c: &mut Criterion) {
    let matrix: Matrix<State> = Matrix::with_size(19);
    c.bench_function("bench_not_black_regions_on_empty_board", |b| {
        b.iter(|| matrix.get_regions(|vertex| vertex != &State::Black));
    });
}

fn bench_not_black_regions_on_black_checkered_board(c: &mut Criterion) {
    let matrix: Matrix<State> = black_checkered_matrix(19);
    c.bench_function("bench_not_black_on_black_checkered_board", |b| {
        b.iter(|| matrix.get_regions(|vertex| vertex != &State::Black));
    });
}

fn bench_regions_by_value_on_empty_board(c: &mut Criterion) {
    let matrix: Matrix<State> = Matrix::with_size(19);
    c.bench_function("bench_regions_by_value_on_empty_board", |b| {
        b.iter(|| matrix.get_regions_by_value());
    });
}

fn bench_regions_by_value_on_black_checkered_board(c: &mut Criterion) {
    let matrix: Matrix<State> = black_checkered_matrix(19);
    c.bench_function("bench_regions_by_value_on_black_checkered_board", |b| {
        b.iter(|| matrix.get_regions_by_value());
    });
}

criterion_group!(
    benches,
    bench_first_move_genmove_random,
    bench_first_move_all_legal_moves,
    bench_first_move_play_in_game,
    bench_first_move_play_on_board,
    bench_is_vacant,
    bench_not_black_regions_on_empty_board,
    bench_not_black_regions_on_black_checkered_board,
    bench_regions_by_value_on_empty_board,
    bench_regions_by_value_on_black_checkered_board,
);
criterion_main!(benches);
