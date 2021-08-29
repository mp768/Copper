use std::fmt;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Token {
    /*  
     I'll be honest, I'm not so smart to have made this regex. 
     This just came from the official tests where a literal string regex was done.
     I just really needed to know how to regex strings (especially with basically no experience with regex).
    */
    Str(String),
    Int(i64),
    Uint(u64),
    Decimal(f64),

    If,
    Else,
    For,
    Return,

    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,

    Comma, 
    Plus,
    PlusEqual,
    Minus,
    MinusEqual,
    Star,
    StarEqual,
    Slash,
    SlashEqual,
    Dot,
    Semicolon,

    Func,
    While,

    Colon,
    ColonEqual,
    Equal,
    EqualEqual,
    Not,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    CmpOr,

    CmpAnd,

    CmpTrue,

    CmpFalse,

    Var,

    Identifer(String),

    TypeInt,
    TypeUint,
    TypeDecimal,
    TypeString,
    TypeBool,
    TypeAny,

    NewLine,
    Comment,

    ERROR,
}

macro_rules! two_wide_token {
    ($self:expr, $condition:expr, $first:expr, $second:expr) => {
        if $self.peek() != $condition {
            return Some($first);
        } else {
            $self.end += 1;
            return Some($second);
        }
    };
}

pub struct Lexer {
    source: String,
    pub line: usize,
    start: usize,
    end: usize,
}

impl Lexer {
    #[inline]
    fn at_end(&self) -> bool {
        return self.start >= self.source.len() || self.end >= self.source.len();
    }

    #[inline]
    fn peek(&self) -> &str {
        if self.at_end() {
            return "%$%^";
        }

        return &self.source[self.end..self.end+1];
    }

    #[inline]
    fn peek_next(&self) -> &str {
        if self.at_end() {
            return "@#$^";
        }

        return &self.source[self.end+1..self.end+2];
    }

    fn parse_string(&mut self) -> Option<Token> {
        let mut contents: String = String::new();

        while self.peek() != "\"" && !self.at_end() {
            if self.peek() == "\\" {
                match self.peek_next() {
                    "\"" => contents.push('\"'),
                    "\\" => contents.push('\\'),
                    "n" => contents.push('\n'),
                    "t" => contents.push('\t'),
                    "r" => contents.push('\r'),
                    "\'" => contents.push('\''),
                    _ => panic!("Invalid character token with '\\'"),
                }
                self.end += 1;
            } else {
                contents.push_str(self.peek());
            }
            self.end += 1;
        }
        
        if self.at_end() {
            return Some(Token::ERROR);
        }
        self.end += 1;

        return Some(Token::Str(contents));
    }
    
    fn parse_identifer(&mut self) -> Option<Token> {
        while self.peek().chars().all(|x| x.is_alphanumeric() || x == '_') {
            self.end += 1;
        }

        let identifer = &self.source[self.start..self.end];

        match identifer {
            "or" => return Some(Token::CmpOr),
            "and" => return Some(Token::CmpAnd),
            "true" => return Some(Token::CmpTrue),
            "false" => return Some(Token::CmpFalse),
            "for" => return Some(Token::For),
            "func" => return Some(Token::Func),
            "while" => return Some(Token::While),
            "not" => return Some(Token::Not),
            "var" => return Some(Token::Var),
            "if" => return Some(Token::If),
            "else" => return Some(Token::Else),
            "return" => return Some(Token::Return),
            "int" => return Some(Token::TypeInt),
            "uint" => return Some(Token::TypeUint),
            "decimal" => return Some(Token::TypeDecimal),
            "string" => return Some(Token::TypeString),
            "any" => return Some(Token::TypeAny),
            "bool" => return Some(Token::TypeBool),
            _ => return Some(Token::Identifer(identifer.to_string())),
        }
    }

    fn parse_numeric(&mut self) -> Option<Token> {
        while self.peek().chars().all(|x| x.is_numeric()) {
            self.end += 1;
        }

        let mut decimal = false;

        if self.peek() == "." && self.peek_next().chars().all(|x| x.is_numeric()) {
            self.end += 1;
            decimal = true;

            while self.peek().chars().all(|x| x.is_numeric()) {
                self.end += 1;
            }
        }

        if decimal {
            return Some(Token::Decimal(self.source[self.start..self.end].parse().unwrap_or(0.0)));
        } else {
            return Some(Token::Int(self.source[self.start..self.end].parse().unwrap_or(0)));
        }
    }

    pub fn next(&mut self) -> Option<Token> {
        if self.at_end() {
            return None;
        }

        self.start = self.end;
        self.end += 1;

        let mut char = &self.source[self.start..self.end];
            if self.at_end() {
                return None
            }

        let mut in_loop: usize = 0;

        while !self.at_end() {
            if in_loop > 1000 {
                return None;
            }

            in_loop += 1;

            match char {
                " " | "\t" | "\r" => {
                    self.start = self.end;
                    self.end += 1;
                    char = &self.source[self.start..self.end];
                }
                "/" => if self.peek() == "/" {
                    while self.peek() != "\n" && !self.at_end() {
                        self.end += 1;
                    }

                    if self.at_end() {
                        return None;
                    }

                    self.end += 1;
                    self.start = self.end;
                    self.end += 1;
                    char = &self.source[self.start..self.end];
                } else {
                    char = "/";
                    break;
                }
                "\n" => {
                    self.line += 1;
                    self.start = self.end;
                    self.end += 1;
                    char = &self.source[self.start..self.end];
                }
                _ => break,
            }
        }

        if self.at_end() {
            return None;
        }

        match char {
            "=" => two_wide_token!(self, "=", Token::Equal, Token::EqualEqual),
            "!" => two_wide_token!(self, "=", Token::Not, Token::NotEqual),
            ">" => two_wide_token!(self, "=", Token::Greater, Token::GreaterEqual),
            "<" => two_wide_token!(self, "=", Token::Less, Token::LessEqual),
            ":" => two_wide_token!(self, "=", Token::Colon, Token::ColonEqual),
            ";" => return Some(Token::Semicolon),
            "," => return Some(Token::Comma),
            "." => return Some(Token::Dot),
            "/" => two_wide_token!(self, "=", Token::Slash, Token::SlashEqual),
            "*" => two_wide_token!(self, "=", Token::Star, Token::StarEqual),
            "+" => two_wide_token!(self, "=", Token::Plus, Token::PlusEqual),
            "-" => two_wide_token!(self, "=", Token::Minus, Token::MinusEqual),
            "(" => return Some(Token::LeftParen),
            ")" => return Some(Token::RightParen),
            "{" => return Some(Token::LeftBrace),
            "}" => return Some(Token::RightBrace),
            "[" => return Some(Token::LeftBracket),
            "]" => return Some(Token::RightBracket),
            "\"" => return self.parse_string(),
            _ => {
                if char.chars().all(|x| x.is_alphabetic() || x == '_') {
                    return self.parse_identifer();
                } else if char.chars().all(|x| x.is_numeric()) {
                    return self.parse_numeric();
                }

                self.end += 1;
                char = &self.source[self.start..self.end];

                match char {
                    "&&" => return Some(Token::CmpAnd),
                    "||" => return Some(Token::CmpOr),
                    _ => return Some(Token::ERROR),
                }
            },
        }
        
        
    }

    pub fn slice(&self) -> String {
        String::from(&self.source[self.start..self.end])
    }

    pub fn new(source: String) -> Self {
        let mut source = source;
        source.push(' ');
        Self {
            source,
            line: 0,
            start: 0,
            end: 0,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.clone() {
            Token::Str(a) => write!(f, "\"{}\"", a),
            Token::Int(a) => write!(f, "{}", a),
            Token::Uint(a) => write!(f, "{}", a),
            Token::Decimal(a) => write!(f, "{}", a),
            Token::If => todo!(),
            Token::Else => todo!(),
            Token::For => todo!(),
            Token::Return => todo!(),
            Token::LeftParen => todo!(),
            Token::RightParen => todo!(),
            Token::LeftBrace => todo!(),
            Token::RightBrace => todo!(),
            Token::LeftBracket => todo!(),
            Token::RightBracket => todo!(),
            Token::Comma => todo!(),
            Token::Plus => write!(f, "+"),
            Token::PlusEqual => write!(f, "+="),
            Token::Minus => write!(f, "-"),
            Token::MinusEqual => write!(f, "-="),
            Token::Star => write!(f, "*"),
            Token::StarEqual => write!(f, "*="),
            Token::Slash => write!(f, "/"),
            Token::SlashEqual => write!(f, "/="),
            Token::Dot => write!(f, "."),
            Token::Semicolon => todo!(),
            Token::Func => todo!(),
            Token::While => todo!(),
            Token::Colon => todo!(),
            Token::ColonEqual => todo!(),
            Token::Equal => write!(f, "="),
            Token::EqualEqual => write!(f, "=="),
            Token::Not => write!(f, "!"),
            Token::NotEqual => write!(f, "!="),
            Token::Greater => write!(f, ">"),
            Token::GreaterEqual => write!(f, ">="),
            Token::Less => write!(f, "<"),
            Token::LessEqual => write!(f, "<="),
            Token::CmpOr => write!(f, "||"),
            Token::CmpAnd => write!(f, "&&"),
            Token::CmpTrue => todo!(),
            Token::CmpFalse => todo!(),
            Token::Var => todo!(),
            Token::Identifer(_) => todo!(),
            Token::TypeInt => todo!(),
            Token::TypeUint => todo!(),
            Token::TypeDecimal => todo!(),
            Token::TypeString => todo!(),
            Token::TypeBool => todo!(),
            Token::TypeAny => todo!(),
            Token::NewLine => todo!(),
            Token::Comment => todo!(),
            Token::ERROR => todo!(),
        }
    }
}