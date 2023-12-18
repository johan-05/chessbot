
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
        return Board {
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
        };
    }

    fn evaluate_position(&self)->i32{


        unimplemented!("amogus");
    }

}

fn print_mask(mask:u64, name:&str){
    println!("{}:", name);
    for i in 0..8{
        let submask = (mask&(0b11111111<<i*8))>>i*8;
        println!("{:08b}", submask);
    }
    println!(" ");
}

fn main() {
    println!("let the chess begin");

    let mut board = Board::new();
    print_mask(board.rooks, "rooks");

}
