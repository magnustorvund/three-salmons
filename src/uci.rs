use crate::board::{Board, Color, Piece};
use crate::movegen::{MoveGenerator, Move};
use crate::evaluation::Evaluator;
use crate::search::Search;
use anyhow::Result;
use std::io::{self, BufRead, Write};
use std::str::FromStr;
use std::time::Duration;

pub struct UciHandler {
    board: Board,
    move_generator: MoveGenerator,
    evaluator: Evaluator,
    search_time: u64, // in milliseconds
    search: Search,
}

impl UciHandler {
    pub fn new() -> Self {
        UciHandler {
            board: Board::new(),
            move_generator: MoveGenerator::new(),
            evaluator: Evaluator::new(),
            search_time: 1000, // Default 1 second per move
            search: Search::new(),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let mut reader = stdin.lock();
        let mut line = String::new();

        while reader.read_line(&mut line).unwrap() > 0 {
            let command = line.trim();
            
            match command {
                "quit" => break,
                "uci" => {
                    println!("id name Three Salmons");
                    println!("id author Magnus Torvund");
                    println!("uciok");
                }
                "isready" => println!("readyok"),
                "ucinewgame" => {
                    self.board = Board::new();
                }
                cmd if cmd.starts_with("position") => {
                    let parts: Vec<&str> = cmd.split_whitespace().collect();
                    self.handle_position(&parts[1..]);
                }
                cmd if cmd.starts_with("go") => {
                    let parts: Vec<&str> = cmd.split_whitespace().collect();
                    let response = self.handle_go(&parts[1..]);
                    print!("{}", response);
                }
                _ => {}
            }
            
            stdout.flush()?;
            line.clear();
        }
        Ok(())
    }

    pub fn handle_command(&mut self, command: &str) -> Result<String> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok("".to_string());
        }

        match parts[0] {
            "uci" => Ok(self.handle_uci()),
            "isready" => Ok("readyok\n".to_string()),
            "ucinewgame" => Ok(self.handle_ucinewgame()),
            "position" => Ok(self.handle_position(&parts[1..])),
            "go" => Ok(self.handle_go(&parts[1..])),
            "quit" => Ok("".to_string()),
            _ => Ok("".to_string()),
        }
    }

    fn handle_uci(&self) -> String {
        "id name Three Salmons\nid author Magnus Torvund\nuciok\n".to_string()
    }

    fn handle_ucinewgame(&mut self) -> String {
        self.board = Board::new();
        "".to_string()
    }

    fn handle_position(&mut self, parts: &[&str]) -> String {
        if parts.is_empty() {
            return "".to_string();
        }

        match parts[0] {
            "startpos" => {
                self.board = Board::new();
                if parts.len() > 1 && parts[1] == "moves" {
                    for move_str in &parts[2..] {
                        if let Some(mv) = self.parse_move(move_str) {
                            self.board.make_move(mv);
                        }
                    }
                }
            }
            "fen" => {
                if parts.len() > 1 {
                    let fen = parts[1..].join(" ");
                    if let Ok(board) = Board::from_fen(&fen) {
                        self.board = board;
                        if parts.len() > 6 && parts[6] == "moves" {
                            for move_str in &parts[7..] {
                                if let Some(mv) = self.parse_move(move_str) {
                                    self.board.make_move(mv);
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        "".to_string()
    }

    fn parse_move(&self, move_str: &str) -> Option<Move> {
        if move_str.len() != 4 && move_str.len() != 5 {
            return None;
        }

        let from_file = move_str.chars().nth(0)? as u8 - b'a';
        let from_rank = move_str.chars().nth(1)? as u8 - b'1';
        let to_file = move_str.chars().nth(2)? as u8 - b'a';
        let to_rank = move_str.chars().nth(3)? as u8 - b'1';

        let from = (from_rank * 8 + from_file) as u8;
        let to = (to_rank * 8 + to_file) as u8;

        let (piece, color) = self.board.get_piece_at(from)?;
        
        // Check if the piece belongs to the side to move
        if color != self.board.side_to_move {
            return None;
        }

        let captured_piece = if let Some((piece, _)) = self.board.get_piece_at(to) {
            Some(piece)
        } else {
            None
        };

        let mut mv = Move::new(from, to, piece);
        mv.captured_piece = captured_piece;

        // Handle promotions
        if move_str.len() == 5 {
            mv.promotion = match move_str.chars().nth(4)? {
                'q' => Some(Piece::Queen),
                'r' => Some(Piece::Rook),
                'b' => Some(Piece::Bishop),
                'n' => Some(Piece::Knight),
                _ => None,
            };
        }

        // Validate the move
        if self.move_generator.is_move_valid(&self.board, &mv) {
            Some(mv)
        } else {
            None
        }
    }

    fn handle_go(&mut self, parts: &[&str]) -> String {
        // Parse search parameters
        let mut max_time = Duration::from_secs(5); // Default 5 seconds
        let mut max_depth = 4; // Default depth

        for i in 0..parts.len() {
            match parts[i] {
                "wtime" => {
                    if let Some(time) = parts.get(i + 1).and_then(|s| s.parse::<u64>().ok()) {
                        max_time = Duration::from_millis(time / 20); // Use 1/20th of the remaining time
                    }
                }
                "btime" => {
                    if let Some(time) = parts.get(i + 1).and_then(|s| s.parse::<u64>().ok()) {
                        max_time = Duration::from_millis(time / 20);
                    }
                }
                "movetime" => {
                    if let Some(time) = parts.get(i + 1).and_then(|s| s.parse::<u64>().ok()) {
                        max_time = Duration::from_millis(time);
                    }
                }
                "depth" => {
                    if let Some(depth) = parts.get(i + 1).and_then(|s| s.parse::<u32>().ok()) {
                        max_depth = depth;
                    }
                }
                _ => {}
            }
        }

        // Configure search parameters
        self.search.set_max_depth(max_depth);
        self.search.set_max_time(max_time.as_secs());

        // Use the search engine to find the best move
        if let Some(best_move) = self.search.find_best_move(&self.board) {
            format!("bestmove {}\n", self.format_move(&best_move))
        } else {
            "bestmove (none)\n".to_string()
        }
    }

    fn format_move(&self, mv: &Move) -> String {
        let from_file = (mv.from % 8) as u8;
        let from_rank = (mv.from / 8) as u8;
        let to_file = (mv.to % 8) as u8;
        let to_rank = (mv.to / 8) as u8;

        let mut result = String::new();
        result.push((b'a' + from_file) as char);
        result.push((b'1' + from_rank) as char);
        result.push((b'a' + to_file) as char);
        result.push((b'1' + to_rank) as char);

        if let Some(promotion) = mv.promotion {
            result.push(match promotion {
                Piece::Queen => 'q',
                Piece::Rook => 'r',
                Piece::Bishop => 'b',
                Piece::Knight => 'n',
                _ => ' ',
            });
        }

        result
    }
}

#[derive(Debug, Clone, Copy)]
enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

impl FromStr for Square {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            return Err("Invalid square format".to_string());
        }
        
        let file = match s.chars().nth(0).unwrap() {
            'a' => 0, 'b' => 1, 'c' => 2, 'd' => 3,
            'e' => 4, 'f' => 5, 'g' => 6, 'h' => 7,
            _ => return Err("Invalid file".to_string()),
        };
        
        let rank = match s.chars().nth(1).unwrap() {
            '1' => 0, '2' => 1, '3' => 2, '4' => 3,
            '5' => 4, '6' => 5, '7' => 6, '8' => 7,
            _ => return Err("Invalid rank".to_string()),
        };
        
        Ok(Square::from_u8(rank * 8 + file))
    }
}

impl Square {
    fn from_u8(value: u8) -> Self {
        unsafe { std::mem::transmute_copy(&value) }
    }
} 