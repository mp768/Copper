use core::fmt;
use std::{ops::Deref};

use crate::{tokens::{Lexer, Token}, value::{ClassType, Value}};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum AstExpr {
    Nothing,
    Binary(Box<AstExpr>, Token, Box<AstExpr>),
    Group(Box<AstExpr>),

    Literal(Value),
    Unary(Token, Box<AstExpr>),
    Variable(String),
    Assign(String, Box<AstExpr>),
    AssignByOp(String, Token, Box<AstExpr>),
    Call(String, Vec<AstExpr>),
    Block(Vec<AstStmt>),
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum AstStmt {
    Expr(AstExpr),
    Declaration(String, ClassType, AstExpr),
    InferDeclaration(String, AstExpr),
    If(AstExpr, Box<AstStmt>, Option<Box<AstStmt>>),
    While(AstExpr, AstExpr),
    Function(String, ClassType, Vec<String>, Vec<ClassType>, AstExpr),
    Return(Option<AstExpr>),
}

macro_rules! unwrap_ast {
    ($var:expr) => {
        match $var {
            Some(x) => x,
            None => return None,
        }
    };
}

macro_rules! do_while {
    ($condition:expr => $block:block) => {
        loop {
            $block;
            if !$condition {
                break;
            }
        }
    };
}

macro_rules! consume {
    ($self:expr, $token:expr, $str:expr) => {
        $self.consume($token, $str);
    };
}

pub struct CopperParser {
    pub current_token: Option<Token>,
    pub previous_token: Option<Token>,
    pub source: String,
    pub current_lexer: Lexer,
    pub current_lexeme: String,
    pub current_line: usize,
    pub continue_parsing: bool,
}

impl CopperParser {
    fn peek(&self) -> Option<Token> {
        return self.current_token.clone();
    }

    fn peek_previous(&self) -> Option<Token> {
        return self.previous_token.clone();
    }

    fn advance(&mut self) -> Option<Token> {
        self.previous_token = self.current_token.clone();
        self.current_token = self.current_lexer.next();
        self.current_lexeme = self.current_lexer.slice();
        self.current_line = self.current_lexer.line;

        //println!("Advance: 
        //        - Previous {:?}
        //        - Current {:?}
        //        - Current Lexeme {}", self.previous_token, self.current_token, self.current_lexeme);


        return self.previous_token.clone();
    }

    fn report_error(&mut self, message: &str) {
        print!("[Line {}] Error", self.current_line);

        if self.at_end() {
            print!(" at end");
        } else {
            print!(" at '{}'", self.current_lexeme);
        }

        println!(": '{}'", message);
        self.continue_parsing = false;
    }

    fn at_end(&self) -> bool {
        return self.current_token == None;
    }

    fn check(&self, token: Token) -> bool {
        if self.at_end() {
            return false;
        } else {
            return self.peek().unwrap() == token;
        }
    }

    fn consume(&mut self, token: Token, error_msg: &str) -> Option<Token> {
        if self.check(token) {
            return self.advance();
        }
        
        self.report_error(error_msg);
        return None;
    }

    fn match_tokens(&mut self, tokens: &[Token]) -> bool {
        for i in tokens {
            if self.check(i.clone()) {
                self.advance();
                return true;
            }
        }

        return false;
    }

    fn primary_expr(&mut self) -> Option<AstExpr> {
        let token = unwrap_ast!(self.peek());
        self.advance();

        match token {
            Token::CmpTrue => return Some(AstExpr::Literal(Value::Bool(true))),
            Token::CmpFalse => return Some(AstExpr::Literal(Value::Bool(false))),

            Token::Int(x) => return Some(AstExpr::Literal(Value::Int(x))),
            Token::Uint(x) => return Some(AstExpr::Literal(Value::Uint(x))),
            Token::Decimal(x) => return Some(AstExpr::Literal(Value::Decimal(x))),
            Token::Str(x) => return Some(AstExpr::Literal(Value::Str(x))),

            Token::Identifer(name) => return Some(AstExpr::Variable(name)),

            Token::LeftParen => {
                let expr = unwrap_ast!(self.expression());

                consume!(self, Token::RightParen, "Expected an ')' after expression");

                return Some(AstExpr::Group(Box::new(expr)));
            }

            _ => {
                self.report_error("Expected an expression");
                return None;
            },
        }
    }

    fn finish_call_expr(&mut self, Callee: AstExpr) -> Option<AstExpr> {
        let name = if let AstExpr::Variable(x) = Callee {
            x
        } else {
            self.report_error("Expected a identifer for calling a function.");
            return None;
        };

        let mut arguments: Vec<AstExpr> = Vec::new();

        if !self.match_tokens(&[Token::RightParen]) {
            do_while!(self.match_tokens(&[Token::Comma]) => {
                if arguments.len() > 255 {
                    self.report_error("Can't have more than 255 arguments in a call");
                    return None;
                }

                arguments.push(unwrap_ast!(self.expression()));
            });
        }

        consume!(self, Token::RightParen, "Expected ')' after call arguments");

        return Some(AstExpr::Call(name, arguments));
    }

    fn call_expr(&mut self) -> Option<AstExpr> {
        let mut expr = unwrap_ast!(self.primary_expr());

        loop {
            if self.match_tokens(&[Token::LeftParen]) {
                expr = unwrap_ast!(self.finish_call_expr(expr));
            } else {
                break;
            }
        }

        return Some(expr);
    }

    fn unary_expr(&mut self) -> Option<AstExpr> {
        if self.match_tokens(&[Token::Minus, Token::Not]) {
            let op = unwrap_ast!(self.peek_previous());
            let right = unwrap_ast!(self.unary_expr());

            return Some(AstExpr::Unary(op, Box::new(right)));
        }

        return self.call_expr();
    }

    fn factor_expr(&mut self) -> Option<AstExpr> {
        let mut expr = unwrap_ast!(self.unary_expr());

        while self.match_tokens(&[Token::Star, Token::Slash]) {
            let op = unwrap_ast!(self.peek_previous());
            let right = unwrap_ast!(self.unary_expr());

            expr = AstExpr::Binary(Box::new(expr), op, Box::new(right));
        }

        return Some(expr);
    }

    fn term_expr(&mut self) -> Option<AstExpr> {
        let mut expr = unwrap_ast!(self.factor_expr());

        while self.match_tokens(&[Token::Minus, Token::Plus]) {
            let op = unwrap_ast!(self.peek_previous());
            let right = unwrap_ast!(self.factor_expr());

            expr = AstExpr::Binary(Box::new(expr), op, Box::new(right));
        }

        return Some(expr);
    }

    fn comparsion_expr(&mut self) -> Option<AstExpr> {
        let mut expr = unwrap_ast!(self.term_expr());
        
        while self.match_tokens(&[Token::Less, Token::LessEqual, Token::Greater, Token::GreaterEqual]) {
            let op = unwrap_ast!(self.peek_previous());
            let right = unwrap_ast!(self.term_expr());

            expr = AstExpr::Binary(Box::new(expr), op, Box::new(right));
        }

        return Some(expr);
    }

    fn equality_expr(&mut self) -> Option<AstExpr> {
        let mut expr = unwrap_ast!(self.comparsion_expr());

        while self.match_tokens(&[Token::EqualEqual, Token::NotEqual]) {
            let op = unwrap_ast!(self.peek_previous());
            let right = unwrap_ast!(self.comparsion_expr());

            expr = AstExpr::Binary(Box::new(expr), op, Box::new(right));
        }

        return Some(expr);
    }

    fn and_expr(&mut self) -> Option<AstExpr> {
        let mut expr = unwrap_ast!(self.equality_expr());

        while self.match_tokens(&[Token::CmpAnd]) {
            let right = unwrap_ast!(self.equality_expr());

            expr = AstExpr::Binary(Box::new(expr), Token::CmpAnd, Box::new(right));
        }

        return Some(expr);
    }

    fn or_expr(&mut self) -> Option<AstExpr> {
        let mut expr = unwrap_ast!(self.and_expr());

        while self.match_tokens(&[Token::CmpOr]) {
            let right = unwrap_ast!(self.and_expr());

            expr = AstExpr::Binary(Box::new(expr), Token::CmpOr, Box::new(right));
        }

        return Some(expr);
    }

    fn assignment_by_op_expr(&mut self) -> Option<AstExpr> {
        let expr = unwrap_ast!(self.or_expr());

        if self.match_tokens(&[Token::PlusEqual, Token::MinusEqual, Token::StarEqual, Token::SlashEqual]) {
            let op = unwrap_ast!(self.peek_previous());
            let value = unwrap_ast!(self.assignment_expr());

            if let AstExpr::Variable(name) = expr {
                return Some(AstExpr::AssignByOp(name, op, Box::new(value)));
            }

            self.report_error("Invalid assignment");
        }

        return Some(expr);
    }

    fn assignment_expr(&mut self) -> Option<AstExpr> {
        let expr = unwrap_ast!(self.assignment_by_op_expr());

        if self.match_tokens(&[Token::Equal]) {
            let value = unwrap_ast!(self.assignment_expr());

            if let AstExpr::Variable(name) = expr {
                return Some(AstExpr::Assign(name, Box::new(value)));
            }

            self.report_error("Invalid assignment");
        }

        return Some(expr);
    }

    fn expression(&mut self) -> Option<AstExpr> {
        let expr = self.assignment_expr();

        //match &expr {
        //    Some(_) => {},
        //    None => self.report_error("Couldn't parse further"),
        //} 

        return expr;
    }

    fn var_declaration_stmt(&mut self) -> Option<AstStmt> {
        let name = self.current_lexeme.clone();
        consume!(self, Token::Identifer(self.current_lexeme.clone()), "Expected variable name");

        let mut expr: AstExpr = AstExpr::Literal(Value::None);
        let mut ttype = ClassType::Any;
        let mut got_a_type = false;

        if self.match_tokens(&[Token::Colon]) {
            ttype = match unwrap_ast!(self.current_token.clone()) {
                Token::TypeBool => ClassType::Bool,
                Token::TypeAny => ClassType::Any,
                Token::TypeDecimal => ClassType::Decimal,
                Token::TypeUint => ClassType::Uint,
                Token::TypeInt => ClassType::Int,
                Token::TypeString => ClassType::Str,
                _ => {
                    self.report_error("Expected a type identifer");
                    return None;
                }
            };
            self.advance();

            got_a_type = true;
        }

        if self.match_tokens(&[Token::Equal]) {
            expr = unwrap_ast!(self.expression());
        }

        consume!(self, Token::Semicolon, "Expected ';' after variable declaration");

        if got_a_type {
            return Some(AstStmt::Declaration(name, ttype, expr));
        } else {
            return Some(AstStmt::InferDeclaration(name, expr));
        }
    }

    fn block(&mut self) -> Option<AstExpr> {
        let mut stmts: Vec<AstStmt> = Vec::new();

        while !self.check(Token::RightBrace) && !self.at_end() {
            stmts.push(unwrap_ast!(self.declaration_stmt()));
        }

        consume!(self, Token::RightBrace, "Expected '}' after block");
        return Some(AstExpr::Block(stmts));
    }

    fn function_stmt(&mut self, ftype: &str) -> Option<AstStmt> {
        let name = self.current_lexeme.clone();
        consume!(self, Token::Identifer(self.current_lexeme.clone()), format!("Expected {} name", ftype).deref());

        consume!(self, Token::LeftParen, format!("Expected '(' after {} name", ftype).deref());

        let mut identifers: Vec<String> = Vec::new();
        let mut ctypes: Vec<ClassType> = Vec::new();

        if !self.check(Token::RightParen) {
            do_while!(self.match_tokens(&[Token::Comma]) => {
                if identifers.len() > 255 {
                    self.report_error("Cannot have more than 255 parameters");
                    return None;
                }

                identifers.push(self.current_lexeme.clone());
                consume!(self, Token::Identifer(self.current_lexeme.clone()), "Expected an identifer for parameter");
                consume!(self, Token::Colon, "Expected ':' after parameter identifer");

                ctypes.push(match unwrap_ast!(self.advance()) {
                    Token::TypeBool => ClassType::Bool,
                    Token::TypeAny => ClassType::Any,
                    Token::TypeDecimal => ClassType::Decimal,
                    Token::TypeUint => ClassType::Uint,
                    Token::TypeInt => ClassType::Int,
                    Token::TypeString => ClassType::Str,
                    _ => {
                        self.report_error("Expected a type identifer");
                        return None;
                    }
                });
            });
        }

        consume!(self, Token::RightParen, "Expected ')' after parameters");

        let mut ctype = ClassType::Any;

        if self.match_tokens(&[Token::Colon]) {
            //consume!(self, Token::Colon, "Expected ':' before function return type");

            ctype = match unwrap_ast!(self.advance()) {
                Token::TypeBool => ClassType::Bool,
                Token::TypeAny => ClassType::Any,
                Token::TypeDecimal => ClassType::Decimal,
                Token::TypeUint => ClassType::Uint,
                Token::TypeInt => ClassType::Int,
                Token::TypeString => ClassType::Str,
                _ => {
                    self.report_error("Expected a type identifer");
                    return None;
                }
            };
        }

        consume!(self, Token::LeftBrace, format!("Expected '{{' before {} body", ftype).deref());
        let block = unwrap_ast!(self.block());

        return Some(AstStmt::Function(name, ctype, identifers, ctypes, block));
    }

    fn while_stmt(&mut self) -> Option<AstStmt> {
        let condition = unwrap_ast!(self.expression());

        consume!(self, Token::LeftBrace, "Expected '{' before 'while' body");
        let body = unwrap_ast!(self.block());

        return Some(AstStmt::While(condition, body));
    }

    fn for_stmt(&mut self) -> Option<AstStmt> {
        let initializer = match self.var_declaration_stmt() {
            Some(x) => x,
            None => AstStmt::Expr(AstExpr::Nothing),
        };

        if initializer == AstStmt::Expr(AstExpr::Nothing) {
            consume!(self, Token::Semicolon, "Expected ';' after initializer");
        }

        let condition = match self.expression() {
            Some(x) => x,
            None => AstExpr::Nothing,
        };

        consume!(self, Token::Semicolon, "Expected ';' after condition");

        let increment = match self.expression() {
            Some(x) => x,
            None => AstExpr::Nothing,
        };

        consume!(self, Token::LeftBrace, "Expected '{' before 'for' body");
        let body = unwrap_ast!(self.block());

        // This is for wrapping a while into looking like a c style for loop.
        let mut for_body: Vec<AstStmt> = Vec::new();

        for_body.push(initializer);

        // This if for use of just getting the 'body' and adding the increment expr.
        let mut while_body: Vec<AstStmt> = Vec::new();

        if let AstExpr::Block(stmts) = body {
            while_body = stmts;
        }
        while_body.push(AstStmt::Expr(increment));

        let body = AstExpr::Block(while_body);

        for_body.push(AstStmt::While(condition, body));

        return Some(AstStmt::Expr(AstExpr::Block(for_body)));
    }

    fn if_stmt(&mut self) -> Option<AstStmt> {
        let condition = unwrap_ast!(self.expression());

        consume!(self, Token::LeftBrace, "Expected '{' after 'if' condition");
        let if_body = Box::new(AstStmt::Expr(unwrap_ast!(self.block())));
        let mut else_body: Option<Box<AstStmt>> = None;

        if self.match_tokens(&[Token::Else]) {
            if self.match_tokens(&[Token::If]) {
                return Some(AstStmt::If(condition, if_body, Some(Box::new(unwrap_ast!(self.if_stmt())))))
            }
            consume!(self, Token::LeftBrace, "Expected '{' after 'else'");
            else_body = Some(Box::new(AstStmt::Expr(unwrap_ast!(self.block()))));
        }

        return Some(AstStmt::If(condition, if_body, else_body));
    }

    fn return_stmt(&mut self) -> Option<AstStmt> {
        if self.match_tokens(&[Token::Semicolon]) {
            return Some(AstStmt::Return(None));
        }

        let expr = unwrap_ast!(self.or_expr());

        consume!(self, Token::Semicolon, "Expected ';' after return statement");

        return Some(AstStmt::Return(Some(expr)));
    }

    fn expr_stmt(&mut self) -> Option<AstStmt> {
        let expr = unwrap_ast!(self.expression());
        consume!(self, Token::Semicolon, "Expected ';' after expression");

        return Some(AstStmt::Expr(expr));
    }

    fn stmt(&mut self) -> Option<AstStmt> {
        if self.match_tokens(&[Token::Func]) {
            return self.function_stmt("function");
        }

        if self.match_tokens(&[Token::While]) {
            return self.while_stmt();
        }

        if self.match_tokens(&[Token::For]) {
            return self.for_stmt();
        }

        if self.match_tokens(&[Token::If]) {
            return self.if_stmt();
        }

        if self.match_tokens(&[Token::Return]) {
            return self.return_stmt();
        }

        return self.expr_stmt();
    }

    fn declaration_stmt(&mut self) -> Option<AstStmt> {
        if self.match_tokens(&[Token::Var]) {
            return self.var_declaration_stmt();
        }

        if self.match_tokens(&[Token::LeftBrace]) {
            return Some(AstStmt::Expr(unwrap_ast!(self.block())));
        }

        return self.stmt();
    }

    pub fn parse(&mut self) -> Option<AstStmt> {
        match self.current_token {
            Some(_) => {},
            None => {
                self.advance();
            },
        };

        unwrap_ast!(self.current_token.clone());

        return self.declaration_stmt();
    }

    pub fn new(source: String) -> Self {
        // TODO: 
        // Objective: Create a new lexer implementation to replace 'logos' implementation.
        // Reason: I can't have strings because of lifetimes (which is stupid, since the reason logos is a thing is to lex files).
        // Requirements: the lexer should provide enough of the same functionality so the need to rewrite the parser is zero (because I spent days on it already).
        //
        // P.S. don't forget to remove any reference of the logos crate.

        return Self {
            current_token: None,
            previous_token: None,
            source: source.clone(),
            current_lexer: Lexer::new(source),
            current_lexeme: String::new(),
            current_line: 0,
            continue_parsing: true,
        };
    }
}


impl fmt::Display for AstExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.clone() {
            AstExpr::Nothing => write!(f, "nothing"),
            AstExpr::Binary(a, op, b) => write!(f, "{} {} {}", a, op, b),
            AstExpr::Group(group) => write!(f, "({})", group),
            AstExpr::Literal(literal) => write!(f, "{:?}", literal),
            AstExpr::Unary(op, b) => write!(f, "{:?} => {}", op, b),
            AstExpr::Variable(name) => write!(f, "{}", name),
            AstExpr::Assign(name, value) => write!(f, "{} = {}", name, value),
            AstExpr::AssignByOp(name, op, value) => write!(f, "{} {} {}", name, op, value),
            AstExpr::Call(name, arguments) => {
                write!(f, "{}(", name)?;

                for i in 0..arguments.len() {
                    write!(f, "{}", arguments[i])?;

                    if i != arguments.len()-1 {
                        write!(f, ", ")?;
                    }
                }

                write!(f, ")")
            },
            AstExpr::Block(stmts) => {
                write!(f, "{{\n")?;

                for i in &stmts {
                    write!(f, "{}", i)?;
                }

                write!(f, "}}\n")
            },
        }
    }
}

impl fmt::Display for AstStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.clone() {
            AstStmt::Expr(expr) => write!(f, "{}\n", expr),
            AstStmt::Declaration(name, ctype, value) => write!(f, "var {}: {:?} = {}\n", name, ctype, value),
            AstStmt::InferDeclaration(name, value) => write!(f, "var {} = {}\n", name, value),
            AstStmt::If(condition, then, next) => if next == None {
                write!(f, "if {} {{\n {} \n }}", condition, then)
            } else {
                write!(f, "if {} {{\n {} \n }}\n else {{\n {} \n}}\n", condition, then, next.unwrap())
            },
            AstStmt::While(condition, body) => write!(f, "while {} {{\n {} \n}}\n", condition, body),
            AstStmt::Function(name, ctype, identifers, ctypes, body) => {                
                write!(f, "function {}(", name)?;

                for i in 0..identifers.len() {
                    write!(f, "{}: {:?}", identifers[i], ctypes[i])?;

                    if i != identifers.len() - 1 {
                        write!(f, ", ")?;
                    }
                }

                write!(f, "): {:?} {}\n", ctype, body)
            },
            AstStmt::Return(value) => if value == None {
                write!(f, "return\n")
            } else {
                write!(f, "return {}\n", value.unwrap())
            },
        }
    }
}