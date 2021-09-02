use std::ops::Deref;

use crate::parser::{AstStmt, AstExpr};
use crate::tokens::Token;
use crate::value::{ClassType, Value};
use crate::{CopperParser, chunk::{Chunk, OpCode}};

pub struct CopperGen {
    pub parser: CopperParser,
    pub current_line: usize,
    pub chunk: Chunk,
    block_increment: usize,
}

impl CopperGen {
    fn generate_expr(&mut self, expr: AstExpr) {
        match expr {
            AstExpr::Nothing => {},
            AstExpr::TypeCall(ctype, expr) => {
                self.generate_expr(*expr);

                let ctype = match ctype {
                    Token::TypeAny => ClassType::Any,
                    Token::TypeInt => ClassType::Int,
                    Token::TypeUint => ClassType::Uint,
                    Token::TypeDecimal => ClassType::Decimal,
                    Token::TypeString => ClassType::Str,
                    Token::TypeBool => ClassType::Bool,
                    _ => ClassType::Any,
                };

                self.chunk.transform_to_type(ctype, self.current_line);
            }
            AstExpr::Binary(a, op, b) => {
                self.generate_expr(*a);
                self.generate_expr(*b);
                self.generate_op(op);
            },
            AstExpr::Ternary(condition, true_expr, false_expr) => {
                self.generate_expr(*condition);

                let jmp_over = self.generate_patch_jmp();

                self.generate_expr(*true_expr);

                let jmp_over_false = self.generate_patch_jmp();

                self.patch_if_false_jmp(self.chunk.code.len(), jmp_over);

                self.generate_expr(*false_expr);

                self.patch_jmp(self.chunk.code.len(), jmp_over_false);
            }
            AstExpr::Group(expr) => self.generate_expr(*expr),
            AstExpr::Literal(val) => self.chunk.write_constant(val, self.current_line),
            AstExpr::Unary(op, expr) => {
                self.generate_expr(*expr);
                match op {
                    Token::Minus => self.chunk.write(OpCode::Negate, self.current_line),
                    Token::Not => self.chunk.write(OpCode::Not, self.current_line),
                    _ => self.chunk.write(OpCode::Pop, self.current_line),
                }
            },
            AstExpr::Variable(name) => self.chunk.write(OpCode::Load(name), self.current_line),
            AstExpr::Assign(name, expr) => {
                if let AstExpr::Block(_) = *expr {
                    self.generate_block_function(*expr);
                } else {
                    self.generate_expr(*expr);
                }
                self.chunk.write(OpCode::Assign(name), self.current_line);
            },
            AstExpr::AssignByOp(name, op, expr) => {
                self.chunk.write_load(name.clone(), self.current_line);

                if let AstExpr::Block(_) = *expr {
                    self.generate_block_function(*expr);
                } else {
                    self.generate_expr(*expr);
                }

                match op {
                    Token::PlusEqual => self.chunk.write(OpCode::Add, self.current_line), 
                    Token::MinusEqual => self.chunk.write(OpCode::Sub, self.current_line), 
                    Token::StarEqual => self.chunk.write(OpCode::Mul, self.current_line), 
                    Token::SlashEqual => self.chunk.write(OpCode::Div, self.current_line), 
                    _ => panic!("Expected an operator for assigning"),
                }
                self.chunk.write(OpCode::Assign(name), self.current_line);
            }
            AstExpr::Call(name, arguments) => self.generate_call_expr(name, arguments),
            AstExpr::Block(stmts) => {
                self.chunk.write(OpCode::StartScope, self.current_line);
                for s in stmts {
                    self.generate_stmt(s);
                }
                self.chunk.write(OpCode::EndScope, self.current_line);
            },
        }
    }

    fn generate_op(&mut self, op: Token) {
        match op {
            Token::CmpAnd => self.chunk.write(OpCode::CmpAnd, self.current_line),
            Token::CmpOr => self.chunk.write(OpCode::CmpOr, self.current_line),
            Token::Plus => self.chunk.write(OpCode::Add, self.current_line),
            Token::Minus => self.chunk.write(OpCode::Sub, self.current_line),
            Token::Star => self.chunk.write(OpCode::Mul, self.current_line),
            Token::Slash => self.chunk.write(OpCode::Div, self.current_line),
            Token::Not => self.chunk.write(OpCode::Not, self.current_line),
            Token::EqualEqual => self.chunk.write(OpCode::CmpEqual, self.current_line),
            Token::NotEqual => self.chunk.write(OpCode::CmpNotEqual, self.current_line),
            Token::Less => self.chunk.write(OpCode::CmpLess, self.current_line),
            Token::LessEqual => self.chunk.write(OpCode::CmpLessEqual, self.current_line),
            Token::Greater => self.chunk.write(OpCode::CmpGreater, self.current_line),
            Token::GreaterEqual => self.chunk.write(OpCode::CmpGreaterEqual, self.current_line),
            _ => {},
        }
    }

    fn generate_block_function(&mut self, block: AstExpr) {
        let jmp_over = self.generate_patch_jmp();
        self.chunk.bind_function(format!("@block_func:{}", self.block_increment), ClassType::Any, 0, self.chunk.code.len());
        
        self.chunk.write(OpCode::StartScope, self.current_line);

        self.generate_expr(block);

        self.chunk.write_constant(Value::None, self.current_line);
        self.chunk.write(OpCode::Return, self.current_line);
        
        self.chunk.write(OpCode::EndScope, self.current_line);

        self.patch_jmp(self.chunk.code.len(), jmp_over);
        self.generate_call_expr(format!("@block_func:{}", self.block_increment), vec![]);
        self.block_increment += 1;
    }

    fn generate_call_expr(&mut self, name: String, arguments: Vec<AstExpr>) {
        for i in arguments.clone() {
            if let AstExpr::Block(_) = i {
                self.generate_block_function(i);
            } else {
                self.generate_expr(i);
            }
        }

        for _ in arguments {
            self.chunk.write(OpCode::PopToCall, self.current_line);
        }

        self.chunk.write_call(name, self.current_line);
    }

    fn blacklist_expr(&mut self, expr: AstExpr) {
        match expr {
            AstExpr::Unary(_, _) => {},
            AstExpr::Literal(_) => {},
            AstExpr::Binary(a, _, b) => {
                self.blacklist_expr(*a);
                self.blacklist_expr(*b);
            },
            AstExpr::Group(expr) => self.generate_expr(*expr),
            AstExpr::Call(name, arguments) => {
                self.generate_call_expr(name, arguments);
                self.chunk.write(OpCode::Pop, self.current_line);
            }
            AstExpr::Ternary(condition, true_expr, false_expr)=> {
                self.generate_expr(*condition);
                
                let jmp_over = self.generate_patch_jmp();

                self.blacklist_expr(*true_expr);

                let jmp_over_false = self.generate_patch_jmp();

                self.patch_if_false_jmp(self.chunk.code.len(), jmp_over);

                self.blacklist_expr(*false_expr);

                self.patch_jmp(self.chunk.code.len(), jmp_over_false);
            }
            _ => self.generate_expr(expr),
        }
    }

    fn generate_patch_jmp(&mut self) -> usize {
        self.chunk.write_jmp(0, self.current_line);
        return self.chunk.code.len()-1;
    }

    fn patch_if_false_jmp(&mut self, new_position: usize, patch_num: usize) {
        self.chunk.code[patch_num] = OpCode::JmpIfFalse(new_position);
    }

    fn patch_jmp(&mut self, new_position: usize, patch_num: usize) {
        self.chunk.code[patch_num] = OpCode::Jmp(new_position);
    }

    fn generate_stmt(&mut self, stmt: AstStmt) {
        match stmt {
            AstStmt::Expr(expr) => self.blacklist_expr(expr),
            AstStmt::Declaration(name, ctype, expr) => {
                if let AstExpr::Block(_) = expr {
                    self.generate_block_function(expr);
                } else {
                    self.generate_expr(expr);
                }
                
                self.chunk.write_store(name, ctype, self.current_line);
            },
            AstStmt::InferDeclaration(name, expr) => {
                if let AstExpr::Block(_) = expr {
                    self.generate_block_function(expr);
                } else {
                    self.generate_expr(expr);
                }
                self.chunk.write_store_infer(name, self.current_line);
            },
            AstStmt::If(condition, then_branch, else_branch) => {
                self.generate_expr(condition);
                let then = self.generate_patch_jmp();
                self.generate_stmt(*then_branch);
                let else_jmp = self.generate_patch_jmp();
                self.patch_if_false_jmp(self.chunk.code.len(), then);

                if let Some(stmt) = else_branch {
                    self.generate_stmt(*stmt);
                }

                self.patch_jmp(self.chunk.code.len(), else_jmp);
            },
            AstStmt::While(condition, body) => {
                let beginning = self.chunk.code.len();
                self.generate_expr(condition);
                let while_loop = self.generate_patch_jmp();

                self.generate_expr(body);

                self.chunk.write_jmp(beginning, self.current_line);
                self.patch_if_false_jmp(self.chunk.code.len(), while_loop);
            },
            AstStmt::Function(name, ctype, arg_names, arg_types, body) => {
                let jmp_over = self.generate_patch_jmp();
                let bytecode_pos = self.chunk.code.len();

                self.chunk.write(OpCode::StartScope, self.current_line);

                for i in 0..arg_names.len() {
                    self.chunk.write_argument_store(arg_names[i].clone(), arg_types[i], self.current_line);
                }

                self.generate_expr(body);

                self.chunk.write_constant(Value::None, self.current_line);
                self.chunk.write(OpCode::Return, self.current_line);

                self.chunk.write(OpCode::EndScope, self.current_line);

                self.chunk.bind_function(name, ctype, arg_names.len(), bytecode_pos);

                self.patch_jmp(self.chunk.code.len(), jmp_over);
            },  
            AstStmt::Return(return_val) => {
                if let Some(expr) = return_val {
                    self.generate_expr(expr);
                } else {
                    self.chunk.write_constant(Value::None, self.current_line);
                }

                self.chunk.write(OpCode::Return, self.current_line);
            },
        }
    }

    fn generate_loop(&mut self) {
        while let Some(stmt) = self.parser.parse() {
            self.current_line = self.parser.current_line;
            self.generate_stmt(stmt);
        }
    }

    pub fn generate_chunk(&mut self, files: Vec<String>) -> Chunk {
        for i in files {
            let source = std::fs::read_to_string(i).expect("Couldn't read the file");

            self.parser = CopperParser::new(source);
            self.generate_loop();
        }

        self.chunk.write(OpCode::EndScript, self.current_line);

        let final_chunk = self.chunk.clone();

        self.chunk.erase();

        return final_chunk;
    }
    
    pub fn new() -> Self {
        Self {
            parser: CopperParser::new("".to_string()),
            current_line: 0,
            chunk: Chunk::new(),
            block_increment: 0,
        }
    }
}