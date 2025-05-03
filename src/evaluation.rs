use crate::board::{Board, Color, Piece};
use crate::movegen::MoveGenerator;

pub struct Evaluator {
    // Piece values
    pub pawn_value: i32,
    pub knight_value: i32,
    pub bishop_value: i32,
    pub rook_value: i32,
    pub queen_value: i32,
    pub king_value: i32,

    // Positional bonuses
    pub pawn_position_bonus: [[i32; 8]; 8],
    pub knight_position_bonus: [[i32; 8]; 8],
    pub bishop_position_bonus: [[i32; 8]; 8],
    pub rook_position_bonus: [[i32; 8]; 8],
    pub queen_position_bonus: [[i32; 8]; 8],
    pub king_position_bonus: [[i32; 8]; 8],
    pub king_endgame_position_bonus: [[i32; 8]; 8],

    // Mobility weights
    pub pawn_mobility_weight: i32,
    pub knight_mobility_weight: i32,
    pub bishop_mobility_weight: i32,
    pub rook_mobility_weight: i32,
    pub queen_mobility_weight: i32,
    pub king_mobility_weight: i32,

    // Pawn structure weights
    pub doubled_pawn_penalty: i32,
    pub isolated_pawn_penalty: i32,
    pub passed_pawn_bonus: i32,
    pub connected_pawn_bonus: i32,

    // King safety weights
    pub pawn_shield_bonus: i32,
    pub open_file_penalty: i32,
    pub semi_open_file_penalty: i32,
    pub king_attack_bonus: i32,
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
            pawn_value: 100,
            knight_value: 320,
            bishop_value: 330,
            rook_value: 500,
            queen_value: 900,
            king_value: 20000,

            // Pawn position bonuses (encourages central control and advancement)
            pawn_position_bonus: [
                [0, 0, 0, 0, 0, 0, 0, 0],
                [50, 50, 50, 50, 50, 50, 50, 50],
                [10, 10, 20, 30, 30, 20, 10, 10],
                [5, 5, 10, 25, 25, 10, 5, 5],
                [0, 0, 0, 20, 20, 0, 0, 0],
                [5, -5, -10, 0, 0, -10, -5, 5],
                [5, 10, 10, -20, -20, 10, 10, 5],
                [0, 0, 0, 0, 0, 0, 0, 0],
            ],

            // Knight position bonuses (encourages central control)
            knight_position_bonus: [
                [-50, -40, -30, -30, -30, -30, -40, -50],
                [-40, -20, 0, 0, 0, 0, -20, -40],
                [-30, 0, 10, 15, 15, 10, 0, -30],
                [-30, 5, 15, 20, 20, 15, 5, -30],
                [-30, 0, 15, 20, 20, 15, 0, -30],
                [-30, 5, 10, 15, 15, 10, 5, -30],
                [-40, -20, 0, 5, 5, 0, -20, -40],
                [-50, -40, -30, -30, -30, -30, -40, -50],
            ],

            // Bishop position bonuses (encourages central control and long diagonals)
            bishop_position_bonus: [
                [-20, -10, -10, -10, -10, -10, -10, -20],
                [-10, 0, 0, 0, 0, 0, 0, -10],
                [-10, 0, 5, 10, 10, 5, 0, -10],
                [-10, 5, 5, 10, 10, 5, 5, -10],
                [-10, 0, 10, 10, 10, 10, 0, -10],
                [-10, 10, 10, 10, 10, 10, 10, -10],
                [-10, 5, 0, 0, 0, 0, 5, -10],
                [-20, -10, -10, -10, -10, -10, -10, -20],
            ],

            // Rook position bonuses (encourages open files and central control)
            rook_position_bonus: [
                [0, 0, 0, 0, 0, 0, 0, 0],
                [5, 10, 10, 10, 10, 10, 10, 5],
                [-5, 0, 0, 0, 0, 0, 0, -5],
                [-5, 0, 0, 0, 0, 0, 0, -5],
                [-5, 0, 0, 0, 0, 0, 0, -5],
                [-5, 0, 0, 0, 0, 0, 0, -5],
                [-5, 0, 0, 0, 0, 0, 0, -5],
                [0, 0, 0, 5, 5, 0, 0, 0],
            ],

            // Queen position bonuses (encourages central control and mobility)
            queen_position_bonus: [
                [-20, -10, -10, -5, -5, -10, -10, -20],
                [-10, 0, 0, 0, 0, 0, 0, -10],
                [-10, 0, 5, 5, 5, 5, 0, -10],
                [-5, 0, 5, 5, 5, 5, 0, -5],
                [0, 0, 5, 5, 5, 5, 0, -5],
                [-10, 5, 5, 5, 5, 5, 0, -10],
                [-10, 0, 5, 0, 0, 0, 0, -10],
                [-20, -10, -10, -5, -5, -10, -10, -20],
            ],

            // King position bonuses (encourages safety in opening/middlegame)
            king_position_bonus: [
                [-30, -40, -40, -50, -50, -40, -40, -30],
                [-30, -40, -40, -50, -50, -40, -40, -30],
                [-30, -40, -40, -50, -50, -40, -40, -30],
                [-30, -40, -40, -50, -50, -40, -40, -30],
                [-20, -30, -30, -40, -40, -30, -30, -20],
                [-10, -20, -20, -20, -20, -20, -20, -10],
                [20, 20, 0, 0, 0, 0, 20, 20],
                [20, 30, 10, 0, 0, 10, 30, 20],
            ],

            // King position bonuses for endgame (encourages centralization)
            king_endgame_position_bonus: [
                [-50, -40, -30, -20, -20, -30, -40, -50],
                [-30, -20, -10, 0, 0, -10, -20, -30],
                [-30, -10, 20, 30, 30, 20, -10, -30],
                [-30, -10, 30, 40, 40, 30, -10, -30],
                [-30, -10, 30, 40, 40, 30, -10, -30],
                [-30, -10, 20, 30, 30, 20, -10, -30],
                [-30, -30, 0, 0, 0, 0, -30, -30],
                [-50, -30, -30, -30, -30, -30, -30, -50],
            ],

            // Mobility weights
            pawn_mobility_weight: 1,
            knight_mobility_weight: 2,
            bishop_mobility_weight: 3,
            rook_mobility_weight: 2,
            queen_mobility_weight: 1,
            king_mobility_weight: 1,

            // Pawn structure weights
            doubled_pawn_penalty: -10,
            isolated_pawn_penalty: -20,
            passed_pawn_bonus: 20,
            connected_pawn_bonus: 10,

            // King safety weights
            pawn_shield_bonus: 5,
            open_file_penalty: -15,
            semi_open_file_penalty: -10,
            king_attack_bonus: 5,
        }
    }

    pub fn evaluate(&self, board: &Board) -> i32 {
        let mut score = 0;
        let is_endgame = self.is_endgame(board);

        // Evaluate material and position for each piece
        for square in 0..64 {
            if let Some((piece, color)) = board.get_piece_at(square as u8) {
                let rank = (square / 8) as usize;
                let file = (square % 8) as usize;
                let value = self.get_piece_value(piece, rank, file, is_endgame);
                score += if color == Color::White { value } else { -value };
            }
        }

        // Add mobility bonus
        score += self.evaluate_mobility(board);

        // Add pawn structure bonus
        score += self.evaluate_pawn_structure(board);

        // Add king safety bonus
        score += self.evaluate_king_safety(board);

        score
    }

    fn get_piece_value(&self, piece: Piece, rank: usize, file: usize, is_endgame: bool) -> i32 {
        let base_value = match piece {
            Piece::Pawn => self.pawn_value,
            Piece::Knight => self.knight_value,
            Piece::Bishop => self.bishop_value,
            Piece::Rook => self.rook_value,
            Piece::Queen => self.queen_value,
            Piece::King => self.king_value,
        };

        let position_bonus = match piece {
            Piece::Pawn => self.pawn_position_bonus[rank][file],
            Piece::Knight => self.knight_position_bonus[rank][file],
            Piece::Bishop => self.bishop_position_bonus[rank][file],
            Piece::Rook => self.rook_position_bonus[rank][file],
            Piece::Queen => self.queen_position_bonus[rank][file],
            Piece::King => if is_endgame {
                self.king_endgame_position_bonus[rank][file]
            } else {
                self.king_position_bonus[rank][file]
            },
        };

        base_value + position_bonus
    }

    fn is_endgame(&self, board: &Board) -> bool {
        // Count major pieces (queens and rooks)
        let mut major_pieces = 0;
        for square in 0..64 {
            if let Some((piece, _)) = board.get_piece_at(square as u8) {
                if piece == Piece::Queen || piece == Piece::Rook {
                    major_pieces += 1;
                }
            }
        }
        major_pieces <= 2
    }

    fn evaluate_mobility(&self, board: &Board) -> i32 {
        let mut score = 0;
        let move_generator = MoveGenerator::new();
        let moves = move_generator.generate_moves(board);

        // Count moves for each piece type
        let mut piece_moves = [0; 6]; // Pawn, Knight, Bishop, Rook, Queen, King
        for mv in moves {
            let piece_index = match mv.piece {
                Piece::Pawn => 0,
                Piece::Knight => 1,
                Piece::Bishop => 2,
                Piece::Rook => 3,
                Piece::Queen => 4,
                Piece::King => 5,
            };
            piece_moves[piece_index] += 1;
        }

        // Apply mobility weights
        score += piece_moves[0] * self.pawn_mobility_weight;
        score += piece_moves[1] * self.knight_mobility_weight;
        score += piece_moves[2] * self.bishop_mobility_weight;
        score += piece_moves[3] * self.rook_mobility_weight;
        score += piece_moves[4] * self.queen_mobility_weight;
        score += piece_moves[5] * self.king_mobility_weight;

        // Adjust for color
        if board.side_to_move == Color::Black {
            score = -score;
        }

        score
    }

    fn evaluate_pawn_structure(&self, board: &Board) -> i32 {
        let mut score = 0;
        let mut white_pawns = [0; 8]; // Count pawns per file
        let mut black_pawns = [0; 8];

        // Count pawns on each file
        for square in 0..64 {
            if let Some((piece, color)) = board.get_piece_at(square as u8) {
                if piece == Piece::Pawn {
                    let file = (square % 8) as usize;
                    match color {
                        Color::White => white_pawns[file] += 1,
                        Color::Black => black_pawns[file] += 1,
                    }
                }
            }
        }

        // Evaluate pawn structure for both colors
        score += self.evaluate_pawn_structure_for_color(white_pawns, true);
        score -= self.evaluate_pawn_structure_for_color(black_pawns, false);

        score
    }

    fn evaluate_pawn_structure_for_color(&self, pawns: [i32; 8], is_white: bool) -> i32 {
        let mut score = 0;

        // Check for doubled pawns
        for &count in pawns.iter() {
            if count > 1 {
                score += self.doubled_pawn_penalty * (count - 1);
            }
        }

        // Check for isolated pawns
        for file in 0..8 {
            if pawns[file] > 0 {
                let has_neighbor = (file > 0 && pawns[file - 1] > 0) || 
                                 (file < 7 && pawns[file + 1] > 0);
                if !has_neighbor {
                    score += self.isolated_pawn_penalty;
                }
            }
        }

        // Check for passed pawns
        for file in 0..8 {
            if pawns[file] > 0 {
                let is_passed = if is_white {
                    // For white pawns, check if there are no black pawns on adjacent files
                    (file == 0 || pawns[file - 1] == 0) && 
                    (file == 7 || pawns[file + 1] == 0)
                } else {
                    // For black pawns, check if there are no white pawns on adjacent files
                    (file == 0 || pawns[file - 1] == 0) && 
                    (file == 7 || pawns[file + 1] == 0)
                };
                if is_passed {
                    score += self.passed_pawn_bonus;
                }
            }
        }

        // Check for connected pawns
        for file in 0..7 {
            if pawns[file] > 0 && pawns[file + 1] > 0 {
                score += self.connected_pawn_bonus;
            }
        }

        score
    }

    fn evaluate_king_safety(&self, board: &Board) -> i32 {
        let mut score = 0;

        // Find king positions
        let (white_king_square, black_king_square) = self.find_kings(board);

        // Evaluate pawn shield
        score += self.evaluate_pawn_shield(board, white_king_square, true);
        score -= self.evaluate_pawn_shield(board, black_king_square, false);

        // Evaluate open files near king
        score += self.evaluate_open_files(board, white_king_square, true);
        score -= self.evaluate_open_files(board, black_king_square, false);

        score
    }

    fn find_kings(&self, board: &Board) -> (Option<u8>, Option<u8>) {
        let mut white_king = None;
        let mut black_king = None;

        for square in 0..64 {
            if let Some((piece, color)) = board.get_piece_at(square as u8) {
                if piece == Piece::King {
                    match color {
                        Color::White => white_king = Some(square as u8),
                        Color::Black => black_king = Some(square as u8),
                    }
                }
            }
        }

        (white_king, black_king)
    }

    fn evaluate_pawn_shield(&self, board: &Board, king_square: Option<u8>, is_white: bool) -> i32 {
        let mut score = 0;

        if let Some(square) = king_square {
            let rank = square / 8;
            let file = square % 8;

            // Check pawns in front of the king
            let shield_rank = if is_white { rank + 1 } else { rank - 1 };
            if shield_rank < 8 {
                for file_offset in -1..=1 {
                    let shield_file = file as i8 + file_offset;
                    if shield_file >= 0 && shield_file < 8 {
                        let shield_square = (shield_rank * 8 + shield_file as u8) as u8;
                        if let Some((piece, color)) = board.get_piece_at(shield_square) {
                            if piece == Piece::Pawn && color == (if is_white { Color::White } else { Color::Black }) {
                                score += self.pawn_shield_bonus;
                            }
                        }
                    }
                }
            }
        }

        score
    }

    fn evaluate_open_files(&self, board: &Board, king_square: Option<u8>, is_white: bool) -> i32 {
        let mut score = 0;

        if let Some(square) = king_square {
            let file = square % 8;

            // Check if the file is open or semi-open
            let mut has_own_pawn = false;
            let mut has_opponent_pawn = false;

            for rank in 0..8 {
                let square = rank * 8 + file;
                if let Some((piece, color)) = board.get_piece_at(square as u8) {
                    if piece == Piece::Pawn {
                        if color == (if is_white { Color::White } else { Color::Black }) {
                            has_own_pawn = true;
                        } else {
                            has_opponent_pawn = true;
                        }
                    }
                }
            }

            if !has_own_pawn && !has_opponent_pawn {
                score += self.open_file_penalty;
            } else if !has_own_pawn {
                score += self.semi_open_file_penalty;
            }
        }

        score
    }
} 