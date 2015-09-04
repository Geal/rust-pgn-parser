extern crate pgnparser;

use pgnparser::*;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

fn main() {


    let path = Path::new("examples/game.pgn");
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open game.pgn: {}", Error::description(&why)),
        Ok(file) => file,
    };

    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read game.pgn: {}", Error::description(&why)),
        Ok(_) => ()
    };

    let game_str = &s[..];

    let g = PGNGame::from_str(game_str);

    match g {
        Ok(game) => {
            fn print_half_move<'a>(p: &'a str, capture: bool, row_from: Option<&'a str>, col_from: Option<&'a str>, square: &'a str, promotion: Option<&'a str>) {
                let cap = if capture {"x"} else {""};
                let from = format!("{}{}", row_from.unwrap_or(""), col_from.unwrap_or(""));
                let prom = promotion.map(|p| format!("={}", p)).unwrap_or(String::from(""));
                print!("{}{}{}{}{}", p, from, cap, square, prom);
            }
            for (k, v) in game.headers.iter() {
                println!("{}: {}", k, v);
            }
            for (i, m) in game.moves.iter().cloned().enumerate() {
                let is_white_turn = i % 2 == 0;
                if is_white_turn {
                    print!("{}. ", i/2 + 1);
                }
                match m {
                    HalfMove::Regular{piece, capture, col_from, row_from, square, promotion} => {
                        let p = if is_white_turn { piece.as_white() } else { piece.as_black() };
                        let prom = if is_white_turn { promotion.as_ref().map(Piece::as_white) } else { promotion.as_ref().map(Piece::as_black) };
                        print_half_move(p, capture,row_from, col_from, square, prom);
                    },

                    HalfMove::KingsideCastling => print!("O-O"),
                    HalfMove::QueensideCastling => print!("O-O-O"),
                    _ => print!("Something else")
                }
                if is_white_turn {
                    print!("\t");
                } else {
                    print!("\n");
                }
            }
        },
        Err(x) => panic!(x)
    }
}
