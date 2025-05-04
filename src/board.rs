use std::fmt;
use crate::movegen::Move;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Board {
    pub white_pieces: [u64; 6],  // Pawn, Knight, Bishop, Rook, Queen, King
    pub black_pieces: [u64; 6],  // Pawn, Knight, Bishop, Rook, Queen, King
    pub side_to_move: Color,
    pub castling_rights: u8,  // 4 bits: KQkq
    pub en_passant_square: Option<u8>,
    pub halfmove_clock: u8,
    pub fullmove_number: u16,
}

impl Board {
    pub fn new() -> Self {
        Self {
            white_pieces: [
                0x000000000000FF00,  // Pawns
                0x0000000000000042,  // Knights
                0x0000000000000024,  // Bishops
                0x0000000000000081,  // Rooks
                0x0000000000000008,  // Queen
                0x0000000000000010,  // King
            ],
            black_pieces: [
                0x00FF000000000000,  // Pawns
                0x4200000000000000,  // Knights
                0x2400000000000000,  // Bishops
                0x8100000000000000,  // Rooks
                0x0800000000000000,  // Queen
                0x1000000000000000,  // King
            ],
            side_to_move: Color::White,
            castling_rights: 0b1111,  // All castling rights available
            en_passant_square: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    pub fn from_fen(_fen: &str) -> Result<Self, String> {
        // TODO: Implement FEN parsing
        Ok(Board::new())
    }

    pub fn to_fen(&self) -> String {
        // TODO: Implement FEN generation
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string()
    }

    pub fn make_move(&mut self, mv: Move) {
        let from_mask = 1u64 << mv.from;
        let to_mask = 1u64 << mv.to;
        let is_white = self.side_to_move == Color::White;

        // Remove piece from source square
        let pieces = if is_white {
            &mut self.white_pieces
        } else {
            &mut self.black_pieces
        };
        for piece_bb in pieces.iter_mut() {
            if (*piece_bb & from_mask) != 0 {
                *piece_bb &= !from_mask;
                break;
            }
        }

        // Handle captures
        if mv.captured_piece.is_some() {
            let captured_piece = mv.captured_piece.unwrap();
            let piece_index = match captured_piece {
                Piece::Pawn => 0,
                Piece::Knight => 1,
                Piece::Bishop => 2,
                Piece::Rook => 3,
                Piece::Queen => 4,
                Piece::King => 5,
            };
            let captured_square = if mv.is_en_passant {
                if is_white {
                    mv.to - 8
                } else {
                    mv.to + 8
                }
            } else {
                mv.to
            };
            let captured_mask = 1u64 << captured_square;
            if is_white {
                self.black_pieces[piece_index] &= !captured_mask;
            } else {
                self.white_pieces[piece_index] &= !captured_mask;
            }
        }

        // Place piece on target square
        let piece_index = match mv.piece {
            Piece::Pawn => 0,
            Piece::Knight => 1,
            Piece::Bishop => 2,
            Piece::Rook => 3,
            Piece::Queen => 4,
            Piece::King => 5,
        };

        // Handle promotion
        if let Some(promotion) = mv.promotion {
            let promotion_index = match promotion {
                Piece::Queen => 4,
                Piece::Rook => 3,
                Piece::Bishop => 2,
                Piece::Knight => 1,
                _ => unreachable!(),
            };
            if is_white {
                self.white_pieces[promotion_index] |= to_mask;
            } else {
                self.black_pieces[promotion_index] |= to_mask;
            }
        } else {
            if is_white {
                self.white_pieces[piece_index] |= to_mask;
            } else {
                self.black_pieces[piece_index] |= to_mask;
            }
        }

        // Handle castling
        if mv.is_castling {
            let (rook_from, rook_to) = if mv.to > mv.from {  // Kingside
                if is_white {
                    (7, 5)  // h1 to f1
                } else {
                    (63, 61)  // h8 to f8
                }
            } else {  // Queenside
                if is_white {
                    (0, 3)  // a1 to d1
                } else {
                    (56, 59)  // a8 to d8
                }
            };
            let rook_from_mask = 1u64 << rook_from;
            let rook_to_mask = 1u64 << rook_to;
            if is_white {
                self.white_pieces[3] &= !rook_from_mask;  // Remove rook from source square
                self.white_pieces[3] |= rook_to_mask;     // Place rook on target square
            } else {
                self.black_pieces[3] &= !rook_from_mask;  // Remove rook from source square
                self.black_pieces[3] |= rook_to_mask;     // Place rook on target square
            }
        }

        // Update castling rights
        if mv.piece == Piece::King {
            if is_white {
                self.castling_rights &= !0b0011;  // Clear white castling rights
            } else {
                self.castling_rights &= !0b1100;  // Clear black castling rights
            }
        } else if mv.piece == Piece::Rook {
            match (self.side_to_move, mv.from) {
                (Color::White, 0) => self.castling_rights &= !0b0010,  // White queenside
                (Color::White, 7) => self.castling_rights &= !0b0001,  // White kingside
                (Color::Black, 56) => self.castling_rights &= !0b1000, // Black queenside
                (Color::Black, 63) => self.castling_rights &= !0b0100, // Black kingside
                _ => {}
            }
        }

        // Update en passant square
        self.en_passant_square = if mv.piece == Piece::Pawn && (mv.to as i8 - mv.from as i8).abs() == 16 {
            Some(if is_white { mv.from + 8 } else { mv.from - 8 })
        } else {
            None
        };

        // Update move counters
        if mv.piece == Piece::Pawn || mv.captured_piece.is_some() {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }
        if !is_white {
            self.fullmove_number += 1;
        }

        // Switch side to move
        self.side_to_move = self.side_to_move.opposite();
    }

    pub fn get_piece_at(&self, square: u8) -> Option<(Piece, Color)> {
        let mask = 1u64 << square;
        
        // Check white pieces
        for (i, bb) in self.white_pieces.iter().enumerate() {
            if (bb & mask) != 0 {
                return Some((match i {
                    0 => Piece::Pawn,
                    1 => Piece::Knight,
                    2 => Piece::Bishop,
                    3 => Piece::Rook,
                    4 => Piece::Queen,
                    5 => Piece::King,
                    _ => return None,
                }, Color::White));
            }
        }
        
        // Check black pieces
        for (i, bb) in self.black_pieces.iter().enumerate() {
            if (bb & mask) != 0 {
                return Some((match i {
                    0 => Piece::Pawn,
                    1 => Piece::Knight,
                    2 => Piece::Bishop,
                    3 => Piece::Rook,
                    4 => Piece::Queen,
                    5 => Piece::King,
                    _ => return None,
                }, Color::Black));
            }
        }
        
        None
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result = String::new();
        for rank in (0..8).rev() {
            for file in 0..8 {
                let square = rank * 8 + file;
                let mut piece_found = false;
                
                // Check white pieces
                for (piece_type, &bitboard) in self.white_pieces.iter().enumerate() {
                    if (bitboard >> square) & 1 != 0 {
                        let piece_char = match piece_type {
                            0 => 'P', 1 => 'N', 2 => 'B', 3 => 'R', 4 => 'Q', 5 => 'K',
                            _ => '?',
                        };
                        result.push(piece_char);
                        piece_found = true;
                        break;
                    }
                }
                
                // Check black pieces
                if !piece_found {
                    for (piece_type, &bitboard) in self.black_pieces.iter().enumerate() {
                        if (bitboard >> square) & 1 != 0 {
                            let piece_char = match piece_type {
                                0 => 'p', 1 => 'n', 2 => 'b', 3 => 'r', 4 => 'q', 5 => 'k',
                                _ => '?',
                            };
                            result.push(piece_char);
                            piece_found = true;
                            break;
                        }
                    }
                }
                
                if !piece_found {
                    result.push('.');
                }
                
                if file < 7 {
                    result.push(' ');
                }
            }
            result.push('\n');
        }
        write!(f, "{}", result)
    }
} 