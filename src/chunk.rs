use crate::{environment::Environment, value::{Value, ClassType}};

#[derive(Debug, Clone)]
pub enum OpCode {
    Return,
    EndScript,

    Push(Value),
    Pop,
    Add,
    Sub,
    Mul,
    Div,

    CallFunc(String),
    TransformToType(ClassType),

    CmpLess,
    CmpLessEqual,
    CmpGreater,
    CmpGreaterEqual,
    CmpEqual,
    CmpNotEqual,
    CmpAnd,
    CmpOr,

    Jmp(usize),
    JmpIfFalse(usize),

    Negate,
    Not,

    // Used to store arguments from call stack.
    ArgumentStore(String, ClassType),
    Store(String, ClassType),
    InferStore(String),
    Load(String),
    Assign(String),
    NewStruct(String),
    StructGet(String),
    StructSet(String, Vec<String>),
    StructSetByIndex(usize),

    PopToCall,

    StartScope,
    EndScope,
}

#[derive(Clone)]
pub struct Chunk {
    pub code: Vec<OpCode>,
    pub lines: Vec<usize>,
    pub functions: Environment,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk { code: Vec::new(), lines: Vec::new(), functions: Environment::new() }
    }

    pub fn erase(&mut self) {
        self.code = Vec::new();
        self.lines = Vec::new();
        self.functions = Environment::new();
    }

    // Base function that every opcode can implement from.
    pub fn write(&mut self, byte: OpCode, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn write_store(&mut self, name: String, ctype: ClassType, line: usize) {
        self.write(OpCode::Store(name, ctype), line);
    }

    pub fn write_store_infer(&mut self, name: String, line: usize) {
        self.write(OpCode::InferStore(name), line);
    }

    pub fn write_argument_store(&mut self, name: String, ctype: ClassType, line: usize) {
        self.write(OpCode::ArgumentStore(name, ctype), line);
    }

    pub fn write_load(&mut self, name: String, line: usize) {
        self.write(OpCode::Load(name), line);
    }

    pub fn write_constant(&mut self, val: Value, line: usize) {
        self.write(OpCode::Push(val.clone()), line);
    }

    pub fn write_jmp(&mut self, jmp_to_point: usize, line: usize) {
        self.write(OpCode::Jmp(jmp_to_point), line);
    }

    pub fn write_jmp_if_false(&mut self, jmp_to_point: usize, line: usize) {
        self.write(OpCode::JmpIfFalse(jmp_to_point), line);
    }

    pub fn write_call(&mut self, name: String, line: usize) {
        self.write(OpCode::CallFunc(name), line);
    }

    pub fn transform_to_type(&mut self, ctype: ClassType, line: usize) {
        self.write(OpCode::TransformToType(ctype), line);
    } 

    pub fn bind_function(&mut self, name: String,  ctype: ClassType, arg_count: usize, bytecode_position: usize) {
        self.functions.add_function(name, ctype, arg_count, bytecode_position);
    }

    pub fn bind_native_function(&mut self, name: String, arg_count: usize, func: &'static dyn Fn(Vec<Value>) -> Value) {
        self.functions.add_native_function(name, arg_count, func);
    }
}


// Sets up disassembling of instructions in a more understandable manner.
impl Chunk {
    fn match_print(&mut self, op: &OpCode) {
        match op {
            OpCode::Return => print!("return"),
            OpCode::EndScript => print!("end script"),
            OpCode::Push(value) => print!("push '{}'   {}", value.string_s(), value.type_to_string()),
            OpCode::Pop => print!("pop"),
            OpCode::Add => print!("add"),
            OpCode::Sub => print!("sub"),
            OpCode::Mul => print!("mul"),
            OpCode::Div => print!("div"),
            OpCode::CallFunc(func_name) => print!("call function '{}'", func_name),
            OpCode::CmpLess => print!("[<]"),
            OpCode::CmpLessEqual => print!("[<=]"),
            OpCode::CmpGreater => print!("[>]"),
            OpCode::CmpGreaterEqual => print!("[>=]"),
            OpCode::CmpEqual => print!("[==]"),
            OpCode::CmpNotEqual => print!("[!=]"),
            OpCode::CmpAnd => print!("[&&]"),
            OpCode::CmpOr => print!("[||]"),
            OpCode::Jmp(at) => print!("jump at '{}'", at),
            OpCode::JmpIfFalse(at) => print!("jump if false at '{}'", at),
            OpCode::Negate => print!("negate"),
            OpCode::ArgumentStore(name, ctype) => print!("argument store '{}'   {:?}", name, ctype),
            OpCode::Store(name, ctype) => print!("store '{}'   {:?}", name, ctype),
            OpCode::InferStore(x) => print!("infer store '{}'", x),
            OpCode::Load(name) => print!("load '{}'", name),
            OpCode::Assign(name) => print!("assign '{}'", name),
            OpCode::PopToCall => print!("pop from stack to call stack"),
            OpCode::StartScope => print!("start scope"),
            OpCode::Not => print!("not"),
            OpCode::EndScope => print!("end scope"),
            OpCode::TransformToType(c) => print!("transform to {:?}", c),
            OpCode::StructGet(name) => print!("struct get {}", name),
            OpCode::StructSet(name, sets) => print!("struct set {}, {:?}", name, sets),
            OpCode::StructSetByIndex(index) => print!("set {} index of struct", index),
            OpCode::NewStruct(name) => print!("new set of structure {}", name),
            // _ => print!("[Unknown opcode]"),
        }
    }

    fn simple_instruction(&mut self, offset: usize) -> usize {
        self.match_print(&self.code[offset].clone());
        println!();
        
        return offset + 1;
    }

    fn disassemble_instruction(&mut self, offset: usize) -> usize {
        print!("{:04} ", offset);

        if offset > 0 && self.lines[offset] == self.lines[offset-1] {
            print!("   | ");
        } else {
            print!("{:4} ", self.lines[offset]);
        }

        return self.simple_instruction(offset);
    } 

    pub fn disassemble(&mut self) {
        let mut offset: usize = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
    }
}