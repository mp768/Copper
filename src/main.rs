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

fn copper_println(values: Vec<Value>) -> Value {
    let val = values[0].clone();
    val.println();

    return Value::None;
}

fn main() {
    let mut cmd_args: Vec<String> = std::env::args().collect();
    cmd_args.remove(0);

    if cmd_args.len() == 0 {
        //println!("copper [file names...]");
        //return;
    }


    let mut gen = CopperGen::new();

    let mut new_chunk = gen.generate_chunk(vec!["test.txt".to_string()]);

    new_chunk.bind_native_function("print".to_string(), 1, &copper_print);
    new_chunk.bind_native_function("println".to_string(), 1, &copper_println);

    new_chunk.disassemble();

    let mut vm = VM::new(&new_chunk);

    //let mut parser = CopperParser::new("/".to_string());
    //
    //while let Some(x) = parser.parse() {
    //    println!("{:?}", x);
    //}

    vm.interpret();

    //let mut c = scanner.next();
    //while c != None {
    //    print!("{:?}: ", c.unwrap());
    //    println!("{}", scanner.slice());
//
    //    c = scanner.next();
    //}
}
