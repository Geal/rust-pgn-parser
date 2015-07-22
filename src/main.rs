#[macro_use]
extern crate nom;
extern crate regex;

use nom::{IResult, space, digit, alphanumeric, multispace, is_alphanumeric, is_space};
use nom::IResult::*;
use nom::Err::*;
use regex::{Regex};
use std::str;

#[derive(PartialEq,Eq,Debug)]
pub enum Piece {
    King,
    Queen,
    Bishop,
    Knight,
    Rook,
    Pawn
}

#[derive(PartialEq,Eq,Debug)]
pub enum HalfMove<'a> {
    Regular {
        piece: Piece,
        capture: bool,
        col_from: Option<&'a str>,
        row_from: Option<&'a str>,
        square: &'a str,
        promotion: Option<Piece>
    },
    KingsideCastling,
    QueensideCastling
}


#[derive(PartialEq,Eq,Debug)]
pub struct Move<'a> {
    id: &'a str
}

fn piece_parser(p: &str) -> Piece {
    match p {
        "K" => Piece::King,
        "Q" => Piece::Queen,
        "B" => Piece::Bishop,
        "N" => Piece::Knight,
        "R" => Piece::Rook,
        _ => panic!("Impossible")
    }
}

fn pawn_move(input:&[u8]) -> IResult<&[u8], HalfMove > {
    let mut index = 0;
    let re = Regex::new(r"^([a-h][1-8])(?:=([QBNR]))?$").unwrap();

    for idx in 0..input.len() {
        if is_space(input[idx]) {
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
                        promotion: cap.at(2).map(piece_parser)
                    });
    }
    return Error(Position(0, input));
}

fn pawn_capture(input:&[u8]) -> IResult<&[u8], HalfMove> {
    let mut index = 0;
    let re = Regex::new(r"^([a-h])x([a-h][1-8])(?:=([QBNR]))?$").unwrap();

    for idx in 0..input.len() {
        if is_space(input[idx]) {
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
                        promotion: cap.at(3).map(piece_parser)
                    });
    }
    return Error(Position(0, input));
}

fn piece_move(input:&[u8]) -> IResult<&[u8], HalfMove > {
    let mut index = 0;
    let re = Regex::new(r"^([KQBNR])([a-h])?([1-8])?([a-h][1-8])$").unwrap();

    for idx in 0..input.len() {
        if is_space(input[idx]) {
            break;
        }
        index = index + 1;
    }

    let text = str::from_utf8(&input[..index]).unwrap_or("");

    if re.is_match(text) {
        let cap = re.captures(text).unwrap();
        return Done(&input[index..],
                    HalfMove::Regular {
                        piece: piece_parser(cap.at(1).unwrap()),
                        capture: false,
                        col_from: cap.at(2),
                        row_from: cap.at(3),
                        square: cap.at(4).unwrap(),
                        promotion: None
                    });
    }
    return Error(Position(0, input));
}

fn piece_capture(input:&[u8]) -> IResult<&[u8], HalfMove > {
    let mut index = 0;
    let re = Regex::new(r"^([KQBNR])([a-h])?([1-8])?x([a-h][1-8])$").unwrap();

    for idx in 0..input.len() {
        if is_space(input[idx]) {
            break;
        }
        index = index + 1;
    }

    let text = str::from_utf8(&input[..index]).unwrap_or("");

    if re.is_match(text) {
        let cap = re.captures(text).unwrap();
        return Done(&input[index..],
                    HalfMove::Regular {
                        piece: piece_parser(cap.at(1).unwrap()),
                        capture: true,
                        col_from: cap.at(2),
                        row_from: cap.at(3),
                        square: cap.at(4).unwrap(),
                        promotion: None
                    });
    }
    return Error(Position(0, input));
}

named!(san_move<&[u8], HalfMove>,
       alt!(
           pawn_move |
           pawn_capture |
           piece_move |
           piece_capture
       ));


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

named!(mve<&[u8], Move>,
       chain!(
           i: map_res!(digit, str::from_utf8) ~
           tag!(".") ~
           multispace? ~
           san_move ~
           space ~
           multispace? ~
           san_move ~
           multispace?,
       || {Move { id: i } } ));

fn main() {

    assert_eq!(mve(b"7.e4 e5"), Done(&b""[..], Move{id: "7"}));

    let m3 = HalfMove::Regular {
        piece: Piece::Queen,
        capture: false,
        col_from: None,
        row_from: None,
        square: "f2",
        promotion: None };
    assert_eq!(piece_move(b"Qf2"), Done(&b""[..], m3));

    let m4 = HalfMove::Regular {
        piece: Piece::Bishop,
        capture: false,
        col_from: Some("g"),
        row_from: None,
        square: "a8",
        promotion: None };
    assert_eq!(piece_move(b"Bga8"), Done(&b""[..], m4));

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
        capture: false,
        col_from: Some("c"),
        row_from: Some("4"),
        square: "f4",
        promotion: None };
    assert_eq!(piece_move(b"Rc4f4"), Done(&b""[..], m6));

    let m7 = HalfMove::Regular {
        piece: Piece::King,
        capture: true,
        col_from: None,
        row_from: None,
        square: "f2",
        promotion: None };
    assert_eq!(piece_capture(b"Kxf2"), Done(&b""[..], m7));

    let m8 = HalfMove::Regular {
        piece: Piece::Queen,
        capture: true,
        col_from: Some("h"),
        row_from: None,
        square: "b5",
        promotion: None };
    assert_eq!(piece_capture(b"Qhxb5"), Done(&b""[..], m8));

    let m100 = HalfMove::Regular {
        piece: Piece::Pawn,
        capture: false,
        col_from: None,
        row_from: None,
        square: "e3",
        promotion: None };
    assert_eq!(san_move(b"e3 f5"), Done(&b" f5"[..], m100));

    /*
    assert_eq!(hmve(b"Rd7"), Done(&b"d7"[..], HalfMove::Regular{piece: Piece::Rook, capture: false}));
    assert_eq!(hmve(b"g5"), Done(&b"g5"[..], HalfMove::Regular{piece: Piece::Pawn, capture: false}));
    assert_eq!(hmve(b"Bxf2"), Done(&b"f2"[..], HalfMove::Regular{piece: Piece::Bishop, capture: true}));
     */
}

#[cfg(test)]
mod tests {
    use super::{headers, alphanumeric_or_underscore, pawn_move, pawn_capture};
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
}
