extern crate colored;

use colored::*;
use colored::ColoredString;
#[derive(Clone, Copy)]
struct Board{
    kings:u64,
    queens:u64,             // u64 bitmaps of all piece-types
    rooks:u64,
    bishops:u64,
    knights:u64,
    pawns:u64,
    whites:u64,             // colors are tracked with color-bitmasks
    blacks:u64,         
    castelable_pieces:u64,  // bitmap of rooks and kings that can castle
    fifty_rule:u8,          // number of moves without capture of pawn push
    en_passant_index:u8,    // index of piece susceptible to en passant
    eval:i8,                // evaluation of the position
}                       
/*
abcdefgh
10001001
00000000
00000000
00000000
00000000
00000000
00000000
10001001
*/


const MOVE_SEARCH_DEPTH:i32 = 5;


#[allow(dead_code)]
impl Board{
    fn new()->Board{
        Board {
            kings: 576460752303423496,
            queens: 1152921504606846992,
            rooks: 9295429630892703873, 
            bishops: 2594073385365405732,
            knights: 4755801206503243842,
            pawns: 71776119061282560,
            whites: 65535,
            blacks: 18446462598732840960,
            castelable_pieces:9871890383196127369,
            fifty_rule:0,
            en_passant_index:0,
            eval:0,
        }
    }

    fn evaluate_position(&self)->i32{


        unimplemented!("amogus");
    }

}

fn print_mask(mask:u64, name:&str){
    println!("{}:", name);
    let range = 0..8;
    for i in range.into_iter().rev(){
        let submask = (mask&(0b11111111<<(i*8)))>>(i*8);
        println!("{:08b}", submask);
    }
    println!(" ");
}


fn display_board(board:&Board){
    let mut board_list:Vec<ColoredString> = Vec::with_capacity(64);
    for  i in 0..64{
        let mut square:ColoredString;

        if board.whites&(1<<i)!=0{
            if (i+i/8)%2==0{
                if (board.whites|board.blacks)&(1<<i)==0{
                    square = "  ".truecolor(8, 11, 11).on_truecolor(255, 255, 255);
                }else if board.kings&(1<<i)!=0{
                    square = "♔ ".truecolor(8, 11, 11).on_truecolor(255, 255, 255);
                }else if board.queens&(1<<i)!=0{
                    square = "♕ ".truecolor(8, 11, 11).on_truecolor(255, 255, 255);
                }else if board.rooks&(1<<i)!=0{
                    square = "♖ ".truecolor(8, 11, 11).on_truecolor(255, 255, 255);
                }else if board.bishops&(1<<i)!=0{
                    square = "♗ ".truecolor(8, 11, 11).on_truecolor(255, 255, 255);
                }else if board.knights&(1<<i)!=0{
                    square = "♘ ".truecolor(8, 11, 11).on_truecolor(255, 255, 255);
                }else{
                    square = "♙ ".truecolor(8, 11, 11).on_truecolor(255, 255, 255);
                } 
            }else if (board.whites|board.blacks)&(1<<i)==0{
                square = "  ".truecolor(255, 255, 255).on_truecolor(119, 149, 86);
            }else if board.kings&(1<<i)!=0{
                square = "♚ ".truecolor(255, 255, 255).on_truecolor(119, 149, 86);
            }else if board.queens&(1<<i)!=0{
                square = "♛ ".truecolor(255, 255, 255).on_truecolor(119, 149, 86);
            }else if board.rooks&(1<<i)!=0{
                square = "♜ ".truecolor(255, 255, 255).on_truecolor(119, 149, 86);
            }else if board.bishops&(1<<i)!=0{
                square = "♝ ".truecolor(255, 255, 255).on_truecolor(119, 149, 86);
            }else if board.knights&(1<<i)!=0{
                square = "♞ ".truecolor(255, 255, 255).on_truecolor(119, 149, 86);
            }else{
                square = "♟︎ ".truecolor(255, 255, 255).on_truecolor(119, 149, 86);
            }    
        }else{
            if (board.whites|board.blacks)&(1<<i)==0{
                square = "  ".truecolor(8, 11, 11);
            }else if board.kings&(1<<i)!=0{
                square = "♚ ".truecolor(8, 11, 11);
            }else if board.queens&(1<<i)!=0{
                square = "♛ ".truecolor(8, 11, 11);
            }else if board.rooks&(1<<i)!=0{
                square = "♜ ".truecolor(8, 11, 11);
            }else if board.bishops&(1<<i)!=0{
                square = "♝ ".truecolor(8, 11, 11);
            }else if board.knights&(1<<i)!=0{
                square = "♞ ".truecolor(8, 11, 11);
            }else{
                square = "♟︎ ".truecolor(8, 11, 11);
            }
    
            if (i+i/8)%2==0{
                square = square.on_truecolor(255, 255, 255);
            }else{
                square = square.on_truecolor(119, 149, 86);
            }
        }
        board_list.push(square);
    }
    let range = 0..8;
    for i in range.into_iter().rev(){
        println!("{}{}{}{}{}{}{}{}",
            board_list[8*i+7], board_list[8*i+6],
            board_list[8*i+5], board_list[8*i+4],
            board_list[8*i+3], board_list[8*i+2],
            board_list[8*i+1], board_list[8*i  ],
        )
    }
    println!(" ");
}

fn collect_white_move(board:Board)->Board{
    let mut row = 9;
    let mut col = 9;
    while board.whites&(1<<(8*(col-1)+8-row))==0{
        let mut input:String = "".to_owned();
        println!("Select piece");
        std::io::stdin().read_line(&mut input).unwrap();
        println!("{}", input);
        if input.chars().count() != 3{
            println!("invalid square");
            continue;
        }
        let mut iter = input.chars();
        row = iter.next().unwrap() as u32 - 96;
        col = iter.next().unwrap() as u32 - 48;
    }

    println!("{}, {}", row, col);
    let piece_mask = 1<<(8*(col-1)+8-row);
    let move_squares:Vec<(i32,i32)> = possible_white_moves(&board, piece_mask);

    // mark and display the possible moves from move_squares
    // collect second input, move piece and return the board

    unimplemented!("amogus");
}

fn possible_white_moves(board:&Board, piece_mask:u64)->Vec<(i32, i32)>{

    // figure out what piece it is
    // use engine to find possible next positions
    // match difference between posissions and the current
    // make those into coridiantes and return them
    unimplemented!("amogus");
}

fn compare_boards(best_board:&mut Option<Board>, new_board:&mut Board, depth:i32){
    let best_response = find_best_response(*new_board); 
    let eval = find_best_move(best_response, depth-1).eval; //evaluation happens by magic
    new_board.eval = eval; 
    match best_board{
    None=>{*best_board = Some(*new_board)},
    Some(ref prev_board)=>{
        if eval > prev_board.eval{
        *best_board = Some(*new_board);    
        }
    }}
}


fn find_best_move(board:Board,  depth:i32)->Board{
    // the functions job is to take in results from the same functions one
    // layer above and compare them one at a time
    // my first intuition is to use while let Some() for all of it
    // the challange is not checking the same position twize without      
    // storing it on the heap. 
    let mut best_board:Option<Board> = None;
    let mut pawn_bitmap:u64 = 0;
    while let Some(mut new_board) = find_new_pawn_move(&board, &mut pawn_bitmap, depth){
        compare_boards(&mut best_board, &mut new_board, depth);
    }

    let mut knight_bitmap:u64 = 0;
    while let Some(mut new_board) = find_new_knight_move(&board, &mut knight_bitmap, depth){
        compare_boards(&mut best_board, &mut new_board, depth)
    }
    // challange with the knigth bitmap, and really all maps exept pawns
    // how to handle squares that multiple pieces can move to?
    // its a propper mind-bender


    
    

    unimplemented!("amogus");
}

fn find_best_response(board:Board)->Board{
    unimplemented!("amogus");
}

fn find_new_pawn_move(board:&Board, pawn_bitmap:&mut u64, depth:i32)->Option<Board>{
    
    // find a move that has not happened yet
    // tick the bitmap
    let mut new_board = None;
    let mut found_new_move = false;
    let mut pawns = board.pawns & board.blacks;
    while !found_new_move{
        let first_pawn = 1<<pawns.ilog2();
        if first_pawn == 0 {found_new_move = true}
        if first_pawn&(board.whites|board.blacks) != 0{continue}
        let pushed_pawn = first_pawn*256;
        if pushed_pawn&(*pawn_bitmap) == 0 {
            pawns = pawns^(first_pawn|pushed_pawn); 
            *pawn_bitmap = *pawn_bitmap|pushed_pawn;
            found_new_move = true;
            let mut board_copy = board.clone();
            board_copy.pawns = pawns;
            new_board = Some(board_copy);
        }else{
            let jumped_pawn = first_pawn*65536;
            if jumped_pawn&(*pawn_bitmap) == 0{
                pawns = pawns^(first_pawn|jumped_pawn);
                *pawn_bitmap = *pawn_bitmap|jumped_pawn;
                found_new_move = true;
                let mut board_copy = board.clone();
                board_copy.pawns = pawns;
                new_board = Some(board_copy);
            }else{
                pawns = pawns^first_pawn;
            }
        }
    }

    return new_board;
}

fn find_new_knight_move(board:&Board, pawn_bitmap:&mut u64, depth:i32)->Option<Board>{

    /*
    01010
    10001
    00x00  // find knight - shift this map - & with !white - gives all legal moves
    10001  // check with bitmap , select first that does not clash (legal & bitmap != 0)
    01010  // double bitmap shenanigans with move and piece encoding sdkjfskdj
    */

    let mut new_board:Option<Board> = None;
    let mut found_new_move = false;
    let mut knights  = board.knights & board.blacks;
    while !found_new_move{
        let index_of_knight = knights.ilog2() as u64;
        
    }
    unimplemented!("amogus");
}

fn main() {
    println!("let the chess begin");


    let mut board = Board::new();

    print_mask(board.castelable_pieces, "castelable pieces");

    while &board.kings.count_ones()==&2{ //yes this game lets you capture the kings before the game ends
        display_board(&board);
        
        board = collect_white_move(board);
        display_board(&board);

        board = find_best_move(board, MOVE_SEARCH_DEPTH);

        //modify move to board
        //calculate best response
        //make response
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
