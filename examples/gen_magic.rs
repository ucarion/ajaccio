extern crate ajaccio;
extern crate rand;

use ajaccio::magic::{find_bishop_magic, find_rook_magic};

fn main() {
    println!("Rook magics:");

    for square_index in 0..64 {
        println!("{:?}", find_rook_magic(square_index));
    }

    println!("Bishop magics:");

    for square_index in 0..64 {
        println!("{:?}", find_bishop_magic(square_index));
    }
}
