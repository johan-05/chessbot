extern crate colored;

use colored::*;
use colored::ColoredString;
struct Board{
    kings:u64,
    queens:u64,         //u64 bitmaps of all piece-types
    rooks:u64,
    bishops:u64,
    knights:u64,
    pawns:u64,
    whites:u64,         //colors are tracked with color-bitmasks
    blacks:u64,         //state numbers store additional state structured like this (little endian):
    white_state:u32,    //  bit 0-3 castling abilities og kings and rooks, init is 1011
    black_state:u32,    //  bit 4-15  50-rule, number of moves with only king on the board   
}                       //  bit 16-23  index of piece susceptible to en passant
                        // 0000 0000 0000 00000000 00000000 1011 
/*
abcdefgh
11111111
11111111
00000000
00000000
00000000
00000000
00000000
00000000





*/

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
            white_state: 11,
            black_state: 11, 
        }
    }

    fn evaluate_position(&self)->i32{


        unimplemented!("amogus");
    }

}

fn print_mask(mask:u64, name:&str){
    println!("{}:", name);
    for i in 0..8{
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
            }else{
                if (board.whites|board.blacks)&(1<<i)==0{
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
            board_list[8*i+0], board_list[8*i+1],
            board_list[8*i+2], board_list[8*i+3],
            board_list[8*i+4], board_list[8*i+5],
            board_list[8*i+6], board_list[8*i+7]
        )
    }
    println!("");
}

fn main() {
    println!("let the chess begin");

    let board = Board::new();
    print_mask(board.knights, "pawns");

    while board.kings.count_ones()==2{ //yes this game lets you capture the kings before the game ends
        display_board(&board);
        //input move from white

        //modify move to board
        //calculate best response
        //make response
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
