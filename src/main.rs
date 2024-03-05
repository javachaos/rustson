mod token;

use crate::JsonValue::JsonString;
use std::string::String;
use std::collections::HashSet;
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufRead, Read};
use std::os::unix::fs::FileExt;
use std::rc::Rc;
use crate::token::Token;
use std::borrow::BorrowMut;
use std::hash::{BuildHasher, Hasher};
use crate::JsonValue::{Array, Number, Object};

const BUFFER_LEN: usize = 8;// 64bits
const NEW_LINE : char = '\n';
const HEXIDECIMAL_RADIX: u32 = 16;

#[derive(Debug, Eq, PartialEq, Clone)]
enum JsonValue {
    Null(i32),
    True,
    False,
    Number(String),
    JsonString(String),
    Object(HashSet<(String, JsonValue)>),  //unordered set
    Array(Vec<JsonValue>)                  //ordered list
}

impl std::hash::Hash for JsonValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            JsonValue::Null(_) => 0.hash(state),
            JsonValue::True => true.hash(state),
            JsonValue::False => false.hash(state),
            JsonString(n) | Number(n) => n.hash(state),
            Object(o) => o.iter().for_each(|x| x.hash(state)),
            Array(a) => a.hash(state)
        }
    }
}

impl JsonValue {
    fn from_int(value: i32) -> JsonValue {
        Number(value.to_string())
    }
    fn from_float(value: f32) -> JsonValue {
        Number(value.to_string())
    }
    fn from_str(value: String) -> JsonValue {
        JsonString(value)
    }
    fn from_bool(value: bool) -> JsonValue {
        match value {
            true => JsonValue::True,
            false => JsonValue::False
        }
    }
    fn from_vec(value: Vec<JsonValue>) -> JsonValue {
        Array(value)
    }
    fn from_set(value: HashSet<(String, JsonValue)>) -> JsonValue {
        Object(value)
    }
}

struct JsonParser {
    file: Rc<File>,
    objects: JsonValue,
    read_buffer: [u8; BUFFER_LEN],
    curr_token: Token, // the current token
    prev_char: char,   // the previous character
    curr_char: char,   // current character
    buff_index: usize, // character position in the buffer
    read_index: usize, // character position in the file,
    // will always be (read_index % BUFFER_LEN)
    look_ahead_index: usize,
    line: usize
}

impl JsonParser {
    fn new(filename: &'static str) -> JsonParser {
        JsonParser {
            file: Rc::from(File::open(filename).unwrap()),
            objects: Object(HashSet::new()),
            read_buffer: [0; BUFFER_LEN],
            curr_token: Token::NONE0,
            prev_char: 0 as char,
            curr_char: 0 as char,
            buff_index: 0,
            read_index: 0,
            look_ahead_index: 0,
            line: 0
        }
    }

    fn get_next_char(&mut self) -> char {
        self.reset_lookahead();
        if self.buff_index >= BUFFER_LEN {
            self.read_index += BUFFER_LEN;
            self.buff_index = 0;
        }
        self.file.read_exact_at(&mut self.read_buffer, self.read_index as u64).unwrap();
        self.prev_char = self.curr_char;
        self.curr_char = char::from(self.read_buffer[self.buff_index]);
        self.buff_index += 1;
        if self.curr_char == NEW_LINE {
            self.line += 1
        }
        self.curr_char
    }

    fn reset_lookahead(&mut self) {
        self.look_ahead_index = 0;
    }

    fn peek_next_char(&mut self) -> char {
        let mut buff_index = self.buff_index;
        let mut read_index = self.read_index;
        let mut read_buff = self.read_buffer.clone();
        if buff_index >= BUFFER_LEN {
            read_index += BUFFER_LEN;
            buff_index = 0;
        }
        self.file.read_exact_at(&mut read_buff, (read_index + self.look_ahead_index) as u64)
            .unwrap();
        self.look_ahead_index += 1;
        char::from(read_buff[buff_index])
    }

    fn get_next_token(&mut self) -> Token {
        self.whitespace();//skip whitespace
        let c = self.get_next_char();
        if c == 't' || c == 'f' {
            //read next character and determine if we need to test for true or false
            let n = self.peek_next_char();
            if n == 'r' {
                let u = self.peek_next_char();
                let e = self.peek_next_char();
                if u == 'u' && e == 'e' {
                    self.reset_lookahead();
                    return Token::TRUE;
                }
            }
            if n == 'a' {
                let l = self.peek_next_char();
                let s = self.peek_next_char();
                let e = self.peek_next_char();
                if l == 'l' && s == 's' && e == 'e' {
                    self.reset_lookahead();
                    return Token::FALSE;
                }
            }
        }
        self.curr_token = Token::has_symbol(char::from(c).to_string().as_str());
        self.curr_token.clone()
    }

    fn whitespace(&mut self) {
        loop {
            let c = self.peek_next_char();
            if c != ' ' && c != '\t' && c != '\r' && c != '\n' {
                break;
            } else {
                self.get_next_char();
            }
        }
    }

    ///
    /// Consume next token, and return true if the current token is equal to t
    /// # Args
    /// * `t` - The token under consideration
    ///
    fn accept(&mut self, t: Token, advance: bool) -> bool {
        if advance {
            self.get_next_token();
        }
        self.curr_token == t
    }

    ///
    /// Expect that the current token is equal to t
    /// # Args
    /// * `t` - The token under consideration
    ///
    fn expect(&mut self, t: Token) -> bool {
        if self.accept(t.clone(), false) {
            return true;
        }
        panic!("Expected {:?} at line {:?}, position {:?}.",
               t, self.line, self.read_index + self.buff_index);
    }

    fn object(&mut self) {
        self.get_next_token();
        self.expect(Token::LBRACE);
        self.whitespace();
        if self.accept(Token::RBRACE, false) {
            self.get_next_token();
            return;
        }
        loop {
            self.whitespace();
            self.get_next_token();
            if self.expect(Token::DQUOTE) {
                let s = self.string();
                self.whitespace();
                self.get_next_token();
                self.expect(Token::COLON);
                let v = self.value();//TODO impl value()
               // self.objects.insert((JsonName::new(&s), JsonValue::from_set(v)));
                self.whitespace();
                if self.curr_token == Token::RBRACE {
                    self.get_next_token();
                    self.expect(Token::RBRACE);
                    return;
                }
            }
            if self.accept(Token::COMMA, true) {
                break;
            }
        }
    }

    fn string(&mut self) -> String {
        let mut str_buff: String = "".to_owned();
        let mut prev_char: char = self.curr_char;
        if self.curr_char == '"' {
            self.get_next_char();
        } else {
            panic!("Not a string!")
        }
        if prev_char == '"' && self.curr_char == '"' {
            return str_buff.clone();
        }
        loop {
            str_buff.push(self.curr_char);
            if prev_char == '\\' {
                prev_char = self.curr_char;
                self.get_next_char();
                match prev_char {
                    '"' | '\\' | '/' | 'b' | 'f' | 'n' | 't'  => {
                        str_buff.push(prev_char)
                    }
                    'u' => {
                        str_buff.push(prev_char);
                        if self.get_next_char().is_digit(HEXIDECIMAL_RADIX) {
                            str_buff.push(self.curr_char);
                        } else {
                            panic!("Unexpected hex value: {:?}", self.curr_char)
                        }
                        if self.get_next_char().is_digit(HEXIDECIMAL_RADIX) {
                            str_buff.push(self.curr_char);
                        } else {
                            panic!("Unexpected hex value: {:?}", self.curr_char)
                        }
                        if self.get_next_char().is_digit(HEXIDECIMAL_RADIX) {
                            str_buff.push(self.curr_char);
                        } else {
                            panic!("Unexpected hex value: {:?}", self.curr_char)
                        }
                        if self.get_next_char().is_digit(HEXIDECIMAL_RADIX) {
                            str_buff.push(self.curr_char);
                        } else {
                            panic!("Unexpected hex value: {:?}", self.curr_char)
                        }
                        return str_buff.clone();
                    },
                    _ => { panic!("Unexpected value: {prev_char}") }
                }

            } else if self.get_next_char() == '"' {
                self.expect(Token::DQUOTE);
                break;
            }
        }
        str_buff
    }

    fn value(&mut self) -> HashSet<JsonValue> {
        let mut list = HashSet::new();
        self.whitespace();
        match self.curr_token {
            Token::DQUOTE => {
                self.get_next_token();
                let s = JsonValue::from_str(self.string());
                list.insert(s);
                list
            }
            Token::DASH|Token::ZERO|Token::ONE|Token::TWO|Token::THREE|
            Token::FOUR|Token::FIVE|Token::SIX|Token::SEVEN|Token::EIGHT|Token::NINE => {
                //TODO complete this
                list
            }
            _ => list
        }
    }

}

fn main() {
    let mut p = JsonParser::new("input.json");
    p.object();
}
