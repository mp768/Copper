// Implement what tokens are needed for the macros.

#[derive(Debug, Clone)]
pub enum MiniToken {
    DefMacro,
    ArrowUp,

    Space,
    Identifer(String),
    Numeric(String),
    String(String),
    LeftBrace,
    LeftParen,
    RightBrace,
    RightParen,
    NewLine,
    Exclamation,
    Rest(String),
}

pub struct MacroTokenizer {
    source: String,
    start: usize,
    end: usize,
}

impl MacroTokenizer {
    pub fn new(file: String) -> Self {
        Self {
            source: std::fs::read_to_string(file).unwrap(),
            start: 0,
            end: 0,
        }
    }

    fn peek_next(&self) -> &str {
        if self.at_end() {
            return "你好";
        }

        &self.source[self.end+1..self.end+2]
    }

    #[inline]
    fn peek(&self) -> &str {
        if self.at_end() {
            return "你好";
        }

        &self.source[self.end..self.end+1]
    }

    #[inline]
    fn at_end(&self) -> bool {
        self.end >= self.source.len() || self.start >= self.source.len()
    }

    pub fn new_token(&mut self) -> Option<MiniToken> {
        if self.at_end() {
            return None;
        }

        self.start = self.end;
        self.end += 1;
        let chars = &self.source[self.start..self.end];
        
        while !self.at_end() {
            match chars {
                " " | "\t" | "\r" => {
                    return Some(MiniToken::Space)
                }
                "\n" => {
                    return Some(MiniToken::NewLine);
                }
                _ => break,
            }
        }

        if self.at_end() {
            return None;
        }
        
        match chars {
            "(" => return Some(MiniToken::LeftParen),
            ")" => return Some(MiniToken::RightParen),
            "{" => return Some(MiniToken::LeftBrace),
            "}" => return Some(MiniToken::RightBrace),
            "!" => return Some(MiniToken::Exclamation),
            "\"" => {
                while self.peek() != "\"" && !self.at_end() {
                    self.end += 1;
                }
                self.end += 1;

                if self.at_end() {
                    return None;
                }

                return Some(MiniToken::String(self.source[self.start..self.end].to_string()));
            }
            _ => {
                if chars.chars().all(|x| x.is_alphabetic() || x == '_' || x == '$') {
                    let new_identifer = chars == "$";
                    while self.peek().chars().all(|x| x.is_alphanumeric() || x == '_') {
                        self.end += 1;
                    }

                    let identifer = &self.source[self.start..self.end];

                    return match identifer {
                        "defmacro" => Some(MiniToken::DefMacro),
                        _ => if new_identifer {
                            return Some(MiniToken::Identifer(identifer.to_string()));
                        } else {
                            return Some(MiniToken::Rest(identifer.to_string()));
                        },
                    }
                } else if chars.chars().all(|x| x.is_numeric()) {
                    while self.peek().chars().all(|x| x.is_numeric()) {
                        self.end += 1;
                    }

                    if self.peek() == "." && self.peek_next().chars().all(|x| x.is_numeric()) {
                        self.end += 1;

                        while self.peek().chars().all(|x| x.is_numeric()) {
                            self.end += 1;
                        }
                    }

                    let number = &self.source[self.start..self.end];
                    
                    return Some(MiniToken::Numeric(number.to_string()))
                }

                return Some(MiniToken::Rest(chars.to_string()));
            }
        }

        
    }
}


