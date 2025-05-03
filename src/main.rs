mod board;
mod evaluation;
mod movegen;
mod search;
mod transposition;
mod uci;

use board::{Board, Color, Piece};
use evaluation::Evaluator;
use movegen::{Move, MoveGenerator};
use search::Search;
use std::io;
use std::time::{Duration, Instant};
use uci::UciHandler;

fn main() {
    let mut uci = UciHandler::new();
    uci.run().unwrap();
}

fn parse_move(input: &str) -> Option<Move> {
    // TODO: Implement move parsing from algebraic notation
    None
}

fn format_move(mv: &Move) -> String {
    // TODO: Implement move formatting to algebraic notation
    String::new()
}

fn is_move_legal(board: &Board, mv: &Move) -> bool {
    let generator = MoveGenerator::new();
    let legal_moves = generator.generate_moves(board);
    legal_moves.contains(mv)
} 