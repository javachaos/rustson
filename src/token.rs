const MIN_WORD_LENGTH: usize = 1;
const MAX_WORD_LENGTH: usize = 5;
const MAX_HASH_VALUE: usize = 62;

#[repr(u8)]
#[derive(Debug, PartialOrd, PartialEq, Clone)]
pub(crate) enum Token {//sparse with none's for better hash performance (perfect hash)
    NONE0, T, COLON, DASH, TRUE, NONE1, N, NINE, COMMA, NULL, NONE2, F, EIGHT, PLUS, NONE3, FALSE,
    RBRACE, SEVEN, DQUOTE, NONE4, NONE5, LBRACE, SIX, NONE6, NONE7, NONE8, U, FIVE, NONE9, NONE10,
    NONE11, R, FOUR, NONE12, NONE13, NONE14, E, THREE, NONE15, NONE16, NONE17, B, TWO, NONE18,
    NONE19, NONE20, RBRAC, ONE, NONE21, NONE22, NONE23, BACKSLASH, ZERO, NONE24, NONE25, NONE26,
    LBRAC, FWDSLASH, NONE27, NONE28, NONE29, UE, PER
}
 const SYMBOLS: &'static [&'static str] = &["", "t", ":", "-", "true", "", "n", "9", ",", "null",
    "", "f", "8", "+", "", "false", "}", "7", "\"", "", "", "{", "6", "", "", "", "u", "5", "", "",
    "", "r", "4", "", "", "", "e", "3", "",  "", "", "b", "2", "", "", "", "]", "1", "", "", "",
    "\\", "0", "", "", "",  "[", "/", "", "", "", "E", "."];

 const ASSOC: [u8; 256] = [63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63,
    63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 17, 63, 63, 63, 63, 63, 63, 63,
    63, 12,  7,  2, 61, 56, 51, 46, 41, 36, 31, 26, 21, 16, 11,  6,  1, 63, 63, 63, 63, 63, 63, 63,
    63, 63, 63, 60, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63,
    63, 55, 50, 45, 63, 63, 63, 63, 40, 63, 63, 35, 10, 63, 63, 63, 63, 63, 63, 63,  5, 63, 63, 63,
    30, 63,  0, 25, 63, 63, 63, 63, 63, 20, 63, 15, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63,
    63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63,
    63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63,
    63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63,
    63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63,
    63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63, 63];

fn from_index(index: usize) -> Option<Token> {
    if index < MAX_HASH_VALUE {
        Some(unsafe { std::mem::transmute(index as u8) })
    } else {
        None
    }
}

fn hash(string: &str, len: i32) -> usize {
    (len + ASSOC[string.as_bytes()[0] as usize] as i32) as usize
}

impl Token {
    pub(crate) fn has_symbol(sym: &str) -> Token {
        if sym.len() <= MAX_WORD_LENGTH && sym.len() >= MIN_WORD_LENGTH {
            let key = hash(sym, sym.len() as i32);
            if key <= MAX_HASH_VALUE {
                let s: &str = SYMBOLS[key];
                if s.len() < 1 {
                    return Token::NONE1;
                }
                if sym.as_bytes()[0] == s.as_bytes()[0]
                    && sym.as_bytes().iter()
                    .zip(s.as_bytes().iter()).all(|(a, b)| a == b) {
                    return from_index(key).unwrap();
                }
            }
        }
        Token::NONE1
    }

    pub(crate) fn get_digit(&self) -> char {
        match self {
            Token::ZERO  => '0',
            Token::ONE   => '1',
            Token::TWO   => '2',
            Token::THREE => '3',
            Token::FOUR  => '4',
            Token::FIVE  => '5',
            Token::SIX   => '6',
            Token::SEVEN => '7',
            Token::EIGHT => '8',
            Token::NINE  => '9',
            _ => '_'
        }
    }

    pub(crate) fn is_none(&self) -> bool {
        match self {
            Token::NONE0 => true,
            Token::NONE1 => true,
            Token::NONE2 => true,
            Token::NONE3 => true,
            Token::NONE4 => true,
            Token::NONE5 => true,
            Token::NONE6 => true,
            Token::NONE7 => true,
            Token::NONE8 => true,
            Token::NONE9 => true,
            Token::NONE10 => true,
            Token::NONE11 => true,
            Token::NONE12 => true,
            Token::NONE13 => true,
            Token::NONE14 => true,
            Token::NONE15 => true,
            Token::NONE16 => true,
            Token::NONE17 => true,
            Token::NONE18 => true,
            Token::NONE19 => true,
            Token::NONE20 => true,
            Token::NONE21 => true,
            Token::NONE22 => true,
            Token::NONE23 => true,
            Token::NONE24 => true,
            Token::NONE25 => true,
            Token::NONE26 => true,
            Token::NONE27 => true,
            Token::NONE28 => true,
            Token::NONE29 => true,
            _ => false
        }
    }

    pub(crate) fn is_digit(&self) -> bool {
        match self {
            Token::ZERO  => true,
            Token::ONE   => true,
            Token::TWO   => true,
            Token::THREE => true,
            Token::FOUR  => true,
            Token::FIVE  => true,
            Token::SIX   => true,
            Token::SEVEN => true,
            Token::EIGHT => true,
            Token::NINE  => true,
            _ => false
        }
    }

    pub(crate) fn ordinal(&self) -> u8 {
        unsafe { *(self as *const Self as *const u8) }
    }
}