use crate::board::{Board, Color, Piece};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    pub piece: Piece,
    pub captured_piece: Option<Piece>,
    pub promotion: Option<Piece>,
    pub is_en_passant: bool,
    pub is_castling: bool,
    pub castling_rook_from: Option<u8>,
    pub castling_rook_to: Option<u8>,
}

impl Move {
    pub fn new(from: u8, to: u8, piece: Piece) -> Self {
        Self {
            from,
            to,
            piece,
            captured_piece: None,
            promotion: None,
            is_en_passant: false,
            is_castling: false,
            castling_rook_from: None,
            castling_rook_to: None,
        }
    }

    pub fn new_en_passant(from: u8, to: u8, piece: Piece) -> Self {
        Self {
            from,
            to,
            piece,
            captured_piece: Some(Piece::Pawn),
            promotion: None,
            is_en_passant: true,
            is_castling: false,
            castling_rook_from: None,
            castling_rook_to: None,
        }
    }

    pub fn new_castling(from: u8, to: u8, rook_from: u8, rook_to: u8) -> Self {
        Self {
            from,
            to,
            piece: Piece::King,
            captured_piece: None,
            promotion: None,
            is_en_passant: false,
            is_castling: true,
            castling_rook_from: Some(rook_from),
            castling_rook_to: Some(rook_to),
        }
    }

    pub fn new_promotion(from: u8, to: u8, promotion: Piece) -> Self {
        Self {
            from,
            to,
            piece: Piece::Pawn,
            captured_piece: None,
            promotion: Some(promotion),
            is_en_passant: false,
            is_castling: false,
            castling_rook_from: None,
            castling_rook_to: None,
        }
    }

    pub fn new_promotion_capture(from: u8, to: u8, captured_piece: Piece, promotion: Piece) -> Self {
        Self {
            from,
            to,
            piece: Piece::Pawn,
            captured_piece: Some(captured_piece),
            promotion: Some(promotion),
            is_en_passant: false,
            is_castling: false,
            castling_rook_from: None,
            castling_rook_to: None,
        }
    }
}

pub struct MoveGenerator {
    pub bishop_magics: [u64; 64],
    pub rook_magics: [u64; 64],
}

impl MoveGenerator {
    pub fn new() -> Self {
        Self {
            bishop_magics: [0; 64],
            rook_magics: [0; 64],
        }
    }

    fn get_bishop_attacks(&self, square: u8, occupied: u64) -> u64 {
        let mut attacks = 0u64;
        let rank = (square / 8) as i8;
        let file = (square % 8) as i8;
        
        // Generate attacks in all four diagonal directions
        for &(dr, df) in &[(1, 1), (1, -1), (-1, 1), (-1, -1)] {
            let mut r = rank + dr;
            let mut f = file + df;
            while r >= 0 && r < 8 && f >= 0 && f < 8 {
                let target = (r * 8 + f) as u8;
                let target_mask = 1u64 << target;
                attacks |= target_mask;
                if (occupied & target_mask) != 0 {
                    break;
                }
                r += dr;
                f += df;
            }
        }
        attacks
    }

    fn get_rook_attacks(&self, square: u8, occupied: u64) -> u64 {
        let mut attacks = 0u64;
        let rank = (square / 8) as i8;
        let file = (square % 8) as i8;
        
        // Generate attacks in all four orthogonal directions
        for &(dr, df) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let mut r = rank + dr;
            let mut f = file + df;
            while r >= 0 && r < 8 && f >= 0 && f < 8 {
                let target = (r * 8 + f) as u8;
                let target_mask = 1u64 << target;
                attacks |= target_mask;
                if (occupied & target_mask) != 0 {
                    break;
                }
                r += dr;
                f += df;
            }
        }
        attacks
    }

    pub fn is_square_under_attack(&self, board: &Board, square: u8, attacker_color: Color) -> bool {
        let square_mask = 1u64 << square;
        let square_rank = square / 8;
        let square_file = square % 8;
        let attacker_pieces = match attacker_color {
            Color::White => &board.white_pieces,
            Color::Black => &board.black_pieces,
        };
        let defender_pieces = match attacker_color {
            Color::White => &board.black_pieces,
            Color::Black => &board.white_pieces,
        };

        // Check pawn attacks
        let pawn_attacks = match attacker_color {
            Color::White => {
                let mut attacks = 0u64;
                if square_rank < 7 {
                    if square_file > 0 {
                        attacks |= 1u64 << (square + 7);
                    }
                    if square_file < 7 {
                        attacks |= 1u64 << (square + 9);
                    }
                }
                attacks
            }
            Color::Black => {
                let mut attacks = 0u64;
                if square_rank > 0 {
                    if square_file > 0 {
                        attacks |= 1u64 << (square - 9);
                    }
                    if square_file < 7 {
                        attacks |= 1u64 << (square - 7);
                    }
                }
                attacks
            }
        };
        if (pawn_attacks & attacker_pieces[0]) != 0 {
            return true;
        }

        // Check knight attacks
        let knight_attacks = {
            let mut attacks = 0u64;
            let knight_moves = [
                (-2, -1), (-2, 1), (-1, -2), (-1, 2),
                (1, -2), (1, 2), (2, -1), (2, 1)
            ];
            for &(dr, df) in &knight_moves {
                let rank = square_rank as i8 + dr;
                let file = square_file as i8 + df;
                if rank >= 0 && rank < 8 && file >= 0 && file < 8 {
                    attacks |= 1u64 << (rank * 8 + file);
                }
            }
            attacks
        };
        if (knight_attacks & attacker_pieces[1]) != 0 {
            return true;
        }

        // Check king attacks
        let king_attacks = {
            let mut attacks = 0u64;
            let king_moves = [
                (-1, -1), (-1, 0), (-1, 1),
                (0, -1), (0, 1),
                (1, -1), (1, 0), (1, 1)
            ];
            for &(dr, df) in &king_moves {
                let rank = square_rank as i8 + dr;
                let file = square_file as i8 + df;
                if rank >= 0 && rank < 8 && file >= 0 && file < 8 {
                    attacks |= 1u64 << (rank * 8 + file);
                }
            }
            attacks
        };
        if (king_attacks & attacker_pieces[5]) != 0 {
            return true;
        }

        // Check bishop/queen attacks (diagonals)
        for &(dr, df) in &[(1, 1), (1, -1), (-1, 1), (-1, -1)] {
            let mut rank = square_rank as i8;
            let mut file = square_file as i8;
            loop {
                rank += dr;
                file += df;
                if rank < 0 || rank >= 8 || file < 0 || file >= 8 {
                    break;
                }
                let target = 1u64 << (rank * 8 + file);
                // If we hit a piece, check if it's an attacker's bishop or queen
                if (attacker_pieces[2] | attacker_pieces[4]) & target != 0 {
                    return true;
                }
                // If we hit any other piece, stop looking in this direction
                if defender_pieces.iter().any(|&bb| bb & target != 0) ||
                   attacker_pieces.iter().any(|&bb| bb & target != 0) {
                    break;
                }
            }
        }

        // Check rook/queen attacks (orthogonals)
        for &(dr, df) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let mut rank = square_rank as i8;
            let mut file = square_file as i8;
            loop {
                rank += dr;
                file += df;
                if rank < 0 || rank >= 8 || file < 0 || file >= 8 {
                    break;
                }
                let target = 1u64 << (rank * 8 + file);
                // If we hit a piece, check if it's an attacker's rook or queen
                if (attacker_pieces[3] | attacker_pieces[4]) & target != 0 {
                    return true;
                }
                // If we hit any other piece, stop looking in this direction
                if defender_pieces.iter().any(|&bb| bb & target != 0) ||
                   attacker_pieces.iter().any(|&bb| bb & target != 0) {
                    break;
                }
            }
        }

        false
    }

    pub fn is_king_in_check(&self, board: &Board, color: Color) -> bool {
        let king_pieces = match color {
            Color::White => &board.white_pieces[5],
            Color::Black => &board.black_pieces[5],
        };
        
        // Find the king's square
        let mut king_square = None;
        for square in 0..64 {
            if (king_pieces >> square) & 1 != 0 {
                king_square = Some(square as u8);
                break;
            }
        }
        
        if let Some(king_square) = king_square {
            self.is_square_under_attack(board, king_square, color.opposite())
        } else {
            false  // No king found (shouldn't happen in a valid position)
        }
    }

    pub fn is_move_valid(&self, board: &Board, mv: &Move) -> bool {
        // First verify that the piece at the source square matches the move's piece and color
        let from_mask = 1u64 << mv.from;
        let pieces = if board.side_to_move == Color::White {
            &board.white_pieces
        } else {
            &board.black_pieces
        };
        
        // Check if the piece at the source square matches the move's piece
        let piece_index = match mv.piece {
            Piece::Pawn => 0,
            Piece::Knight => 1,
            Piece::Bishop => 2,
            Piece::Rook => 3,
            Piece::Queen => 4,
            Piece::King => 5,
        };
        if (pieces[piece_index] & from_mask) == 0 {
            return false;
        }

        // Check if the destination square is occupied by our own piece
        let to_mask = 1u64 << mv.to;
        if pieces.iter().any(|&p| (p & to_mask) != 0) {
            return false;
        }

        // Check if the move is legal for the piece type
        let from_rank = (mv.from / 8) as i8;
        let from_file = (mv.from % 8) as i8;
        let to_rank = (mv.to / 8) as i8;
        let to_file = (mv.to % 8) as i8;

        let is_legal = match mv.piece {
            Piece::Pawn => {
                // Pawns can only move forward
                let direction = if board.side_to_move == Color::White { 1 } else { -1 };
                let rank_diff = to_rank - from_rank;
                let file_diff = to_file - from_file;

                // Single push
                if file_diff == 0 && rank_diff == direction {
                    // Check if the destination square is empty
                    let to_mask = 1u64 << mv.to;
                    board.white_pieces[0..6].iter().chain(board.black_pieces[0..6].iter())
                        .all(|&p| (p & to_mask) == 0)
                }
                // Double push from starting rank
                else if file_diff == 0 && rank_diff == 2 * direction && 
                        ((board.side_to_move == Color::White && from_rank == 1) || 
                         (board.side_to_move == Color::Black && from_rank == 6)) {
                    // Check if both squares are empty
                    let to_mask = 1u64 << mv.to;
                    let intermediate = if board.side_to_move == Color::White {
                        mv.from + 8
                    } else {
                        mv.from - 8
                    };
                    let intermediate_mask = 1u64 << intermediate;
                    board.white_pieces[0..6].iter().chain(board.black_pieces[0..6].iter())
                        .all(|&p| (p & to_mask) == 0 && (p & intermediate_mask) == 0)
                }
                // Capture
                else if file_diff.abs() == 1 && rank_diff == direction {
                    // Check if there's a piece to capture
                    let to_mask = 1u64 << mv.to;
                    let opponent_pieces = if board.side_to_move == Color::White {
                        &board.black_pieces
                    } else {
                        &board.white_pieces
                    };
                    if mv.is_en_passant {
                        if board.en_passant_square != Some(mv.to) {
                            false
                        } else {
                            // Check if there's a pawn to capture
                            let captured_pawn_square = if board.side_to_move == Color::White {
                                mv.to - 8
                            } else {
                                mv.to + 8
                            };
                            let captured_pawn_mask = 1u64 << captured_pawn_square;
                            opponent_pieces[0] & captured_pawn_mask != 0
                        }
                    } else {
                        opponent_pieces.iter().any(|&p| (p & to_mask) != 0)
                    }
                }
                else {
                    false
                }
            }
            Piece::Knight => {
                let rank_diff = (to_rank - from_rank).abs();
                let file_diff = (to_file - from_file).abs();
                (rank_diff == 2 && file_diff == 1) || (rank_diff == 1 && file_diff == 2)
            }
            Piece::Bishop => {
                let rank_diff = (to_rank - from_rank).abs();
                let file_diff = (to_file - from_file).abs();
                if rank_diff != file_diff {
                    false
                } else {
                    // Check if the path is clear
                    let rank_step = if to_rank > from_rank { 1 } else { -1 };
                    let file_step = if to_file > from_file { 1 } else { -1 };
                    let mut rank = from_rank + rank_step;
                    let mut file = from_file + file_step;
                    while rank != to_rank && file != to_file {
                        let square = (rank * 8 + file) as u8;
                        let square_mask = 1u64 << square;
                        if board.white_pieces[0..6].iter().chain(board.black_pieces[0..6].iter())
                            .any(|&p| (p & square_mask) != 0) {
                            return false;
                        }
                        rank += rank_step;
                        file += file_step;
                    }
                    true
                }
            }
            Piece::Rook => {
                let rank_diff = (to_rank - from_rank).abs();
                let file_diff = (to_file - from_file).abs();
                if rank_diff != 0 && file_diff != 0 {
                    false
                } else {
                    // Check if the path is clear
                    let rank_step = if to_rank > from_rank { 1 } else if to_rank < from_rank { -1 } else { 0 };
                    let file_step = if to_file > from_file { 1 } else if to_file < from_file { -1 } else { 0 };
                    let mut rank = from_rank + rank_step;
                    let mut file = from_file + file_step;
                    while rank != to_rank || file != to_file {
                        let square = (rank * 8 + file) as u8;
                        let square_mask = 1u64 << square;
                        if board.white_pieces[0..6].iter().chain(board.black_pieces[0..6].iter())
                            .any(|&p| (p & square_mask) != 0) {
                            return false;
                        }
                        rank += rank_step;
                        file += file_step;
                    }
                    true
                }
            }
            Piece::Queen => {
                let rank_diff = (to_rank - from_rank).abs();
                let file_diff = (to_file - from_file).abs();
                if rank_diff != 0 && file_diff != 0 && rank_diff != file_diff {
                    false
                } else {
                    // Check if the path is clear
                    let rank_step = if to_rank > from_rank { 1 } else if to_rank < from_rank { -1 } else { 0 };
                    let file_step = if to_file > from_file { 1 } else if to_file < from_file { -1 } else { 0 };
                    let mut rank = from_rank + rank_step;
                    let mut file = from_file + file_step;
                    while rank != to_rank || file != to_file {
                        let square = (rank * 8 + file) as u8;
                        let square_mask = 1u64 << square;
                        if board.white_pieces[0..6].iter().chain(board.black_pieces[0..6].iter())
                            .any(|&p| (p & square_mask) != 0) {
                            return false;
                        }
                        rank += rank_step;
                        file += file_step;
                    }
                    true
                }
            }
            Piece::King => {
                let rank_diff = (to_rank - from_rank).abs();
                let file_diff = (to_file - from_file).abs();
                if mv.is_castling {
                    // Check if castling is still allowed
                    let castling_mask = if board.side_to_move == Color::White {
                        if mv.to > mv.from { 0b0001 } else { 0b0010 }  // White kingside or queenside
                    } else {
                        if mv.to > mv.from { 0b0100 } else { 0b1000 }  // Black kingside or queenside
                    };
                    if board.castling_rights & castling_mask == 0 {
                        false
                    } else {
                        // Check if the path is clear
                        let rank = if board.side_to_move == Color::White { 0 } else { 7 };
                        let (start_file, end_file) = if mv.to > mv.from {
                            (4, 7)  // Kingside
                        } else {
                            (0, 4)  // Queenside
                        };
                        for file in start_file..=end_file {
                            let square = rank * 8 + file;
                            let square_mask = 1u64 << square;
                            if file != start_file && file != end_file &&  // Skip king and rook squares
                                board.white_pieces[0..6].iter().chain(board.black_pieces[0..6].iter())
                                .any(|&p| (p & square_mask) != 0) {
                                return false;
                            }
                        }

                        // Check if any of the squares the king moves through are under attack
                        let attacker_color = board.side_to_move.opposite();
                        for file in if mv.to > mv.from { 4..=6 } else { 2..=4 } {
                            let square = rank * 8 + file;
                            if self.is_square_under_attack(board, square as u8, attacker_color) {
                                return false;
                            }
                        }
                        true
                    }
                } else {
                    rank_diff <= 1 && file_diff <= 1
                }
            }
        };

        if !is_legal {
            return false;
        }

        // Make the move and check if the king is in check
        let mut board_copy = board.clone();
        board_copy.make_move(*mv);
        !self.is_king_in_check(&board_copy, board.side_to_move)
    }

    pub fn generate_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::new();
        let pieces = if board.side_to_move == Color::White {
            &board.white_pieces
        } else {
            &board.black_pieces
        };
        let opponent_pieces = if board.side_to_move == Color::White {
            &board.black_pieces
        } else {
            &board.white_pieces
        };

        // Generate pawn moves
        let pawns = pieces[0];
        for from in 0..64 {
            if (pawns >> from) & 1 != 0 {
                // Single push
                let to = if board.side_to_move == Color::White {
                    (from as i8).checked_add(8).filter(|&x| x < 64 && from / 8 < 7)
                } else {
                    (from as i8).checked_sub(8).filter(|&x| x >= 0 && from / 8 > 0)
                };
                if let Some(to) = to {
                    let to_mask = 1u64 << to;
                    let is_empty = board.white_pieces[0..6].iter().chain(board.black_pieces[0..6].iter())
                        .all(|&p| (p & to_mask) == 0);
                    if is_empty {
                        // Check for promotion
                        if (board.side_to_move == Color::White && to >= 56) ||
                            (board.side_to_move == Color::Black && to < 8) {
                            for promotion in [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight] {
                                let mv = Move::new_promotion(from as u8, to as u8, promotion);
                                // Make the move and check if the king is in check
                                let mut board_copy = board.clone();
                                board_copy.make_move(mv);
                                if !self.is_king_in_check(&board_copy, board.side_to_move) {
                                    moves.push(mv);
                                }
                            }
                        } else {
                            let mv = Move::new(from as u8, to as u8, Piece::Pawn);
                            // Make the move and check if the king is in check
                            let mut board_copy = board.clone();
                            board_copy.make_move(mv);
                            if !self.is_king_in_check(&board_copy, board.side_to_move) {
                                moves.push(mv);
                            }
                        }
                    }
                }

                // Double push
                let to = if board.side_to_move == Color::White {
                    (from as i8).checked_add(16).filter(|&x| x < 64 && from / 8 == 1)
                } else {
                    (from as i8).checked_sub(16).filter(|&x| x >= 0 && from / 8 == 6)
                };
                if let Some(to) = to {
                    let intermediate = if board.side_to_move == Color::White {
                        from + 8
                    } else {
                        from - 8
                    };
                    let to_mask = 1u64 << to;
                    let intermediate_mask = 1u64 << intermediate;
                    let is_empty = board.white_pieces[0..6].iter().chain(board.black_pieces[0..6].iter())
                        .all(|&p| (p & to_mask) == 0) &&
                        board.white_pieces[0..6].iter().chain(board.black_pieces[0..6].iter())
                        .all(|&p| (p & intermediate_mask) == 0);
                    if is_empty {
                        let mv = Move::new(from as u8, to as u8, Piece::Pawn);
                        // Make the move and check if the king is in check
                        let mut board_copy = board.clone();
                        board_copy.make_move(mv);
                        if !self.is_king_in_check(&board_copy, board.side_to_move) {
                            moves.push(mv);
                        }
                    }
                }

                // Captures
                let from_rank = (from / 8) as i8;
                let from_file = (from % 8) as i8;
                let capture_squares = if board.side_to_move == Color::White {
                    [
                        (from_rank + 1, from_file - 1),
                        (from_rank + 1, from_file + 1),
                    ]
                } else {
                    [
                        (from_rank - 1, from_file - 1),
                        (from_rank - 1, from_file + 1),
                    ]
                };
                for &(rank, file) in &capture_squares {
                    if rank >= 0 && rank < 8 && file >= 0 && file < 8 {
                        let to = (rank * 8 + file) as u8;
                        let to_mask = 1u64 << to;
                        let is_capture = opponent_pieces.iter().any(|&p| (p & to_mask) != 0);
                        if is_capture {
                            let captured_piece = self.get_piece_at(board, to);
                            // Check for promotion
                            if (board.side_to_move == Color::White && rank == 7) ||
                                (board.side_to_move == Color::Black && rank == 0) {
                                for promotion in [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight] {
                                    let mv = Move::new_promotion_capture(from as u8, to, captured_piece, promotion);
                                    // Make the move and check if the king is in check
                                    let mut board_copy = board.clone();
                                    board_copy.make_move(mv);
                                    if !self.is_king_in_check(&board_copy, board.side_to_move) {
                                        moves.push(mv);
                                    }
                                }
                            } else {
                                let mv = Move {
                                    from: from as u8,
                                    to,
                                    piece: Piece::Pawn,
                                    captured_piece: Some(captured_piece),
                                    promotion: None,
                                    is_en_passant: false,
                                    is_castling: false,
                                    castling_rook_from: None,
                                    castling_rook_to: None,
                                };
                                // Make the move and check if the king is in check
                                let mut board_copy = board.clone();
                                board_copy.make_move(mv);
                                if !self.is_king_in_check(&board_copy, board.side_to_move) {
                                    moves.push(mv);
                                }
                            }
                        }
                    }
                }

                // En passant
                if let Some(ep_square) = board.en_passant_square {
                    let ep_rank = ep_square / 8;
                    let from_rank = from / 8;
                    let from_file = from % 8;
                    let ep_file = ep_square % 8;
                    if (board.side_to_move == Color::White && ep_rank == 5 && from_rank == 4) ||
                        (board.side_to_move == Color::Black && ep_rank == 2 && from_rank == 3) {
                        if (ep_file as i8 - from_file as i8).abs() == 1 {
                            let captured_pawn_square = if board.side_to_move == Color::White {
                                ep_square - 8
                            } else {
                                ep_square + 8
                            };
                            let captured_pawn_mask = 1u64 << captured_pawn_square;
                            let has_pawn_to_capture = if board.side_to_move == Color::White {
                                (board.black_pieces[0] & captured_pawn_mask) != 0
                            } else {
                                (board.white_pieces[0] & captured_pawn_mask) != 0
                            };
                            if has_pawn_to_capture {
                                let mut mv = Move::new_en_passant(from as u8, ep_square, Piece::Pawn);
                                mv.captured_piece = Some(Piece::Pawn);
                                // Make the move and check if the king is in check
                                let mut board_copy = board.clone();
                                board_copy.make_move(mv);
                                if !self.is_king_in_check(&board_copy, board.side_to_move) {
                                    moves.push(mv);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Generate knight moves
        let knights = pieces[1];
        for from in 0..64 {
            if (knights >> from) & 1 != 0 {
                let from_rank = (from / 8) as i8;
                let from_file = (from % 8) as i8;
                let knight_moves = [
                    (2, 1), (2, -1), (-2, 1), (-2, -1),
                    (1, 2), (1, -2), (-1, 2), (-1, -2)
                ];
                for &(dr, df) in &knight_moves {
                    let rank = from_rank + dr;
                    let file = from_file + df;
                    if rank >= 0 && rank < 8 && file >= 0 && file < 8 {
                        let to = (rank * 8 + file) as u8;
                        let to_mask = 1u64 << to;
                        let is_capture = opponent_pieces.iter().any(|&p| (p & to_mask) != 0);
                        let is_empty = !pieces.iter().any(|&p| (p & to_mask) != 0);
                        if is_capture || is_empty {
                            let mut mv = Move::new(from as u8, to, Piece::Knight);
                            if is_capture {
                                mv.captured_piece = Some(self.get_piece_at(board, to));
                            }
                            // Make the move and check if the king is in check
                            let mut board_copy = board.clone();
                            board_copy.make_move(mv);
                            if !self.is_king_in_check(&board_copy, board.side_to_move) {
                                moves.push(mv);
                            }
                        }
                    }
                }
            }
        }

        // Generate bishop moves
        let bishops = pieces[2];
        for from in 0..64 {
            if (bishops >> from) & 1 != 0 {
                let occupied = board.white_pieces.iter().chain(board.black_pieces.iter())
                    .fold(0u64, |acc, &p| acc | p);
                let attacks = self.get_bishop_attacks(from as u8, occupied);
                for to in 0..64 {
                    if (attacks >> to) & 1 != 0 {
                        let to_mask = 1u64 << to;
                        let is_capture = opponent_pieces.iter().any(|&p| (p & to_mask) != 0);
                        let is_empty = !pieces.iter().any(|&p| (p & to_mask) != 0);
                        if is_capture || is_empty {
                            let mut mv = Move::new(from as u8, to as u8, Piece::Bishop);
                            if is_capture {
                                mv.captured_piece = Some(self.get_piece_at(board, to));
                            }
                            // Make the move and check if the king is in check
                            let mut board_copy = board.clone();
                            board_copy.make_move(mv);
                            if !self.is_king_in_check(&board_copy, board.side_to_move) {
                                moves.push(mv);
                            }
                        }
                    }
                }
            }
        }

        // Generate rook moves
        let rooks = pieces[3];
        for from in 0..64 {
            if (rooks >> from) & 1 != 0 {
                let occupied = board.white_pieces.iter().chain(board.black_pieces.iter())
                    .fold(0u64, |acc, &p| acc | p);
                let attacks = self.get_rook_attacks(from as u8, occupied);
                for to in 0..64 {
                    if (attacks >> to) & 1 != 0 {
                        let to_mask = 1u64 << to;
                        let is_capture = opponent_pieces.iter().any(|&p| (p & to_mask) != 0);
                        let is_empty = !pieces.iter().any(|&p| (p & to_mask) != 0);
                        if is_capture || is_empty {
                            let mut mv = Move::new(from as u8, to as u8, Piece::Rook);
                            if is_capture {
                                mv.captured_piece = Some(self.get_piece_at(board, to));
                            }
                            // Make the move and check if the king is in check
                            let mut board_copy = board.clone();
                            board_copy.make_move(mv);
                            if !self.is_king_in_check(&board_copy, board.side_to_move) {
                                moves.push(mv);
                            }
                        }
                    }
                }
            }
        }

        // Generate queen moves
        let queens = pieces[4];
        for from in 0..64 {
            if (queens >> from) & 1 != 0 {
                let occupied = board.white_pieces.iter().chain(board.black_pieces.iter())
                    .fold(0u64, |acc, &p| acc | p);
                let attacks = self.get_bishop_attacks(from as u8, occupied) |
                            self.get_rook_attacks(from as u8, occupied);
                for to in 0..64 {
                    if (attacks >> to) & 1 != 0 {
                        let to_mask = 1u64 << to;
                        let is_capture = opponent_pieces.iter().any(|&p| (p & to_mask) != 0);
                        let is_empty = !pieces.iter().any(|&p| (p & to_mask) != 0);
                        if is_capture || is_empty {
                            let mut mv = Move::new(from as u8, to as u8, Piece::Queen);
                            if is_capture {
                                mv.captured_piece = Some(self.get_piece_at(board, to));
                            }
                            // Make the move and check if the king is in check
                            let mut board_copy = board.clone();
                            board_copy.make_move(mv);
                            if !self.is_king_in_check(&board_copy, board.side_to_move) {
                                moves.push(mv);
                            }
                        }
                    }
                }
            }
        }

        // Generate king moves
        let king = pieces[5];
        for from in 0..64 {
            if (king >> from) & 1 != 0 {
                let from_rank = (from / 8) as i8;
                let from_file = (from % 8) as i8;
                let king_moves = [
                    (1, 0), (1, 1), (0, 1), (-1, 1),
                    (-1, 0), (-1, -1), (0, -1), (1, -1)
                ];
                for &(dr, df) in &king_moves {
                    let rank = from_rank + dr;
                    let file = from_file + df;
                    if rank >= 0 && rank < 8 && file >= 0 && file < 8 {
                        let to = (rank * 8 + file) as u8;
                        let to_mask = 1u64 << to;
                        let is_capture = opponent_pieces.iter().any(|&p| (p & to_mask) != 0);
                        let is_empty = !pieces.iter().any(|&p| (p & to_mask) != 0);
                        if is_capture || is_empty {
                            let mut mv = Move::new(from as u8, to as u8, Piece::King);
                            if is_capture {
                                mv.captured_piece = Some(self.get_piece_at(board, to));
                            }
                            // Make the move and check if the king is in check
                            let mut board_copy = board.clone();
                            board_copy.make_move(mv);
                            if !self.is_king_in_check(&board_copy, board.side_to_move) {
                                moves.push(mv);
                            }
                        }
                    }
                }

                // Castling
                let occupied = board.white_pieces.iter().chain(board.black_pieces.iter())
                    .fold(0u64, |acc, &p| acc | p);
                if board.side_to_move == Color::White {
                    // Kingside castling
                    if (board.castling_rights & 0b0001) != 0 &&
                        (board.white_pieces[3] & (1 << 7)) != 0 && // Rook on h1
                        (occupied & ((1 << 5) | (1 << 6))) == 0 && // f1 and g1 are empty
                        !self.is_square_under_attack(board, 4, Color::Black) && // e1 not attacked
                        !self.is_square_under_attack(board, 5, Color::Black) && // f1 not attacked
                        !self.is_square_under_attack(board, 6, Color::Black) { // g1 not attacked
                        let mv = Move::new_castling(4, 6, 7, 5);
                        // Make the move and check if the king is in check
                        let mut board_copy = board.clone();
                        board_copy.make_move(mv);
                        if !self.is_king_in_check(&board_copy, board.side_to_move) {
                            moves.push(mv);
                        }
                    }
                    // Queenside castling
                    if (board.castling_rights & 0b0010) != 0 &&
                        (board.white_pieces[3] & 1) != 0 && // Rook on a1
                        (occupied & ((1 << 1) | (1 << 2) | (1 << 3))) == 0 && // b1, c1, and d1 are empty
                        !self.is_square_under_attack(board, 4, Color::Black) && // e1 not attacked
                        !self.is_square_under_attack(board, 3, Color::Black) && // d1 not attacked
                        !self.is_square_under_attack(board, 2, Color::Black) { // c1 not attacked
                        let mv = Move::new_castling(4, 2, 0, 3);
                        // Make the move and check if the king is in check
                        let mut board_copy = board.clone();
                        board_copy.make_move(mv);
                        if !self.is_king_in_check(&board_copy, board.side_to_move) {
                            moves.push(mv);
                        }
                    }
                } else {
                    // Kingside castling
                    if (board.castling_rights & 0b0100) != 0 &&
                        (board.black_pieces[3] & (1 << 63)) != 0 && // Rook on h8
                        (occupied & ((1 << 61) | (1 << 62))) == 0 && // f8 and g8 are empty
                        !self.is_square_under_attack(board, 60, Color::White) && // e8 not attacked
                        !self.is_square_under_attack(board, 61, Color::White) && // f8 not attacked
                        !self.is_square_under_attack(board, 62, Color::White) { // g8 not attacked
                        let mv = Move::new_castling(60, 62, 63, 61);
                        // Make the move and check if the king is in check
                        let mut board_copy = board.clone();
                        board_copy.make_move(mv);
                        if !self.is_king_in_check(&board_copy, board.side_to_move) {
                            moves.push(mv);
                        }
                    }
                    // Queenside castling
                    if (board.castling_rights & 0b1000) != 0 &&
                        (board.black_pieces[3] & (1 << 56)) != 0 && // Rook on a8
                        (occupied & ((1 << 57) | (1 << 58) | (1 << 59))) == 0 && // b8, c8, and d8 are empty
                        !self.is_square_under_attack(board, 60, Color::White) && // e8 not attacked
                        !self.is_square_under_attack(board, 59, Color::White) && // d8 not attacked
                        !self.is_square_under_attack(board, 58, Color::White) { // c8 not attacked
                        let mv = Move::new_castling(60, 58, 56, 59);
                        // Make the move and check if the king is in check
                        let mut board_copy = board.clone();
                        board_copy.make_move(mv);
                        if !self.is_king_in_check(&board_copy, board.side_to_move) {
                            moves.push(mv);
                        }
                    }
                }
            }
        }

        moves
    }

    fn get_piece_at(&self, board: &Board, square: u8) -> Piece {
        let square_mask = 1u64 << square;
        
        // Check white pieces
        if (board.white_pieces[0] & square_mask) != 0 {
            return Piece::Pawn;
        }
        if (board.white_pieces[1] & square_mask) != 0 {
            return Piece::Knight;
        }
        if (board.white_pieces[2] & square_mask) != 0 {
            return Piece::Bishop;
        }
        if (board.white_pieces[3] & square_mask) != 0 {
            return Piece::Rook;
        }
        if (board.white_pieces[4] & square_mask) != 0 {
            return Piece::Queen;
        }
        if (board.white_pieces[5] & square_mask) != 0 {
            return Piece::King;
        }
        
        // Check black pieces
        if (board.black_pieces[0] & square_mask) != 0 {
            return Piece::Pawn;
        }
        if (board.black_pieces[1] & square_mask) != 0 {
            return Piece::Knight;
        }
        if (board.black_pieces[2] & square_mask) != 0 {
            return Piece::Bishop;
        }
        if (board.black_pieces[3] & square_mask) != 0 {
            return Piece::Rook;
        }
        if (board.black_pieces[4] & square_mask) != 0 {
            return Piece::Queen;
        }
        if (board.black_pieces[5] & square_mask) != 0 {
            return Piece::King;
        }
        
        // No piece found
        Piece::Pawn  // Default value, should never be reached
    }

    pub fn get_game_state(&self, board: &Board, move_history: &[(Board, Move)]) -> GameState {
        // Check for insufficient material
        if self.is_insufficient_material(board) {
            return GameState::InsufficientMaterial;
        }

        // Check for fifty-move rule
        if board.halfmove_clock >= 50 {
            return GameState::FiftyMoveRule;
        }

        // Check for threefold repetition
        if self.is_threefold_repetition(board, move_history) {
            return GameState::ThreefoldRepetition;
        }

        // Generate all legal moves
        let moves = self.generate_moves(board);

        // If there are no legal moves
        if moves.is_empty() {
            // Check if the king is in check
            if self.is_king_in_check(board, board.side_to_move) {
                // Checkmate - the side to move is in check and has no legal moves
                return GameState::Checkmate(board.side_to_move.opposite());
            } else {
                // Stalemate - the side to move is not in check but has no legal moves
                return GameState::Stalemate;
            }
        }

        GameState::Ongoing
    }

    fn is_threefold_repetition(&self, board: &Board, move_history: &[(Board, Move)]) -> bool {
        let current_hash = self.get_position_hash(board);
        let mut repetition_count = 1;

        for (past_board, _) in move_history {
            if self.get_position_hash(past_board) == current_hash {
                repetition_count += 1;
                if repetition_count >= 3 {
                    return true;
                }
            }
        }

        false
    }

    fn get_position_hash(&self, board: &Board) -> u64 {
        let mut hash = 0u64;

        // Hash white pieces
        for (piece_type, &bitboard) in board.white_pieces.iter().enumerate() {
            hash = hash.wrapping_mul(PRIME_NUMBERS[piece_type]);
            hash = hash.wrapping_add(bitboard);
        }

        // Hash black pieces
        for (piece_type, &bitboard) in board.black_pieces.iter().enumerate() {
            hash = hash.wrapping_mul(PRIME_NUMBERS[piece_type + 6]);
            hash = hash.wrapping_add(bitboard);
        }

        // Hash game state
        hash = hash.wrapping_mul(PRIME_NUMBERS[12]);
        hash = hash.wrapping_add(board.castling_rights as u64);

        if let Some(ep_square) = board.en_passant_square {
            hash = hash.wrapping_mul(PRIME_NUMBERS[13]);
            hash = hash.wrapping_add(ep_square as u64);
        }

        hash = hash.wrapping_mul(PRIME_NUMBERS[14]);
        hash = hash.wrapping_add(if board.side_to_move == Color::White { 0 } else { 1 });

        hash
    }

    fn is_insufficient_material(&self, board: &Board) -> bool {
        let (white_pieces, black_pieces) = self.count_pieces(board);
        let (white_minors, black_minors) = self.count_minor_pieces(board);
        let (white_bishops, black_bishops) = self.count_bishops(board);

        // King vs King
        if white_pieces == 1 && black_pieces == 1 {
            return true;
        }

        // King and Bishop vs King
        if (white_pieces == 2 && white_bishops == 1 && black_pieces == 1) ||
           (black_pieces == 2 && black_bishops == 1 && white_pieces == 1) {
            return true;
        }

        // King and Knight vs King
        if (white_pieces == 2 && white_minors == 1 && black_pieces == 1) ||
           (black_pieces == 2 && black_minors == 1 && white_pieces == 1) {
            return true;
        }

        // King and Bishop vs King and Bishop (same colored squares)
        if white_pieces == 2 && black_pieces == 2 &&
           white_bishops == 1 && black_bishops == 1 {
            let white_bishop_square = self.find_bishop_square(board, Color::White);
            let black_bishop_square = self.find_bishop_square(board, Color::Black);
            if let (Some(white_sq), Some(black_sq)) = (white_bishop_square, black_bishop_square) {
                let white_is_dark = (white_sq / 8 + white_sq % 8) % 2 == 1;
                let black_is_dark = (black_sq / 8 + black_sq % 8) % 2 == 1;
                if white_is_dark == black_is_dark {
                    return true;
                }
            }
        }

        false
    }

    fn count_pieces(&self, board: &Board) -> (u8, u8) {
        let white_count = board.white_pieces.iter().map(|&bb| bb.count_ones() as u8).sum();
        let black_count = board.black_pieces.iter().map(|&bb| bb.count_ones() as u8).sum();
        (white_count, black_count)
    }

    fn count_minor_pieces(&self, board: &Board) -> (u8, u8) {
        let white_count = (board.white_pieces[1] | board.white_pieces[2]).count_ones() as u8;
        let black_count = (board.black_pieces[1] | board.black_pieces[2]).count_ones() as u8;
        (white_count, black_count)
    }

    fn count_bishops(&self, board: &Board) -> (u8, u8) {
        let white_count = board.white_pieces[2].count_ones() as u8;
        let black_count = board.black_pieces[2].count_ones() as u8;
        (white_count, black_count)
    }

    fn find_bishop_square(&self, board: &Board, color: Color) -> Option<u8> {
        let bishops = match color {
            Color::White => board.white_pieces[2],
            Color::Black => board.black_pieces[2],
        };
        if bishops != 0 {
            Some(bishops.trailing_zeros() as u8)
        } else {
            None
        }
    }
}

const PRIME_NUMBERS: [u64; 15] = [
    2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47,
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameState {
    Ongoing,
    Checkmate(Color),  // Color is the winner
    Stalemate,
    ThreefoldRepetition,
    FiftyMoveRule,
    InsufficientMaterial,
} 