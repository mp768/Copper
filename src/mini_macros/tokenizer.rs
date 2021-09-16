// Implement what tokens are needed for the macros.

#[derive(Debug, Clone, PartialEq)]
pub enum MiniToken {
    DefMacro,

    Comma,
    Space,
    Identifer(String),
    CallIdentifer(String),
    LeftBrace,
    LeftParen,
    RightBrace,
    RightParen,
    NewLine,
    SemiColon,
    Rest(String),
    None,
}

#[derive(Clone)]
pub struct MacroTokenizer {
    source: String,
    start: usize,
    end: usize,
}

impl MacroTokenizer {
    pub fn new(file: String) -> Self {
        Self {
            source: if file == "" { "".to_string() } else { 
                let mut string = std::fs::read_to_string(file).unwrap();
                string.push_str("       ");
                string 
            },
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
                "/" => {
                    if self.peek() == "/" {
                        while self.peek() != "\n" && !self.at_end() {
                            self.end += 1;
                        }
                        return Some(MiniToken::Rest(self.source[self.start..self.end].to_string()));
                    } else {
                        return Some(MiniToken::Rest("/".to_string()));
                    }
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
            "," => return Some(MiniToken::Comma),
            ";" => return Some(MiniToken::SemiColon),
            "\"" => {
                while self.peek() != "\"" && !self.at_end() {
                    self.end += 1;
                }
                self.end += 1;

                if self.at_end() {
                    return None;
                }

                return Some(MiniToken::Rest(self.source[self.start..self.end].to_string()));
            }
            _ => {
                if chars.chars().all(|x| x.is_alphabetic() || x == '_' || x == '$') {
                    let new_identifer = chars == "$";
                    while self.peek().chars().all(|x| x.is_alphanumeric() || x == '_') && !self.at_end() {
                        self.end += 1;
                    }

                    let identifer = &self.source[self.start..self.end];

                    return match identifer {
                        "defmacro" => Some(MiniToken::DefMacro),
                        _ => if new_identifer {
                            return Some(MiniToken::Identifer(identifer.to_string()));
                        } else if self.peek() == "!" {
                            self.end += 1;
                            let identifer = &self.source[self.start..self.end];
                            return Some(MiniToken::CallIdentifer(identifer.to_string()));
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
                    
                    return Some(MiniToken::Rest(number.to_string()))
                }

                return Some(MiniToken::Rest(chars.to_string()));
            }
        }
    }
}

macro_rules! check_consume {
    ($self:expr, $token:expr) => {
        match $self.consume($token) {
            Some(x) => x,
            None => return,
        }
    };

    (none => $self:expr, $token:expr) => {
        match $self.consume($token) {
            Some(x) => x,
            None => return None,
        } 
    }
}

macro_rules! clear_space {
    ($self:expr) => {
        //println!("STARTED CLEARING SPACE");
        loop {
            if $self.check(MiniToken::Space) {
                //println!("SPACE CONSUMED");
                $self.consume(MiniToken::Space);
                continue;
            } 

            if $self.check(MiniToken::NewLine) {
                //println!("NEWLINE CONSUMED");
                $self.consume(MiniToken::NewLine);
                continue;
            }

            
            //println!("BROKE OUT OF CLEAR SPACE");
            break;
        }
    };

    (require => $self:expr) => {
        if $self.check(MiniToken::Space) {
            $self.consume(MiniToken::Space);
        } 
        
        else if $self.check(MiniToken::NewLine) {
            $self.consume(MiniToken::NewLine);
        }

        else {
            return;
        }

        clear_space!($self);
    }
}

#[derive(Debug, Clone)]
pub enum MacroAst {
    Block(Vec<MacroAst>),
    Normal(MiniToken),
    Def(String, Vec<MiniToken>, Box<MacroAst>),
    Call(String, Vec<MiniToken>),
}

#[derive(Clone)]
pub struct MacroDefs {
    pub files: Vec<String>,
    pub tokenizer: MacroTokenizer,
    pub defines: Vec<MacroAst>,
    pub file_content: Vec<MacroAst>,
    current_token: MiniToken,
}

impl MacroDefs {
    pub fn new(files: Vec<String>) -> Self {
        Self {
            files,
            tokenizer: MacroTokenizer::new("".to_string()),
            defines: Vec::new(),
            file_content: Vec::new(),
            current_token: MiniToken::None,
        }
    }

    fn advance(&mut self) -> Option<MiniToken> {
        let previous_token = self.current_token.clone();
        self.current_token = match self.tokenizer.new_token() {
            Some(x) => x,
            None => MiniToken::None,
        };

        if self.current_token == MiniToken::None {
            return None;
        }

        Some(previous_token)
    }

    fn block(&mut self) -> MacroAst {
        let mut tokens: Vec<MacroAst> = Vec::new();

        let mut other_token = false;
        while let Some(token) = self.advance() {
            if token == MiniToken::RightBrace {
                break;
            }

            if token == MiniToken::LeftBrace && other_token {
                tokens.push(self.block());
            } else if let MiniToken::CallIdentifer(x) = token {
                let value = match self.parse_call(x) {
                    Some(x) => x,
                    None => MacroAst::Block(Vec::new())
                };

                tokens.push(value);
            } else {
                tokens.push(MacroAst::Normal(token));
            }
            other_token = true;
        }
        
        return MacroAst::Block(tokens);
    }

    fn consume(&mut self, token: MiniToken) -> Option<MiniToken> {
        //println!("DOES {:?} == {:?}", self.current_token, token);

        if self.check(token) {
            return self.advance();
        } else {
            return None;
        }
    }

    fn check(&mut self, token: MiniToken) -> bool {
        self.current_token == token
    }

    fn parse_call(&mut self, name: String) -> Option<MacroAst> {
        clear_space!(self);
        check_consume!(none => self, MiniToken::LeftParen);

        let mut arguments: Vec<MiniToken> = Vec::new();
        if !self.check(MiniToken::RightParen) {
            loop {
                let mut argument: Vec<MiniToken> = Vec::new();
                while match self.current_token {
                    MiniToken::Rest(_) => true,
                    MiniToken::Space => true,
                    MiniToken::NewLine => true,
                    _ => false,
                } {
                    argument.push(match self.advance() {
                        Some(x) => x,
                        None => return None,
                    });
                }


                let mut arg = MiniToken::Rest("".to_string());
                for i in argument {
                    if let MiniToken::Rest(inside) = arg.clone() {
                        match i {
                            MiniToken::Space => arg = MiniToken::Rest(inside + " "),
                            MiniToken::NewLine => arg = MiniToken::Rest(inside + "\n"),
                            MiniToken::Rest(rest) => arg = MiniToken::Rest(inside + &rest.to_owned()),
                            _ => {}
                        }
                    }
                }
                
                //println!("REST: {:?}", arg);
                arguments.push(arg);
                if !self.check(MiniToken::Comma) {
                    break;
                }

                check_consume!(none => self, MiniToken::Comma);
                check_consume!(none => self, MiniToken::Space);
            }
            check_consume!(none => self, MiniToken::RightParen);
        } else {
            check_consume!(none => self, MiniToken::RightParen);
        }

        check_consume!(none => self, MiniToken::SemiColon);

        Some(MacroAst::Call(name, arguments))
    }

    fn parse(&mut self, token: MiniToken) {
        if token == MiniToken::DefMacro {
            //println!("DEF MACRO");
            clear_space!(require => self);
            let identifer = match self.advance() {
                Some(x) => if let MiniToken::Rest(x) = x {
                    x
                } else {
                    return;
                }
                None => return,
            };
            clear_space!(self);
            check_consume!(self, MiniToken::LeftParen);

            let mut arguments: Vec<MiniToken> = Vec::new();
            if !self.check(MiniToken::RightParen) {
                clear_space!(self);
                loop {
                    arguments.push(match self.advance() {
                        Some(x) => if let MiniToken::Identifer(_) = x {
                            x
                        } else if let MiniToken::RightParen = x {
                            break;
                        } else {
                            return;
                        }
                        None => return,
                    });

                    if !self.check(MiniToken::Comma) {
                        break;
                    }
                    check_consume!(self, MiniToken::Comma);
                    clear_space!(self);
                }

                clear_space!(self);
                check_consume!(self, MiniToken::RightParen);
            } else {
                check_consume!(self, MiniToken::RightParen);
            }

            clear_space!(self);

            check_consume!(self, MiniToken::LeftBrace);

            let block = self.block();

            self.defines.push(MacroAst::Def(identifer, arguments, Box::new(block)));

        } else if let MiniToken::CallIdentifer(x) = token {
            let value = match self.parse_call(x) {
                Some(x) => x,
                None => return,
            };

            self.file_content.push(value);
        } else if token == MiniToken::LeftBrace {
            let block = self.block();
            self.file_content.push(block);
        } else {
            self.file_content.push(MacroAst::Normal(token));
        }
    }

    pub fn find_define(&mut self, string: String) -> MacroAst {
        for i in &self.defines {
            if let MacroAst::Def(name, _ , _) = i {
                if string == *name {
                    return i.clone();
                }
            }
        }

        return MacroAst::Def(String::new(), Vec::new(), Box::new(MacroAst::Normal(MiniToken::Space)));
    }

    pub fn compile(&mut self) {
        for i in self.files.clone() {
            self.tokenizer = MacroTokenizer::new(i);

            while let Some(token) = self.advance() {
                self.parse(token);
            }
        }
    }
}

#[derive(Clone)]
pub struct MacroExpander {
    pub defs: MacroDefs,
    pub files: Vec<String>,
    final_source: String,
}

impl MacroExpander {
    pub fn new(files: Vec<String>) -> Self {
        Self {
            defs: MacroDefs::new(files.clone()),
            files,
            final_source: String::new(),
        }
    }

    fn expand_dict_normal(&mut self, i: MiniToken, define_args: Vec<MiniToken>, args: Vec<MiniToken>) {
        match i {
            MiniToken::Comma => self.final_source.push_str(","),
            MiniToken::Space => self.final_source.push_str(" "),
            MiniToken::LeftBrace => self.final_source.push_str("{"),
            MiniToken::LeftParen => self.final_source.push_str("("),
            MiniToken::RightBrace => self.final_source.push_str("}"),
            MiniToken::RightParen => self.final_source.push_str(")"),
            MiniToken::NewLine => self.final_source.push_str("\n"),
            MiniToken::SemiColon => self.final_source.push_str(";"),
            MiniToken::Rest(rest) => self.final_source.push_str(rest.as_str()),
            MiniToken::Identifer(x) => {
                for i in 0..define_args.len() {
                    if let MiniToken::Identifer(t) = &define_args[i] {
                        if *t == *x {
                            //println!("GOT A RESULT : '{}'", x);
                            if let MiniToken::Rest(rest) = &args[i] {
                                self.final_source.push_str(rest.as_str());
                            }
                        }
                    }
                }
            },
            _ => {},
        }
    }

    fn expand_dict_block(&mut self, expr: MacroAst, define_args: Vec<MiniToken>, args: Vec<MiniToken>) {
        self.final_source.push_str("{");

        if let MacroAst::Block(exprs) = expr {
            for i in exprs {
                if let MacroAst::Normal(i) = i {
                    self.expand_dict_normal(i, define_args.clone(), args.clone());
                }

                else if let MacroAst::Block(_) = i {
                    self.expand_dict_block(i, define_args.clone(), args.clone());
                }

                else if let MacroAst::Call(name, arguments_passed) = i {
                    let mut arguments: Vec<MiniToken> = Vec::new();

                    for i in arguments_passed {
                        match i {
                            MiniToken::Rest(_) => arguments.push(i),
                            MiniToken::Identifer(x) => {
                                for i in 0..define_args.len() {
                                    if let MiniToken::Identifer(t) = &define_args[i] {
                                        if *t == *x {
                                            println!("GOT A RESULT : '{}'", x);
                                            let args = args.clone();
                                            if let MiniToken::Rest(rest) = &args[i] {
                                                self.final_source.push_str(rest.as_str());
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {},
                        }
                    }

                    self.expand_dict_call(name, arguments.clone());
                }
            }
        }

        self.final_source.push_str("}");
    }

    fn expand_dict_call(&mut self, name: String, args: Vec<MiniToken>) {
        let mut name = name;
        name.remove(name.len()-1);
        let define = self.defs.find_define(name);

        let define_args = if let MacroAst::Def(_, args, _) = define.clone() {
            args
        } else {
            Vec::new()
        };
        
        if let MacroAst::Def(_, _, block) = define {
            if let MacroAst::Block(content) = *block {
                self.final_source.push_str("{");
                for i in &content {
                    if let MacroAst::Normal(i) = i {
                        self.expand_dict_normal(i.clone(), define_args.clone(), args.clone());
                    }

                    else if let MacroAst::Block(_) = i {
                        self.expand_dict_block(i.clone(), define_args.clone(), args.clone());
                    }

                    else if let MacroAst::Call(name, args) = i {
                        self.expand_dict_call(name.clone(), args.clone());
                    }
                }
                self.final_source.push_str("}");
            }
        }
    }

    fn expand_dict(&mut self, ast: MacroAst) {
        match ast {
            MacroAst::Block(content) => {
                self.final_source.push_str("{");
                for i in &content {
                    self.expand_dict(i.clone());
                }
                self.final_source.push_str("}");
            },
            MacroAst::Normal(rest) => {
                match rest {
                    MiniToken::Comma => self.final_source.push_str(","),
                    MiniToken::Space => self.final_source.push_str(" "),
                    MiniToken::LeftBrace => self.final_source.push_str("{"),
                    MiniToken::LeftParen => self.final_source.push_str("("),
                    MiniToken::RightBrace => self.final_source.push_str("}"),
                    MiniToken::RightParen => self.final_source.push_str(")"),
                    MiniToken::NewLine => self.final_source.push_str("\n"),
                    MiniToken::SemiColon => self.final_source.push_str(";"),
                    MiniToken::Rest(rest) => self.final_source.push_str(rest.as_str()),
                    _ => {},
                }
            },
            MacroAst::Def(_, _, _) => todo!(),
            MacroAst::Call(name, args) => {
                self.expand_dict_call(name, args);
            }
        }
    }

    fn expand(&mut self) {
        for expr in self.defs.file_content.clone() {
            let new_expr = expr.clone();
            self.expand_dict(new_expr);
        }
    }

    pub fn compile_with_path(&mut self, file: String, path: String) -> String {
        let previous_defs = self.defs.clone();

        self.defs = MacroDefs::new(vec![path + file.as_str()]);
        self.defs.compile();
        self.final_source = String::with_capacity(self.defs.file_content.len() + self.defs.defines.len());

        self.expand();

        self.defs = previous_defs;
        return self.final_source.clone();
    }

    pub fn compile(&mut self) -> String {
        self.defs.compile();
        self.final_source = String::with_capacity(self.defs.file_content.len() + self.defs.defines.len());

        self.expand();

        return self.final_source.clone();
    }
}