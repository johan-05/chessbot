extern crate colored;


use std::thread::panicking;

use colored::*;
use colored::ColoredString;
#[derive(Copy)]
#[allow(dead_code)]
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
    eval:i16,                // evaluation of the position
}

impl Clone for Board{
    fn clone(&self)->Board{
        Board {
            kings: self.kings,
            queens: self.queens, 
            rooks: self.rooks, 
            bishops: self.bishops, 
            knights: self.knights, 
            pawns: self.pawns, 
            whites: self.whites, 
            blacks: self.blacks, 
            castelable_pieces: self.castelable_pieces, 
            fifty_rule: self.fifty_rule, 
            en_passant_index: 0, 
            eval: self.eval 
        }
    }
}

struct KnOfst{
    offset:i32,
    usage_mask_p:u64,
    usage_mask_n:u64
}

struct OffsetClosure<T:Fn(u64, u64)->u64>{
    closure:T,
    usage_mask:u64,
}

impl<T> OffsetClosure<T>
where
    T:Fn(u64, u64)->u64
{
    const fn new(closure:T, usage_mask:u64)->Self{
        Self{closure:closure, usage_mask:usage_mask}
    }
}

/*
abcdefgh
00000000
00000000
00000000
00000000
00000000
00000000
11111111
00000000
*/




const MOVE_SEARCH_DEPTH:i32 = 2;
const CENTER_PIECES:u64 = 103481868288;
const BLACK_PAWN_HOMEROW:u64 = 71776119061217280;
const WHITE_PAWN_HOMEROW:u64 = 65280;
const ALLOWED_CAPTURE_LEFT:u64 = 18374403900871474942;
const ALLOWED_CAPTURE_RIGHT:u64 = 9187201950435737471;
const KNIGHT_OFFSETS:[KnOfst;4] = [
    KnOfst{offset:17, usage_mask_p:18446603888132915328, usage_mask_n:723401728381419519},
    KnOfst{offset:15, usage_mask_p:18446463702556279041, usage_mask_n:9259542123273846783},
    KnOfst{offset:10, usage_mask_p:18428941609300181184, usage_mask_n:217020518514230271},
    KnOfst{offset: 6, usage_mask_p:18375534216072069891, usage_mask_n:13889313184910721279},

];


const CLOSURES:[fn(u64, u64)->u64; 8] = [
    |x,y| x<<(9*y),
    |x,y| x<<(7*y),
    |x,y| x>>(9*y),
    |x,y| x>>(7*y),
    |x,y| x<<(1*y),
    |x,y| x<<(8*y),
    |x,y| x>>(1*y),
    |x,y| x>>(8*y),
];

const SHIFTING_CLOSURES:[OffsetClosure<fn(u64, u64)->u64>; 8] = [
    OffsetClosure::new(CLOSURES[0], 72340172838076673),
    OffsetClosure::new(CLOSURES[1], 9259542123273814144),
    OffsetClosure::new(CLOSURES[2], 9259542123273814144),
    OffsetClosure::new(CLOSURES[3], 72340172838076673),
    OffsetClosure::new(CLOSURES[4], 72340172838076673),
    OffsetClosure::new(CLOSURES[5], 0),
    OffsetClosure::new(CLOSURES[6], 9259542123273814144),
    OffsetClosure::new(CLOSURES[7], 0),
];


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

    fn evaluate_position(&mut self)->i16{
        let eval;

        let pawn = (self.pawns&self.blacks).count_ones() as i16 - (self.pawns&self.whites).count_ones() as i16;
        let knight = 3*((self.knights&self.blacks).count_ones() as i16 - (self.knights&self.whites).count_ones() as i16);
        let bishop = 3*((self.bishops&self.blacks).count_ones() as i16 - (self.bishops&self.whites).count_ones() as i16);
        let rook = 5*((self.rooks&self.blacks).count_ones() as i16 - (self.rooks&self.whites).count_ones() as i16);
        let queen = 9*((self.queens&self.blacks).count_ones() as i16 - (self.queens&self.whites).count_ones() as i16);
        let king = 200*((self.kings&self.blacks).count_ones() as i16 - (self.kings&self.whites).count_ones() as i16);
        let centre = (self.pawns&self.blacks&CENTER_PIECES).count_ones() as i16 - (self.pawns&self.whites&CENTER_PIECES).count_ones() as i16;
        eval = pawn+knight+bishop+rook+queen+king+centre;
        self.eval = eval;
        return eval;   
    }


    fn take(&mut self, bitmap:u64){
        if self.blacks & bitmap != 0{
            self.blacks ^= bitmap;
        }else{
            self.whites ^= bitmap;
        }

        if self.pawns & bitmap != 0{
            self.pawns ^= bitmap;
        } else if self.knights & bitmap != 0{
            self.knights ^= bitmap;
        } else if self.bishops & bitmap != 0{
            self.bishops ^= bitmap;
        } else if self.rooks & bitmap != 0{
            self.rooks ^= bitmap;
        } else if self.queens & bitmap != 0{
            self.queens ^= bitmap;
        } else if self.kings & bitmap != 0{
            self.kings ^= bitmap;
        }
    }

}



#[inline]
fn compare_boards(best_board:&mut Option<Board>, new_board:&mut Board, depth:i32){
    //println!("compare_boards()");
    let eval;
    if depth != 0{
        let best_response = find_best_response(*new_board); 
        eval = find_best_move(best_response, depth-1).eval; //evaluation happens by magic
        new_board.eval = eval; 
    }else{
        eval = new_board.evaluate_position();
    }
    match best_board{
        None=>{*best_board = Some(*new_board)},
        Some(ref prev_board)=>{
            if eval >= prev_board.eval{
                *best_board = Some(*new_board);    
            }
        }
    }
}

#[inline]
fn compare_boards_negative(best_board:&mut Option<Board>, new_board:&mut Board){
    //println!("compare_boards_negative()");
    //let best_response = find_best_response(*new_board); 

    // let eval = find_best_move(best_response, depth-1).eval; 
    // I might add small depth to the response search later

    let eval = new_board.evaluate_position();
    new_board.eval = eval; 
    match best_board{
    None=>{*best_board = Some(*new_board)},
    Some(ref prev_board)=>{
        if eval < prev_board.eval{
        *best_board = Some(*new_board);    
        }
    }}
}


fn find_best_move(board:Board,  depth:i32)->Board{
    // the functions job is to take in results from the same functions one
    // layer above and compare them one at a time
    // my first intuition is to use while let Some() for all of it
    // the challange is not checking the same position twize without      
    // storing every checked position on the heap. 

    let mut best_board:Option<Board> = None;
    println!("starting pawn search layer {}", depth);
    let mut pawn_bitmap:u64 = 0;
    while let Some(mut new_board) = find_new_pawn_move(&board, &mut pawn_bitmap){
        compare_boards(&mut best_board, &mut new_board, depth)
    }
    
    println!("starting knight search layer {}", depth);
    let mut knight_bitmap:u64 = 0;
    while let Some(mut new_board) = find_new_knight_move(&board, &mut knight_bitmap, board.blacks){
        compare_boards(&mut best_board, &mut new_board, depth);
    }

    println!("starting bishop search layer {}", depth);
    let mut bishop_bitmap:u64 = 0;
    while let Some(mut new_board) = find_new_bishop_move(&board, &mut bishop_bitmap, board.blacks){
        compare_boards(&mut best_board, &mut new_board, depth);
    }

    println!("starting rook search layer {}", depth);
    let mut rook_bitmap:u64 = 0;
    while let Some(mut new_board) = find_new_rook_move(&board, &mut rook_bitmap, board.blacks){
        //print_mask(new_board.rooks, "rooks");
        compare_boards(&mut best_board, &mut new_board, depth);
    }

    println!("starting queen search layer {}", depth);
    let mut queen_bitmap:u64 = 0;
    while let Some(mut new_board) = find_new_queen_move(&board, &mut queen_bitmap, board.blacks){
        compare_boards(&mut best_board, &mut new_board, depth);
    }
    println!("starting king search layer {}", depth);
    let mut king_bitmap:u64 = 0;
    while let Some(mut new_board) = find_new_king_move(&board, &mut king_bitmap, board.blacks){
        compare_boards(&mut best_board, &mut new_board, depth);
    }
    println!("finished layer {}", depth);
    let return_board = best_board.unwrap();
    if return_board.whites & return_board.blacks != 0{
        display_board_windows(&board, 0);
        print_mask(board.whites, "old whites");
        print_mask(board.blacks, "old blacks");
        display_board_windows(&return_board, 0);
        print_mask(return_board.whites, "new whites");
        print_mask(return_board.blacks, "new blacks");
        panic!("we messed up");
    }
    return return_board;
}

fn find_best_response(board:Board)->Board{
    //println!("find_best_response()");
    // this funciton does the same shit as the find_best_move()
    // but its from white, and searches for the move thats in whites favour
    // compare_boards_negative() finds the "worst" move, from the engines pov

    let mut best_board:Option<Board> = None;

    let mut pawn_bitmap:u64 = 0;
    println!("starting pawn response search layer");
    while let Some(mut new_board) = find_new_white_pawn_move(&board, &mut pawn_bitmap){
        //println!("found_pawn_move()");
        compare_boards_negative(&mut best_board, &mut new_board);
    }

    println!("starting knight response search layer");
    let mut knight_bitmap:u64 = 0;
    while let Some(mut new_board) = find_new_knight_move(&board, &mut knight_bitmap, board.whites){
        //println!("found_knight_move()");
        compare_boards_negative(&mut best_board, &mut new_board);
    }

    println!("starting bishop response search layer");
    let mut bishop_bitmap:u64 = 0;
    while let Some(mut new_board) = find_new_bishop_move(&board, &mut bishop_bitmap, board.whites){
        //print_mask(new_board.bishops, "new bishops");
        compare_boards_negative(&mut best_board, &mut new_board);
    }

    println!("starting rook response search layer");
    let mut rook_bitmap:u64 = 0;
    while let Some(mut new_board) = find_new_rook_move(&board, &mut rook_bitmap, board.whites){
        //println!("found_rook_move()");
        compare_boards_negative(&mut best_board, &mut new_board);
    }

    println!("starting queen response search layer");
    let mut queen_bitmap:u64 = 0;
    while let Some(mut new_board) = find_new_queen_move(&board, &mut queen_bitmap, board.whites){
        //println!("found_queen_move()");
        compare_boards_negative(&mut best_board, &mut new_board);
    }

    println!("starting king response search layer");
    let mut king_bitmap:u64 = 0;
    while let Some(mut new_board) = find_new_king_move(&board, &mut king_bitmap, board.whites){
        //println!("found_king_move()");
        compare_boards_negative(&mut best_board, &mut new_board);
    }

    let return_board = best_board.unwrap();
    if return_board.whites & return_board.blacks != 0{
        display_board_windows(&board, 0);
        print_mask(board.whites, "old whites");
        print_mask(board.blacks, "old blacks");
        print_mask(board.pawns, "old pawns");
        print_mask(board.queens, "old queen");
        display_board_windows(&return_board, 0);
        print_mask(return_board.whites, "new whites");
        print_mask(return_board.blacks, "new blacks");
        print_mask(return_board.pawns, "new pawns");
        print_mask(return_board.queens, "new queen");
        panic!("we messed up");
    }
    
    return return_board;
}

/*
abcdefgh
00010000
00000000
00000000
00010000
00000000
00000000
00000000
00000000

abcdefgh

01111111
01111111
01111111
01111111
01111111
01111111
01111111
01111111


00000000
00000000
00000000
00010000
00000000
00000000
11111111
11101111
*/



 
/*fn not_main(){
    let mut board = Board::new();
    board.queens = 1152921573326323712;
   // board.blacks = 18441959067824947200;
    board.whites = 68719542255;
    let mut pawn_bitmap:u64 = 0;
    display_board_windows(&board, 1);
    std::thread::sleep(std::time::Duration::from_secs(2));
   while let Some(new_board) = find_new_queen_move(&board, &mut pawn_bitmap, board.whites){
        display_board_windows(&new_board, 0);
    }
}*/

fn find_new_pawn_move(board:&Board, pawn_bitmap:&mut u64)->Option<Board>{
    // find a move that has not happened yet
    // tick the bitmap

    let mut pawns = board.pawns & board.blacks;
    while pawns != 0{
        let first_pawn = 1<<pawns.ilog2();
        if first_pawn & *pawn_bitmap != 0{
            pawns ^= first_pawn;
            continue;
        }
        let take_l = first_pawn>>7;
        let take_r = first_pawn>>9;
        let pushed_pawn = first_pawn>>8;
        let jumped_pawn = first_pawn>>16;
        if pushed_pawn & *pawn_bitmap | (board.whites&pushed_pawn)  == 0{
            *pawn_bitmap = *pawn_bitmap|pushed_pawn;
            let mut board_copy = board.clone();
            board_copy.pawns ^= first_pawn|pushed_pawn  ;
            board_copy.blacks ^= first_pawn|pushed_pawn;
            return Some(board_copy);
        }else if jumped_pawn & *pawn_bitmap | (board.whites&jumped_pawn) == 0 && first_pawn & BLACK_PAWN_HOMEROW != 0{
            *pawn_bitmap = *pawn_bitmap|jumped_pawn;
            let mut board_copy = board.clone();
            board_copy.pawns = board.pawns^first_pawn|jumped_pawn;
            board_copy.blacks = board.blacks^first_pawn|jumped_pawn;
            return Some(board_copy);
        }else if take_l & (board.whites & !*pawn_bitmap & ALLOWED_CAPTURE_LEFT) != 0{
            *pawn_bitmap = *pawn_bitmap|take_l;
            let mut board_copy = board.clone();
            board_copy.take(take_l);
            board_copy.pawns ^= first_pawn|take_l;
            board_copy.blacks ^= first_pawn|take_l;
            return Some(board_copy);
        }else if take_r & (board.whites & !*pawn_bitmap & ALLOWED_CAPTURE_RIGHT) != 0{
            *pawn_bitmap = *pawn_bitmap|take_r;
            let mut board_copy = board.clone();
            board_copy.take(take_r);
            board_copy.pawns ^= first_pawn|take_r;
            board_copy.blacks ^= first_pawn|take_r;
            return Some(board_copy);
        }else if (take_l | take_r) & 1<<board.en_passant_index & board.whites != 0{
            *pawn_bitmap |= 1<<board.en_passant_index;
            let mut board_copy = board.clone();
            board_copy.take(1<<(board.en_passant_index-8));
            board_copy.pawns ^= first_pawn|1<<board.en_passant_index;
            board_copy.blacks ^= first_pawn|1<<board.en_passant_index;
            return Some(board_copy);
        }else{
            pawns ^= first_pawn;
            //print_mask(*pawn_bitmap, "black mask before");
            *pawn_bitmap = (*pawn_bitmap | first_pawn) & board.pawns & board.blacks;
            //print_mask(*pawn_bitmap, "mask after");
        }
    }

    return None;
}

fn find_new_white_pawn_move(board:&Board, pawn_bitmap:&mut u64)->Option<Board>{
    //println!("find_new_white_pawn_move()");
    // find a move that has not happened yet
    // tick the bitmap

    let mut pawns = board.pawns & board.whites;
    while pawns != 0{
        let first_pawn = 1<<pawns.ilog2();
        if first_pawn & *pawn_bitmap != 0{
            pawns ^= first_pawn;
            continue;
        }
        let take_l = first_pawn<<9;
        let take_r = first_pawn<<7;
        let pushed_pawn = first_pawn<<8;
        let jumped_pawn = first_pawn<<16;
        if pushed_pawn&(*pawn_bitmap) | (board.blacks&pushed_pawn) == 0 {
            *pawn_bitmap |= pushed_pawn;
            let mut board_copy = board.clone();
            board_copy.pawns ^= first_pawn|pushed_pawn;
            board_copy.whites ^= first_pawn|pushed_pawn;
            return Some(board_copy);
        }else if (jumped_pawn & *pawn_bitmap) | (board.blacks&jumped_pawn) == 0 && first_pawn & WHITE_PAWN_HOMEROW != 0{
            *pawn_bitmap |= jumped_pawn;
            let mut board_copy = board.clone();
            board_copy.pawns ^= first_pawn|jumped_pawn;
            board_copy.whites ^= first_pawn|jumped_pawn;
            return Some(board_copy);
        }else if take_l & (board.blacks & !*pawn_bitmap & ALLOWED_CAPTURE_LEFT) != 0{
            *pawn_bitmap |= take_l;
            let mut board_copy = board.clone();
            board_copy.take(take_l);
            board_copy.pawns ^= first_pawn|take_l;
            board_copy.whites ^= first_pawn|take_l;
            return Some(board_copy);
        }else if take_r & (board.blacks & !*pawn_bitmap & ALLOWED_CAPTURE_RIGHT) != 0{
            *pawn_bitmap |= take_r;
            let mut board_copy = board.clone();
            board_copy.take(take_r);
            board_copy.pawns ^= first_pawn|take_r;
            board_copy.whites ^= first_pawn|take_r;
            return Some(board_copy);
        }else if (take_l | take_r) & 1<<board.en_passant_index & board.blacks != 0{
            *pawn_bitmap |= 1<<board.en_passant_index;
            let mut board_copy = board.clone();
            board_copy.take(1<<(board.en_passant_index+8));
            board_copy.pawns ^= first_pawn|1<<board.en_passant_index;
            board_copy.whites ^= first_pawn|1<<board.en_passant_index;
            return Some(board_copy);
        }else{
            pawns ^= first_pawn;
            //print_mask(*pawn_bitmap, "white mask before");
            *pawn_bitmap = (*pawn_bitmap | first_pawn) & board.pawns & board.whites;
            //print_mask(*pawn_bitmap, "mask after");
        }
    }

    return None;
}

fn find_new_knight_move(board:&Board, knight_bitmap:&mut u64, color_map:u64)->Option<Board>{
    /*
    01010
    10001
    00x00  // find knight - shift this map - & with !white - gives all legal moves
    10001  // check with bitmap , select first that does not clash (legal & bitmap != 0)
    01010  // double bitmap shenanigans with move and piece encoding 
    */
    let mut new_board:Option<Board> = None;
    let mut knights  = board.knights & color_map;
    'outer: loop{
        if knights == 0{break 'outer}
        let first_knight = 1<<knights.ilog2() as u64;
        if first_knight & *knight_bitmap != 0{
            knights = knights^first_knight;
            continue;
        }
        for kn_ofst in KNIGHT_OFFSETS{
            let moved_knight = first_knight<<kn_ofst.offset;
            if (moved_knight&(*knight_bitmap|color_map)==0)&&(first_knight&kn_ofst.usage_mask_p==0){
                knights = board.knights^first_knight|moved_knight;
                *knight_bitmap = *knight_bitmap|moved_knight;
                let mut board_copy = board.clone();
                if moved_knight & ((board.whites|board.blacks)^color_map) != 0{
                    board_copy.take(moved_knight);
                }
                board_copy.knights = knights;
                if color_map==board.whites{
                    board_copy.whites = board.whites^first_knight|moved_knight;
                }else{
                    board_copy.blacks = board.blacks^first_knight|moved_knight;
                }
                new_board = Some(board_copy);
                break 'outer;
            }
        }
        for kn_ofst in KNIGHT_OFFSETS{
            let moved_knight = first_knight>>kn_ofst.offset;
            if (moved_knight&(*knight_bitmap|color_map)==0)&&(first_knight&kn_ofst.usage_mask_n==0){
                knights = board.knights^first_knight|moved_knight;
                *knight_bitmap = *knight_bitmap|moved_knight;
                let mut board_copy = board.clone();
                if moved_knight & ((board.whites|board.blacks)^color_map) != 0{
                    board_copy.take(moved_knight);
                }
                board_copy.knights = knights;
                if color_map==board.whites{
                    board_copy.whites = board.whites^first_knight|moved_knight;
                }else{
                    board_copy.blacks = board.blacks^first_knight|moved_knight;
                }
                new_board = Some(board_copy);
                break 'outer;
            }
        }
        knights = knights^first_knight;
        *knight_bitmap = (*knight_bitmap|first_knight) & board.knights & color_map;
    }

    return new_board;
}

fn cross_positive(first_piece:u64, pieces:&mut u64, piece_bitmap:&mut u64, color_map:u64, rev_color:u64, offset:&OffsetClosure<fn(u64, u64)->u64>)->Option<u64>{
    for offset_scalars in 1..8{
        let moved_piece = (offset.closure)(first_piece, offset_scalars);
        //print_mask(moved_piece, "crossed piece");
        if moved_piece & (*piece_bitmap & rev_color) != 0{
            //print_mask(rev_color, "rev color");
            //println!("first broken");
            break;
        }

        if moved_piece & *piece_bitmap != 0{
            //println!("continued");
            continue;
        }
        if (moved_piece & color_map != 0)||(moved_piece == 0)||(moved_piece & offset.usage_mask != 0){
            //println!("hitttt");
            break;
        } 
        *pieces ^= first_piece|moved_piece;
        *piece_bitmap |= moved_piece;
        //println!("returned Some");
        //print_mask(moved_piece, "moved_piece");
        return Some(moved_piece);
    }
    //println!("returned None");
    return None;
}

fn find_new_bishop_move(board:&Board, bishop_bitmap:&mut u64, color_map:u64)->Option<Board>{
    // fancy schmancy raycasting
    let mut new_board:Option<Board> = None;
    let mut bishops = board.bishops & color_map;
    //print_mask(bishops, "bishops");
    'outer:loop{
        if bishops==0{break 'outer}
        let first_bishop = 1<<bishops.ilog2() as u64;
        //print_mask(first_bishop, "first bishops");
        //print_mask(first_bishop, "first bishop");
        //print_mask(*bishop_bitmap, "bishop bitmap");
        if first_bishop & *bishop_bitmap != 0{
            bishops ^= first_bishop;
            //*bishop_bitmap = *bishop_bitmap & board.bishops;
            continue;
        }

        let bishop_closure_indexes = [0,1,2,3];
        for closure_index in bishop_closure_indexes{
            if let Some(moved_bishop) = cross_positive(first_bishop, &mut bishops, bishop_bitmap, color_map, (board.whites|board.blacks)^color_map, &SHIFTING_CLOSURES[closure_index]){
                let mut board_copy = board.clone();
                if moved_bishop & ((board.whites|board.blacks)^color_map) != 0{
                    board_copy.take(moved_bishop);
                }
                board_copy.bishops = board.bishops^first_bishop|moved_bishop;
                if color_map==board.whites{
                    board_copy.whites ^= first_bishop|moved_bishop;
                }else{
                    board_copy.blacks ^= first_bishop|moved_bishop;
                }
                new_board = Some(board_copy);
                *bishop_bitmap = *bishop_bitmap|moved_bishop;
                break 'outer;
            }
        }
        bishops ^= first_bishop;
        *bishop_bitmap = (*bishop_bitmap|first_bishop) & board.bishops & color_map;
    }
    //print_mask(new_board.unwrap().bishops, "new bishops");
    return new_board;
}

fn find_new_rook_move(board:&Board, rook_bitmap:&mut u64, color_map:u64)->Option<Board>{
    // fancy schmancy raycasting
    let mut new_board:Option<Board> = None;
    let mut rooks = board.rooks & color_map;

    'outer:loop{
        if rooks==0{break 'outer}
        let first_rook = 1<<rooks.ilog2() as u64;
        if first_rook & *rook_bitmap != 0{
            rooks ^= first_rook;
            //*rook_bitmap = *rook_bitmap & board.rooks;
            continue;
        }

        let rook_closure_indexes = [4,5,6,7];
        for closure_index in rook_closure_indexes{
            //print_mask(*rook_bitmap, "rook bitmap before");
            if let Some(moved_rook) = cross_positive(first_rook, &mut rooks, rook_bitmap, color_map, (board.whites|board.blacks)^color_map, &SHIFTING_CLOSURES[closure_index]){
                //print_mask(*rook_bitmap, "rook bitmap after");
                let mut board_copy = board.clone();
                if moved_rook & ((board.whites|board.blacks)^color_map) != 0{
                    board_copy.take(moved_rook);
                }
                board_copy.rooks ^= first_rook|moved_rook;
                if color_map==board.whites{
                    board_copy.whites ^= first_rook|moved_rook;
                }else{
                    board_copy.blacks ^= first_rook|moved_rook;
                }
                new_board = Some(board_copy);
                *rook_bitmap = *rook_bitmap|moved_rook;
                break 'outer;
            }else{
                //println!("unable to move {}", closure_index);
            }
        }
        rooks ^= first_rook;
        *rook_bitmap = (*rook_bitmap|first_rook) & board.rooks & color_map;
    }

    return new_board;
}

fn find_new_queen_move(board:&Board, queen_bitmap:&mut u64, color_map:u64)->Option<Board>{
    // fancy schmancy raycasting
    let mut new_board:Option<Board> = None;
    let mut queens = board.queens & color_map;
    //print_mask(queens, "internal queens");
    'outer:loop{
        if queens==0{break 'outer}
        let first_queen = 1<<queens.ilog2() as u64;
        if first_queen & *queen_bitmap != 0{
            queens ^= first_queen;
            *queen_bitmap = *queen_bitmap & board.queens;
            continue;
        }

        let queen_closure_indexes = [0,1,2,3,4,5,6,7];
        for closure_index in queen_closure_indexes{
            if let Some(moved_queen) = cross_positive(first_queen, &mut queens, queen_bitmap, color_map, (board.whites|board.blacks)^color_map, &SHIFTING_CLOSURES[closure_index]){
                let mut board_copy = board.clone();
                if moved_queen & ((board.whites|board.blacks)^color_map) != 0{
                    board_copy.take(moved_queen);
                }
                board_copy.queens ^= first_queen|moved_queen;
                if color_map==board.whites{
                    board_copy.whites ^= first_queen|moved_queen;
                }else{
                    board_copy.blacks ^= first_queen|moved_queen;
                }
                new_board = Some(board_copy);
                break 'outer;
            }
        }
        queens ^= first_queen;
        *queen_bitmap = (*queen_bitmap|first_queen) & board.queens & color_map;
    }

    return new_board;
}

fn kings_cross_positive(first_piece:u64, pieces:&mut u64, piece_bitmap:&mut u64, color_map:u64, offset:&OffsetClosure<fn(u64,u64)->u64>)->Option<u64>{
    let moved_piece = (offset.closure)(first_piece, 1);

    if moved_piece & *piece_bitmap != 0{
        return None;
    }
    if (moved_piece & color_map != 0)||(moved_piece == 0)||(moved_piece & offset.usage_mask != 0){
        return None;
    }
    *pieces ^= first_piece|moved_piece;
    *piece_bitmap |= moved_piece;
    return Some(moved_piece);
}

fn find_new_king_move(board:&Board, king_bitmap:&mut u64, color_map:u64)->Option<Board>{
    let mut kings = board.kings & color_map;
    //print_mask(kings, "kings");

    while kings != 0{
        let first_king = 1<<kings.ilog2() as u64;
        //print_mask(first_king, "first king");
        //print_mask(*king_bitmap, "king bitmap");
        if first_king & *king_bitmap != 0{
            return None;
        }

        let king_closure_indexes = [0,1,2,3,4,5,6,7];
        for closure_index in king_closure_indexes{
            if let Some(moved_king) = kings_cross_positive(first_king, &mut kings, king_bitmap, color_map, &SHIFTING_CLOSURES[closure_index]){
                //print_mask(moved_king, "moved king");
                let mut board_copy = board.clone();
                if moved_king & ((board.whites|board.blacks)^color_map) != 0{
                    board_copy.take(moved_king);
                }
                board_copy.kings ^= first_king|moved_king;
                if color_map==board.whites{
                    board_copy.whites ^= first_king|moved_king;
                }else{
                    board_copy.blacks ^= first_king|moved_king;
                }
                return Some(board_copy);
            }
        }
        kings ^= first_king;
        *king_bitmap = (*king_bitmap|first_king) & board.kings & color_map;
    }
    //print_mask(new_board.unwrap().kings, "new king move");
    return None;
}



fn main() {


    println!("let the chess begin");


    let mut board = Board::new();
    print_mask(board.rooks, "rookie smookies");


    while &board.kings.count_ones()==&2{ //yes this game lets you capture the kings before the game ends
        display_board_windows(&board, 0);
        
        board = collect_white_move(board);
        display_board_windows(&board, 0);


        board = find_best_move(board, MOVE_SEARCH_DEPTH);

        //modify move to board
        //calculate best response
        //make response
    }
}

#[allow(dead_code)]
fn print_mask(mask:u64, name:&str){
    println!("{}:", name);
    let range = 0..8;
    for i in range.into_iter().rev(){
        let submask = (mask&(0b11111111<<(i*8)))>>(i*8);
        println!("{:08b}", submask);
    }
    println!(" ");
}


fn display_board_windows(board:&Board, moves:u64){
    let mut board_list:Vec<ColoredString> = Vec::with_capacity(64);
    for i in 0..64{
        let mut square:ColoredString;   
        if board.blacks&(1<<i)!=0{
            if (board.whites|board.blacks)&(1<<i)==0{
                square = "  ".truecolor(8, 11, 11).on_truecolor(119, 149, 86);
            }else if board.kings&(1<<i)!=0{
                square = "♔ ".truecolor(8, 11, 11).on_truecolor(119, 149, 86);
            }else if board.queens&(1<<i)!=0{
                square = "♕ ".truecolor(8, 11, 11).on_truecolor(119, 149, 86);
            }else if board.rooks&(1<<i)!=0{
                square = "♖ ".truecolor(8, 11, 11).on_truecolor(119, 149, 86);
            }else if board.bishops&(1<<i)!=0{
                square = "♗ ".truecolor(8, 11, 11).on_truecolor(119, 149, 86);
            }else if board.knights&(1<<i)!=0{
                square = "♘ ".truecolor(8, 11, 11).on_truecolor(119, 149, 86);
            }else{
                square = "♙ ".truecolor(8, 11, 11).on_truecolor(119, 149, 86);
            } 
        } else{
            if (board.whites|board.blacks)&(1<<i)==0{
                square = "  ".truecolor(255, 255, 255).on_truecolor(119, 149, 86);
            }else if board.kings&(1<<i)!=0{
                square = "♔ ".truecolor(255, 255, 255).on_truecolor(119, 149, 86);
            }else if board.queens&(1<<i)!=0{
                square = "♕ ".truecolor(255, 255, 255).on_truecolor(119, 149, 86);
            }else if board.rooks&(1<<i)!=0{
                square = "♖ ".truecolor(255, 255, 255).on_truecolor(119, 149, 86);
            }else if board.bishops&(1<<i)!=0{
                square = "♗ ".truecolor(255, 255, 255).on_truecolor(119, 149, 86);
            }else if board.knights&(1<<i)!=0{
                square = "♘ ".truecolor(255, 255, 255).on_truecolor(119, 149, 86);
            }else if board.pawns&(1<<i)!=0{
                square = "♙ ".truecolor(255, 255, 255).on_truecolor(119, 149, 86);
            }else{
                print_mask(board.whites, "whites");
                print_mask(board.blacks, "blacks");
                print_mask(1<<i, "piece mask");
                panic!("something in the way");
            }
        }
        if (1<<i)&moves != 0{
            square = square.on_bright_magenta();
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

fn display_board(board:&Board, moves:u64){
    let mut board_list:Vec<ColoredString> = Vec::with_capacity(64);
    for i in 0..64{
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
        if (1<<i)&moves != 0{
            square = square.on_bright_magenta();
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

fn collect_white_move(mut board:Board)->Board{
    let mut row = 9;
    let mut col = 9;
    while board.whites&(1<<(8*(col-1)+8-row))==0{
        let mut input:String = "".to_owned();
        println!("Select piece");
        std::io::stdin().read_line(&mut input).unwrap();
        println!("{}", input);
        if input.chars().count() != 4{
            println!("invalid square");
            continue;
        }
        let mut iter = input.chars();
        row = iter.next().unwrap() as u32 - 96;
        col = iter.next().unwrap() as u32 - 48;
    }

    println!("{}, {}", row, col);
    let piece_mask = 1<<(8*(col-1)+8-row);
    let move_squares:u64 = possible_white_moves(&board, piece_mask);

    println!("moved squares{}", move_squares);
    display_board_windows(&board, move_squares);

    row = 9;
    col = 9;
    let mut moved_piece_mask = 0;
    while moved_piece_mask&move_squares==0{
        let mut input:String = "".to_owned();
        println!("Select square");
        std::io::stdin().read_line(&mut input).unwrap();
        println!("{}", input);
        if input.chars().count() != 4{
            println!("invalid square");
            continue;
        }
        let mut iter = input.chars();
        row = iter.next().unwrap() as u32 - 96;
        col = iter.next().unwrap() as u32 - 48;
        moved_piece_mask = 1<<(8*(col-1)+8-row);   
    }
    println!("{}, {}", row, col);
    if moved_piece_mask&board.blacks != 0{
        board.take(moved_piece_mask);
    }
    move_white_piece(&mut board, piece_mask, moved_piece_mask);
    //print_mask(board.pawns, "pawns");
    // mark and display the possible moves from move_squares
    // collect second input, move piece and return the board
    return board;
}

fn possible_white_moves(board:&Board, piece_mask:u64)->u64{

    // figure out what piece it is
    // use engine to find possible next positions
    // match difference between posissions and the current
    // make those into bitmaps and return them

    let mut moves = 0u64;
    if piece_mask & board.pawns != 0{
        let mut pawn_bitmap = !(piece_mask|piece_mask<<8|piece_mask<<16);
        while let Some(new_board) = find_new_white_pawn_move(board, &mut pawn_bitmap){
            let new_move = (board.pawns&board.whites)^(new_board.pawns&new_board.whites)^piece_mask;
            moves |= new_move;
        }
    }else if piece_mask & board.knights != 0{
        let mut knight_bitmap = board.knights^piece_mask;
        while let Some(new_board) = find_new_knight_move(board, &mut knight_bitmap, board.whites){
            let new_move = (board.knights&board.whites)^(new_board.knights&new_board.whites)^piece_mask;
            moves |= new_move;
        }
    } else if piece_mask & board.bishops != 0{
        println!("recognized bishop");
        let mut bishop_bitmap = board.bishops^piece_mask;
        //print_mask(bishop_bitmap, "bishop bitmap");
        while let Some(new_board) = find_new_bishop_move(board, &mut bishop_bitmap, board.whites){
            //println!("found move");
            let new_move = (board.bishops&board.whites)^(new_board.bishops&new_board.whites)^piece_mask;
            moves |= new_move;
        }
    } else if piece_mask & board.rooks != 0{
        let mut rook_bitmap = board.rooks^piece_mask;
        while let Some(new_board) = find_new_rook_move(board, &mut rook_bitmap, board.whites){
            let new_move = (board.rooks&board.whites)^(new_board.rooks&new_board.whites)^piece_mask;
            moves |= new_move;
        }
    } else if piece_mask & board.queens != 0{
        let mut queen_bitmap = 0;
        while let Some(new_board) = find_new_queen_move(board, &mut queen_bitmap, board.whites){
            let new_move = (board.queens&board.whites)^(new_board.queens&new_board.whites)^piece_mask;
            moves |= new_move;
        }
    } else if piece_mask & board.kings != 0{
        let mut king_bitmap = 0;
        while let Some(new_board) = find_new_king_move(board, &mut king_bitmap, board.whites){
            let new_move = (board.kings&board.whites)^(new_board.kings&new_board.whites)^piece_mask;
            moves |= new_move;
        }
    } 

    return moves;
}

#[inline]
fn move_white_piece(board:&mut Board, piece_mask:u64, moved_piece_mask:u64){
    board.whites ^= piece_mask|moved_piece_mask;
    if piece_mask & board.pawns != 0{
        board.pawns ^= piece_mask|moved_piece_mask;
    }else if piece_mask & board.knights != 0{
        board.knights ^= piece_mask|moved_piece_mask;
    }else if piece_mask & board.bishops != 0{
        board.bishops ^= piece_mask|moved_piece_mask;
    }else if piece_mask & board.rooks != 0{
        board.rooks ^= piece_mask|moved_piece_mask;
    }else if piece_mask & board.queens != 0{
        board.queens ^= piece_mask|moved_piece_mask;
    }else{
        board.kings ^= piece_mask|moved_piece_mask;
    }
}