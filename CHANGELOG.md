# Changelog

## Unreleased

## [0.5.0](https://crates.io/crates/libgo/0.5.0)

### Changed

- Update the repository to edition 2021 and rust 1.62.1.
- Update clap to version 4 and slightly change the interface to the GTP server.
- Fix lints.
- Store cargo.lock in version control.
- Update the license.
- Switch to criterion for benchmarks.

## [0.4.0](https://crates.io/crates/libgo/0.4.0)

- Implement Display for Player
- Add limit to moves in a Game, MAX_MOVES (512)
- Add is_over method to Game
- Add player_turn method to Game
