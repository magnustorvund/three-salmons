use crate::board::{Board, Color, Piece};
use crate::evaluation::Evaluator;
use crate::movegen::{Move, MoveGenerator};
use crate::transposition::{NodeType, TranspositionEntry, TranspositionTable};
use std::time::{Duration, Instant};

pub struct Search {
    evaluator: Evaluator,
    move_generator: MoveGenerator,
    transposition_table: TranspositionTable,
    max_depth: u32,
    max_time: Duration,
    nodes_searched: u64,
    start_time: Instant,
}

impl Search {
    pub fn new() -> Self {
        Self {
            evaluator: Evaluator::new(),
            move_generator: MoveGenerator::new(),
            transposition_table: TranspositionTable::new(1_000_000), // 1 million entries
            max_depth: 4,
            max_time: Duration::from_secs(5),
            nodes_searched: 0,
            start_time: Instant::now(),
        }
    }

    pub fn find_best_move(&mut self, board: &Board) -> Option<Move> {
        self.nodes_searched = 0;
        self.start_time = Instant::now();

        let mut best_move = None;
        let mut best_score = i32::MIN;
        let mut alpha = i32::MIN;
        let beta = i32::MAX;

        // Get all legal moves and order them
        let mut moves = self.move_generator.generate_moves(board);
        self.order_moves(&mut moves, board, None);

        // Try each move and evaluate the position
        for mv in moves {
            let mut board_copy = board.clone();
            board_copy.make_move(mv);

            // Evaluate the position after the move
            let score = -self.negamax(&board_copy, self.max_depth - 1, -beta, -alpha);

            if score > best_score {
                best_score = score;
                best_move = Some(mv);
            }

            alpha = alpha.max(score);

            // Check if we've exceeded the time limit
            if self.start_time.elapsed() > self.max_time {
                break;
            }
        }

        best_move
    }

    fn negamax(&mut self, board: &Board, depth: u32, alpha: i32, beta: i32) -> i32 {
        self.nodes_searched += 1;

        // Check transposition table
        let hash = self.get_position_hash(board);
        if let Some(score) = self.transposition_table.probe(hash, depth, alpha, beta) {
            return score;
        }

        // Check if we've reached the maximum depth or if the game is over
        if depth == 0 || self.is_game_over(board) {
            return self.quiescence_search(board, alpha, beta);
        }

        // Get all legal moves and order them
        let mut moves = self.move_generator.generate_moves(board);
        if moves.is_empty() {
            return self.evaluator.evaluate(board);
        }

        self.order_moves(&mut moves, board, self.transposition_table.get_best_move(hash));

        let mut alpha = alpha;
        let mut best_score = i32::MIN;
        let mut best_move = None;

        for mv in moves {
            let mut board_copy = board.clone();
            board_copy.make_move(mv);

            // Recursively evaluate the position
            let score = -self.negamax(&board_copy, depth - 1, -beta, -alpha);

            if score > best_score {
                best_score = score;
                best_move = Some(mv);
            }

            alpha = alpha.max(score);

            // Alpha-beta pruning
            if alpha >= beta {
                break;
            }

            // Check if we've exceeded the time limit
            if self.start_time.elapsed() > self.max_time {
                break;
            }
        }

        // Store in transposition table
        let node_type = if best_score <= alpha {
            NodeType::UpperBound
        } else if best_score >= beta {
            NodeType::LowerBound
        } else {
            NodeType::Exact
        };

        let entry = TranspositionEntry {
            hash,
            depth,
            score: best_score,
            node_type,
            best_move: best_move.map(|mv| self.move_to_u64(mv)),
        };
        self.transposition_table.store(hash, entry);

        best_score
    }

    fn quiescence_search(&mut self, board: &Board, mut alpha: i32, beta: i32) -> i32 {
        self.nodes_searched += 1;

        let stand_pat = self.evaluator.evaluate(board);
        if stand_pat >= beta {
            return beta;
        }
        if alpha < stand_pat {
            alpha = stand_pat;
        }

        // Only consider captures and promotions
        let mut moves = self.move_generator.generate_moves(board)
            .into_iter()
            .filter(|mv| mv.captured_piece.is_some() || mv.promotion.is_some())
            .collect::<Vec<_>>();

        if moves.is_empty() {
            return stand_pat;
        }

        self.order_moves(&mut moves, board, None);

        for mv in moves {
            let mut board_copy = board.clone();
            board_copy.make_move(mv);

            let score = -self.quiescence_search(&board_copy, -beta, -alpha);

            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }

    fn order_moves(&self, moves: &mut Vec<Move>, board: &Board, hash_move: Option<u64>) {
        moves.sort_by(|a, b| {
            // First try the move from the transposition table
            if let Some(hash) = hash_move {
                if self.move_to_u64(*a) == hash {
                    return std::cmp::Ordering::Less;
                }
                if self.move_to_u64(*b) == hash {
                    return std::cmp::Ordering::Greater;
                }
            }

            // Then try captures (MVV-LVA)
            let a_capture = a.captured_piece.map(|p| self.get_piece_value(p)).unwrap_or(0);
            let b_capture = b.captured_piece.map(|p| self.get_piece_value(p)).unwrap_or(0);
            if a_capture != b_capture {
                return b_capture.cmp(&a_capture);
            }

            // Then try promotions
            let a_promo = a.promotion.map(|p| self.get_piece_value(p)).unwrap_or(0);
            let b_promo = b.promotion.map(|p| self.get_piece_value(p)).unwrap_or(0);
            if a_promo != b_promo {
                return b_promo.cmp(&a_promo);
            }

            // Then try killer moves (TODO: implement killer move heuristic)
            // Finally, try history heuristic (TODO: implement history heuristic)

            std::cmp::Ordering::Equal
        });
    }

    fn get_piece_value(&self, piece: Piece) -> i32 {
        match piece {
            Piece::Pawn => 100,
            Piece::Knight => 320,
            Piece::Bishop => 330,
            Piece::Rook => 500,
            Piece::Queen => 900,
            Piece::King => 20000,
        }
    }

    fn get_position_hash(&self, board: &Board) -> u64 {
        // TODO: Implement Zobrist hashing for more accurate position hashing
        // For now, use a simple hash based on piece positions
        let mut hash: u64 = 0;
        for square in 0..64 {
            if let Some((piece, color)) = board.get_piece_at(square as u8) {
                let piece_value = self.get_piece_value(piece) as i64;
                let color_value = if color == Color::White { 1 } else { -1 };
                hash = hash.wrapping_add((piece_value * color_value) as u64);
            }
        }
        hash
    }

    fn move_to_u64(&self, mv: Move) -> u64 {
        // Pack move into a u64: from (6 bits) | to (6 bits) | piece (3 bits) | captured_piece (3 bits) | promotion (3 bits)
        let from = mv.from as u64;
        let to = mv.to as u64;
        let piece = mv.piece as u64;
        let captured = mv.captured_piece.map(|p| p as u64).unwrap_or(0);
        let promo = mv.promotion.map(|p| p as u64).unwrap_or(0);
        
        (from) | (to << 6) | (piece << 12) | (captured << 15) | (promo << 18)
    }

    fn is_game_over(&self, board: &Board) -> bool {
        let moves = self.move_generator.generate_moves(board);
        moves.is_empty()
    }

    pub fn set_max_depth(&mut self, depth: u32) {
        self.max_depth = depth;
    }

    pub fn set_max_time(&mut self, seconds: u64) {
        self.max_time = Duration::from_secs(seconds);
    }

    pub fn get_nodes_searched(&self) -> u64 {
        self.nodes_searched
    }
} 