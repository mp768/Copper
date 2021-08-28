pub mod tokens;
pub mod value;
pub mod chunk;
pub mod vm;
pub mod environment;
pub mod parser;
pub mod codegen;

use codegen::CopperGen;
use parser::CopperParser;
use chunk::{Chunk, OpCode};
use value::{Value, ClassType};
use vm::VM;

fn copper_print(values: Vec<Value>) -> Value {
    let val = values[0].clone();
    val.print();

    return Value::None;
}

fn main() {
    //let mut scanner = Token::lexer("(if) {else} [for] 5 / 8 * 9 - 100000 + -6 9u -93.02 82.1 \"\tHello There fellow neightbor 5456!\n\" \"\" ahello a78");

    let mut chunk = Chunk::new();

    chunk.write_constant(Value::Decimal(9.342), 0);
    chunk.write_store(String::from("Hello"), ClassType::Decimal, 0);
    chunk.write_load(String::from("Hello"), 0);
    //chunk.write(OpCode::Negate, 0);
    chunk.write_constant(Value::Int(5), 1);
    chunk.write(OpCode::StartScope, 1);
    chunk.write_store(String::from("Number"), ClassType::Any,  1);
    chunk.write_load(String::from("Number"), 1);
    chunk.write(OpCode::Add, 1);
    chunk.write(OpCode::EndScope, 1);
    chunk.write_constant(Value::Uint(6), 1);
    chunk.write_constant(Value::Int(5), 1);
    chunk.write(OpCode::CmpGreater, 1);
    chunk.write_jmp_if_false(0, 1);
    chunk.write(OpCode::EndScript, 1);

    //schunk.disassemble();

    let mut gen = CopperGen::new();

    let mut files: Vec<String> = Vec::new();
    files.push(String::from("test.txt"));

    let mut new_chunk = gen.generate_chunk(files);

    new_chunk.bind_native_function("print".to_string(), 1, &copper_print);

    new_chunk.disassemble();

    let mut vm = VM::new(&new_chunk);

    vm.interpret();


    //let mut c = scanner.next();
    //while c != None {
    //    print!("{:?}: ", c.unwrap());
    //    println!("{}", scanner.slice());
//
    //    c = scanner.next();
    //}
}
