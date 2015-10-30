#![crate_name="pgnparser"]

#[macro_use]

extern crate nom;
extern crate regex;

use std::str;
use std::result::Result;
use std::collections::hash_map::HashMap;
use nom::{IResult, ErrorKind, digit, multispace, is_alphanumeric};
use nom::IResult::*;
use nom::Err::*;
use regex::Regex;

#[derive(PartialEq,Eq,Debug,Clone)]
pub struct PGNGame<'a> {
    pub headers: HashMap<&'a str, &'a str>,
    pub moves: Vec<HalfMove<'a>>,
    pub result: GameResult
}

impl<'a> PGNGame<'a> {
    pub fn from_str(gstr: &str) -> Result<PGNGame, &'static str> {
        let g = game(gstr.as_bytes());
        match g {
            Done(_, game) => Ok(game),
            Error(_) => Err("Unable to parse PGN data. The input string is invalid"),
            Incomplete(_) => Err("Unable to parse PGN data. The input string is incomplte")
        }
    }
}

#[derive(PartialEq,Eq,Debug,Clone)]
pub enum Piece {
    King,
    Queen,
    Bishop,
    Knight,
    Rook,
    Pawn
}

impl Piece {
    pub fn as_white(&self) -> &str {
        match *self {
            Piece::King => "♔",
            Piece::Queen => "♕",
            Piece::Bishop => "♗",
            Piece::Knight => "♘",
            Piece::Rook => "♖",
            Piece::Pawn => "",
        }
    }

    pub fn as_black(&self) -> &str {
        match *self {
            Piece::King => "♚",
            Piece::Queen => "♛",
            Piece::Bishop => "♝",
            Piece::Knight => "♞",
            Piece::Rook => "♜",
            Piece::Pawn => "",
        }
    }


    pub fn from_str(p: &str) -> Piece {
        match p {
            "K" => Piece::King,
            "Q" => Piece::Queen,
            "B" => Piece::Bishop,
            "N" => Piece::Knight,
            "R" => Piece::Rook,
            _ => unreachable!()
        }
    }
}

#[derive(PartialEq,Eq,Debug,Clone)]
pub enum GameResult {
    WhiteWon,
    BlackWon,
    Draw,
    Other
}

#[derive(PartialEq,Eq,Debug,Clone)]
pub enum HalfMove<'a> {
    Regular {
        piece: Piece,
        capture: bool,
        col_from: Option<&'a str>,
        row_from: Option<&'a str>,
        square: &'a str,
        promotion: Option<Piece>,
    },
    KingsideCastling,
    QueensideCastling,
    None
}

fn is_space_or_eol(chr:u8) -> bool {
    chr == ' ' as u8 || chr == '\t' as u8 || chr == '\r' as u8 || chr == '\n' as u8
}

fn pawn_move(input:&[u8]) -> IResult<&[u8], HalfMove > {
    let mut index = 0;
    let re = Regex::new(r"^([a-h][1-8])(?:=([QBNR]))?[+#]?$").unwrap();

    for idx in 0..input.len() {
        if is_space_or_eol(input[idx]) {
            break;
        }
        index = index + 1;
    }
    let text = str::from_utf8(&input[..index]).unwrap_or("");
    if re.is_match(text) {
        let cap = re.captures(text).unwrap();
        return Done(&input[index..],
                    HalfMove::Regular {
                        piece: Piece::Pawn,
                        capture: false,
                        col_from: None,
                        row_from: None,
                        square: cap.at(1).unwrap(),
                        promotion: cap.at(2).map(Piece::from_str)
                    });
    }
    return Error(Position(ErrorKind::Custom(0), input));
}

fn pawn_capture(input:&[u8]) -> IResult<&[u8], HalfMove> {
    let mut index = 0;
    let re = Regex::new(r"^([a-h])x([a-h][1-8])(?:=([QBNR]))?[+#]?$").unwrap();

    for idx in 0..input.len() {
        if is_space_or_eol(input[idx]) {
            break;
        }
        index = index + 1;
    }

    let text = str::from_utf8(&input[..index]).unwrap_or("");
    if re.is_match(text) {
        let cap = re.captures(text).unwrap();
        return Done(&input[index..],
                    HalfMove::Regular {
                        piece: Piece::Pawn,
                        capture: true,
                        col_from: cap.at(1),
                        row_from: None,
                        square: cap.at(2).unwrap(),
                        promotion: cap.at(3).map(Piece::from_str)
                    });
    }
    return Error(Position(ErrorKind::Custom(0), input));
}

fn piece_move(input:&[u8]) -> IResult<&[u8], HalfMove > {
    let mut index = 0;
    let re = Regex::new(r"^([KQBNR])([a-h])?([1-8])?(x)?([a-h][1-8])[+#]?$").unwrap();

    for idx in 0..input.len() {
        if is_space_or_eol(input[idx]) {
            break;
        }
        index = index + 1;
    }

    let text = str::from_utf8(&input[..index]).unwrap_or("");

    if re.is_match(text) {
        let cap = re.captures(text).unwrap();
        return Done(&input[index..],
                    HalfMove::Regular {
                        piece: Piece::from_str(cap.at(1).unwrap()),
                        capture: cap.at(4).is_some(),
                        col_from: cap.at(2),
                        row_from: cap.at(3),
                        square: cap.at(5).unwrap(),
                        promotion: None
                    });
    }
    return Error(Position(ErrorKind::Custom(0), input));
}

named!(san<&[u8], HalfMove>,
       chain!(
           s: alt!(
               pawn_move |
               pawn_capture |
               piece_move |
               complete!(tag!("O-O-O")) =>  { |_| return HalfMove::QueensideCastling } |
               complete!(tag!("O-O")) => { |_| return HalfMove::KingsideCastling }
           ) ~
           multispace,
           || s)
       ) ;

fn alphanumeric_or_underscore(input:&[u8]) -> IResult<&[u8], &[u8]> {
    for idx in 0..input.len() {
        if !is_alphanumeric(input[idx]) && input[idx] != 0x5F {
            return Done(&input[idx..], &input[0..idx])
        }
    }
    Done(b"", input)
}

named!(key_value_pair<&[u8], (&str,&str)>,
       chain!(
           tag!("[") ~
           multispace? ~
           k: map_res!(alphanumeric_or_underscore, str::from_utf8) ~
           multispace? ~
           tag!("\"") ~
           v: map_res!(take_until!("\""), str::from_utf8) ~
           tag!("\"") ~
           multispace? ~
           tag!("]") ~
           multispace?,
           || (k, v)
       ));

named!(headers<&[u8], Vec<(&str,&str)> >, many0!(key_value_pair));

named!(mve<&[u8], (HalfMove,HalfMove) >,
       chain!(
           digit ~
           tag!(".") ~
           multispace? ~
           white: san ~
           black: san?,
           || return (white, black.unwrap_or(HalfMove::None))
        ));

named!(moves<&[u8], Vec<HalfMove> >,
       chain!(
           list: many1!( mve ),
           || {
               let mut vec = Vec::new();
               for (white, black) in list {
                   vec.push(white);
                   if black != HalfMove::None {
                       vec.push(black);
                   }
               }
               return vec;
           }
           ));

named!(result<&[u8], GameResult>,
       alt!(
           tag!("1-0") => { |_| GameResult::WhiteWon } |
           tag!("0-1") => { |_| GameResult::BlackWon } |
           tag!("1/2-1/2") => { |_| GameResult::Draw } |
           tag!("*") => { |_| GameResult::Other } )
       );

named!(game<&[u8], PGNGame>,
       chain!(
           headers: headers ~
           multispace? ~
           moves: moves ~
           multispace? ~
           result: result,
           || PGNGame {headers: headers.iter().cloned().collect(), moves: moves, result: result }
       )
);

#[cfg(test)]
mod tests {
    use super::{headers, alphanumeric_or_underscore, pawn_move, pawn_capture, piece_move, mve, moves};
    use super::{Piece, HalfMove};
    use nom::IResult::*;

    #[test]
    fn alphanumeric_or_underscore_tests() {
        assert_eq!(alphanumeric_or_underscore(b"abcd_xxx"), Done(&b""[..], &b"abcd_xxx"[..]));
        assert_eq!(alphanumeric_or_underscore(b"AB_C1_56+XX_12"), Done(&b"+XX_12"[..], &b"AB_C1_56"[..]));
    }

    #[test]
    fn headers_tests() {

        let file = b"[Event \"F/S Return Match\"]
[Site \"?\"]
[Date \"????.??.??\"]
[Round \"?\"]
[White \"Calistri, Tristan\"]
[Black \"Bauduin, Etienne\"]
[Result \"1-0\"]
";

        let res = headers(file);
        match res {
            Done(_, o) => for &(k, v) in o.iter() {
                match k {
                    "Event" => assert_eq!(v, "F/S Return Match"),
                    "Site" => assert_eq!(v, "?"),
                    "Date" => assert_eq!(v, "????.??.??"),
                    "Round" => assert_eq!(v, "?"),
                    "White" => assert_eq!(v, "Calistri, Tristan"),
                    "Black" => assert_eq!(v, "Bauduin, Etienne"),
                    "Result" => assert_eq!(v, "1-0"),
                    _ => assert!(false, "unknown key {}", k)
                }
            },
            _ => assert!(false)
        }
    }

    #[test]
    fn convert_piece_to_string_test() {
        assert_eq!(Piece::King.as_white(), "♔");
        assert_eq!(Piece::Queen.as_white(), "♕");
        assert_eq!(Piece::Bishop.as_white(), "♗");
        assert_eq!(Piece::Knight.as_white(), "♘");
        assert_eq!(Piece::Rook.as_white(), "♖");
        assert_eq!(Piece::Pawn.as_white(), "");
        assert_eq!(Piece::King.as_black(), "♚");
        assert_eq!(Piece::Queen.as_black(), "♛");
        assert_eq!(Piece::Bishop.as_black(), "♝");
        assert_eq!(Piece::Knight.as_black(), "♞");
        assert_eq!(Piece::Rook.as_black(), "♜");
        assert_eq!(Piece::Pawn.as_black(), "");
    }

    #[test]
    fn parse_piece_from_string_test() {
        assert_eq!(Piece::from_str("K"), Piece::King);
        assert_eq!(Piece::from_str("Q"), Piece::Queen);
        assert_eq!(Piece::from_str("B"), Piece::Bishop);
        assert_eq!(Piece::from_str("N"), Piece::Knight);
        assert_eq!(Piece::from_str("R"), Piece::Rook);
    }

    #[test]
    fn pawn_move_tests() {
        let m1 = HalfMove::Regular {
            piece: Piece::Pawn,
            capture: false,
            col_from: None,
            row_from: None,
            square: "e3",
            promotion: None };
        assert_eq!(pawn_move(b"e3"), Done(&b""[..], m1));

        let m2 = HalfMove::Regular {
            piece: Piece::Pawn,
            capture: false,
            col_from: None,
            row_from: None,
            square: "e8",
            promotion: Some(Piece::Queen) };
        assert_eq!(pawn_move(b"e8=Q"), Done(&b""[..], m2));
    }

    #[test]
    fn pawn_capture_tests() {
        let m1 = HalfMove::Regular {
            piece: Piece::Pawn,
            capture: true,
            col_from: Some("e"),
            row_from: None,
            square: "f5",
            promotion: None };
        assert_eq!(pawn_capture(b"exf5"), Done(&b""[..], m1));

        let m2 = HalfMove::Regular {
            piece: Piece::Pawn,
            capture: true,
            col_from: Some("a"),
            row_from: None,
            square: "b8",
            promotion: Some(Piece::Rook) };
        assert_eq!(pawn_capture(b"axb8=R"), Done(&b""[..], m2));
    }

    #[test]
    fn piece_move_tests() {
        let m1 = HalfMove::Regular {
            piece: Piece::Queen,
            capture: false,
            col_from: None,
            row_from: None,
            square: "f2",
            promotion: None };
        assert_eq!(piece_move(b"Qf2"), Done(&b""[..], m1));

        let m2 = HalfMove::Regular {
            piece: Piece::Queen,
            capture: true,
            col_from: None,
            row_from: None,
            square: "f2",
            promotion: None };
        assert_eq!(piece_move(b"Qxf2"), Done(&b""[..], m2));

        let m3 = HalfMove::Regular {
            piece: Piece::Bishop,
            capture: false,
            col_from: Some("g"),
            row_from: None,
            square: "a8",
            promotion: None };
        assert_eq!(piece_move(b"Bga8"), Done(&b""[..], m3));

        let m4 = HalfMove::Regular {
            piece: Piece::Bishop,
            capture: true,
            col_from: Some("g"),
            row_from: None,
            square: "a8",
            promotion: None };
        assert_eq!(piece_move(b"Bgxa8"), Done(&b""[..], m4));

        let m5 = HalfMove::Regular {
            piece: Piece::Rook,
            capture: false,
            col_from: None,
            row_from: Some("1"),
            square: "e1",
            promotion: None };
        assert_eq!(piece_move(b"R1e1"), Done(&b""[..], m5));

        let m6 = HalfMove::Regular {
            piece: Piece::Rook,
            capture: true,
            col_from: None,
            row_from: Some("1"),
            square: "e1",
            promotion: None };
        assert_eq!(piece_move(b"R1xe1"), Done(&b""[..], m6));

        let m7 = HalfMove::Regular {
            piece: Piece::Rook,
            capture: false,
            col_from: Some("c"),
            row_from: Some("4"),
            square: "f4",
            promotion: None };
        assert_eq!(piece_move(b"Rc4f4"), Done(&b""[..], m7));

        let m8 = HalfMove::Regular {
            piece: Piece::Rook,
            capture: true,
            col_from: Some("c"),
            row_from: Some("4"),
            square: "f4",
            promotion: None };
        assert_eq!(piece_move(b"Rc4xf4"), Done(&b""[..], m8));
    }

    #[test]
    fn move_tests() {
        let move_white = HalfMove::Regular {
            piece: Piece::Pawn,
            capture: false,
            col_from: None,
            row_from: None,
            square: "e4",
            promotion: None };
        let move_black = HalfMove::Regular {
            piece: Piece::Pawn,
            capture: false,
            col_from: None,
            row_from: None,
            square: "e5",
            promotion: None };
        //assert_eq!(mve(b"7.e4 e5"), Done(&b""[..], (move_white, move_black)));

        assert_eq!(mve(b"7. e4 e5"), Done(&b""[..], (move_white, move_black)));

        let move_white_2 = HalfMove::Regular {
            piece: Piece::Bishop,
            capture: false,
            col_from: None,
            row_from: None,
            square: "f2",
            promotion: None };
        assert_eq!(mve(b"12.Bf2 O-O"), Done(&b""[..], (move_white_2, HalfMove::KingsideCastling)));

        let move_black_2 = HalfMove::Regular {
            piece: Piece::Pawn,
            capture: true,
            col_from: Some("g"),
            row_from: None,
            square: "h4",
            promotion: None };
        assert_eq!(mve(b"4.O-O-O gxh4"), Done(&b""[..], (HalfMove::QueensideCastling, move_black_2)));
    }

    #[test]
    fn moves_tests() {
        let game = b"1. e4 e5 2. Bc4 Bc5 3. Qh5 Nf6 4. Qxf7";
        let move_list = moves(game);
        println!("{:?}", move_list);
    }
}
