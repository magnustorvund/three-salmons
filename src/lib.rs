pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub mod board;
pub mod movegen;
pub mod evaluation;
pub mod transposition;
pub mod search;
pub mod uci;

#[cfg(test)]
mod tests {
    use super::*;
    use board::{Board, Color, Piece};
    use movegen::{Move, MoveGenerator, GameState};

    #[test]
    fn test_initial_position() {
        let board = Board::new();
        let generator = MoveGenerator::new();
        let moves = generator.generate_moves(&board);
        
        // White should have 20 legal moves in the initial position
        assert_eq!(moves.len(), 20);
        
        // Check that all moves are valid
        for mv in moves {
            assert!(generator.is_move_valid(&board, &mv));
        }
    }

    #[test]
    fn test_pawn_moves() {
        let mut board = Board::new();
        let generator = MoveGenerator::new();
        
        // Test pawn double push
        let moves = generator.generate_moves(&board);
        let double_push = moves.iter().find(|mv| 
            mv.piece == Piece::Pawn && 
            mv.from / 8 == 1 && 
            mv.to / 8 == 3
        );
        assert!(double_push.is_some());
        
        // Test pawn capture
        // Clear all pieces
        for i in 0..6 {
            board.white_pieces[i] = 0;
            board.black_pieces[i] = 0;
        }
        
        // Set up a capture position
        board.white_pieces[0] = 0x0000000000001000;  // White pawn on e4
        board.black_pieces[0] = 0x0000000000080000;  // Black pawn on d5
        board.side_to_move = Color::White;  // White to move
        
        let moves = generator.generate_moves(&board);
        let capture = moves.iter().find(|mv| 
            mv.piece == Piece::Pawn && 
            mv.captured_piece.is_some()
        );
        assert!(capture.is_some());
    }

    #[test]
    fn test_castling() {
        let mut board = Board::new();
        let generator = MoveGenerator::new();
        
        // Clear all pieces except king and rooks
        for i in 0..6 {
            board.white_pieces[i] = 0;
            board.black_pieces[i] = 0;
        }
        
        // Set up king and rooks
        board.white_pieces[5] = 0x10;  // King on e1
        board.white_pieces[3] = 0x81;  // Rooks on a1 and h1
        board.castling_rights = 0b0011;  // Enable both white castling rights
        board.side_to_move = Color::White;  // White to move
        
        let moves = generator.generate_moves(&board);
        let kingside_castle = moves.iter().find(|mv| 
            mv.is_castling && 
            mv.from == 4 && 
            mv.to == 6
        );
        let queenside_castle = moves.iter().find(|mv| 
            mv.is_castling && 
            mv.from == 4 && 
            mv.to == 2
        );
        
        assert!(kingside_castle.is_some());
        assert!(queenside_castle.is_some());
    }

    #[test]
    fn test_en_passant() {
        let mut board = Board::new();
        let generator = MoveGenerator::new();
        
        // Set up en passant position
        board.make_move(Move::new(12, 28, Piece::Pawn));  // e2-e4
        board.make_move(Move::new(51, 35, Piece::Pawn));  // d7-d5
        board.make_move(Move::new(28, 36, Piece::Pawn));  // e4-e5
        board.make_move(Move::new(53, 37, Piece::Pawn));  // f7-f5
        
        let moves = generator.generate_moves(&board);
        let en_passant = moves.iter().find(|mv| 
            mv.is_en_passant && 
            mv.from == 36 && 
            mv.to == 45
        );
        assert!(en_passant.is_some());
    }

    #[test]
    fn test_promotion() {
        let mut board = Board::new();
        let generator = MoveGenerator::new();
        
        // Clear all pieces
        for i in 0..6 {
            board.white_pieces[i] = 0;
            board.black_pieces[i] = 0;
        }
        
        // Set up promotion position
        board.white_pieces[0] = 0x0080000000000000;  // White pawn on a7
        board.side_to_move = Color::White;  // White to move
        
        let moves = generator.generate_moves(&board);
        let promotions = moves.iter().filter(|mv| 
            mv.piece == Piece::Pawn && 
            mv.promotion.is_some()
        ).count();
        
        // Should have 4 promotion options (Queen, Rook, Bishop, Knight)
        assert_eq!(promotions, 4);
    }

    #[test]
    fn test_check() {
        let mut board = Board::new();
        let generator = MoveGenerator::new();
        
        // Set up a position where black is in check
        board.white_pieces[4] = 0x0000000000000004;  // White queen on c1
        board.white_pieces[5] = 0x0000000000000008;  // White king on d1
        board.black_pieces[5] = 0x0000000000000010;  // Black king on e1
        
        assert!(generator.is_king_in_check(&board, Color::Black));
    }

    #[test]
    fn test_checkmate() {
        let mut board = Board::new();
        let generator = MoveGenerator::new();
        
        // Clear all pieces
        for i in 0..6 {
            board.white_pieces[i] = 0;
            board.black_pieces[i] = 0;
        }
        
        // Set up a simple checkmate position with black king in corner
        board.white_pieces[4] = 0x0000000000000002;  // White queen on b1
        board.white_pieces[5] = 0x0000000000000004;  // White king on c1
        board.black_pieces[5] = 0x0000000000000001;  // Black king on a1
        board.side_to_move = Color::Black;  // Black to move
        
        // Print board state
        println!("White queen: 0x{:016x}", board.white_pieces[4]);
        println!("White king: 0x{:016x}", board.white_pieces[5]);
        println!("Black king: 0x{:016x}", board.black_pieces[5]);
        
        // Verify the position
        println!("Black king in check: {}", generator.is_king_in_check(&board, Color::Black));
        let moves = generator.generate_moves(&board);
        println!("Legal moves for black king:");
        for mv in &moves {
            println!("  From: {}, To: {}", mv.from, mv.to);
            println!("    Square {} under attack: {}", mv.to, generator.is_square_under_attack(&board, mv.to, Color::White));
            let mut board_copy = board.clone();
            board_copy.make_move(*mv);
            println!("    After move, king in check: {}", generator.is_king_in_check(&board_copy, Color::Black));
        }
        assert!(moves.is_empty());  // Black has no legal moves
        
        let state = generator.get_game_state(&board, &[]);
        assert_eq!(state, GameState::Checkmate(Color::White));
    }

    #[test]
    fn test_stalemate() {
        let mut board = Board::new();
        let generator = MoveGenerator::new();
        
        // Clear all pieces
        for i in 0..6 {
            board.white_pieces[i] = 0;
            board.black_pieces[i] = 0;
        }
        
        // Set up a simple stalemate position
        board.white_pieces[5] = 0x0000000000000001;  // White king on a1
        board.black_pieces[5] = 0x0000000000000400;  // Black king on c2
        board.black_pieces[4] = 0x0000000000020000;  // Black queen on b3
        board.side_to_move = Color::White;  // White to move
        
        // Verify the position
        assert!(!generator.is_king_in_check(&board, Color::White));  // White king is not in check
        let moves = generator.generate_moves(&board);
        assert!(moves.is_empty());  // White has no legal moves
        
        let state = generator.get_game_state(&board, &[]);
        assert_eq!(state, GameState::Stalemate);
    }

    #[test]
    fn test_insufficient_material() {
        let mut board = Board::new();
        let generator = MoveGenerator::new();
        
        // King vs King
        board.white_pieces[5] = 0x0000000000000008;  // White king
        board.black_pieces[5] = 0x0000000000000010;  // Black king
        for i in 0..5 {
            board.white_pieces[i] = 0;
            board.black_pieces[i] = 0;
        }
        
        let state = generator.get_game_state(&board, &[]);
        assert_eq!(state, GameState::InsufficientMaterial);
        
        // King and bishop vs King
        board.white_pieces[2] = 0x0000000000000004;  // Add white bishop
        let state = generator.get_game_state(&board, &[]);
        assert_eq!(state, GameState::InsufficientMaterial);
    }

    #[test]
    fn test_fifty_move_rule() {
        let mut board = Board::new();
        let generator = MoveGenerator::new();
        
        // Set halfmove clock to 50
        board.halfmove_clock = 50;
        
        let state = generator.get_game_state(&board, &[]);
        assert_eq!(state, GameState::FiftyMoveRule);
    }

    #[test]
    fn test_threefold_repetition() {
        let board = Board::new();
        let generator = MoveGenerator::new();
        
        // Create a move history with three identical positions
        let move_history = vec![
            (board.clone(), Move::new(12, 28, Piece::Pawn)),  // e2-e4
            (board.clone(), Move::new(52, 36, Piece::Pawn)),  // e7-e5
            (board.clone(), Move::new(28, 12, Piece::Pawn)),  // e4-e2
            (board.clone(), Move::new(36, 52, Piece::Pawn)),  // e5-e7
            (board.clone(), Move::new(12, 28, Piece::Pawn)),  // e2-e4
            (board.clone(), Move::new(52, 36, Piece::Pawn)),  // e7-e5
        ];
        
        let state = generator.get_game_state(&board, &move_history);
        assert_eq!(state, GameState::ThreefoldRepetition);
    }

    #[test]
    fn test_move_validation() {
        let mut board = Board::new();
        let generator = MoveGenerator::new();
        
        // Test a move that would leave the king in check
        board.white_pieces[4] = 0x0000000000000004;  // White queen on c1
        board.white_pieces[5] = 0x0000000000000008;  // White king on d1
        board.black_pieces[5] = 0x0000000000000010;  // Black king on e1
        
        let invalid_move = Move::new(8, 0, Piece::Rook);  // a1-a8 (would leave white king in check)
        assert!(!generator.is_move_valid(&board, &invalid_move));
    }

    #[test]
    fn test_perft_initial_position() {
        let board = Board::new();
        let generator = MoveGenerator::new();
        
        // Test perft(1) - initial position
        assert_eq!(perft(&board, &generator, 1), 20);
        
        // Test perft(2) - initial position
        assert_eq!(perft(&board, &generator, 2), 400);
        
        // Test perft(3) - initial position
        assert_eq!(perft(&board, &generator, 3), 8902);
    }

    // Helper function to perform perft
    fn perft(board: &Board, generator: &MoveGenerator, depth: u32) -> u64 {
        if depth == 0 {
            return 1;
        }
        
        let moves = generator.generate_moves(board);
        if depth == 1 {
            return moves.len() as u64;
        }
        
        let mut nodes = 0;
        for mv in moves {
            let mut new_board = board.clone();
            new_board.make_move(mv);
            nodes += perft(&new_board, generator, depth - 1);
        }
        
        nodes
    }
}
