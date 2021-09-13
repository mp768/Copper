pub mod mini_macros;
pub mod tokens;
pub mod value;
pub mod chunk;
pub mod vm;
pub mod environment;
pub mod parser;
pub mod codegen;

use std::io::{Write, stdin, stdout};

use codegen::CopperGen;
use parser::CopperParser;
use value::{Value};
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

fn copper_input(values: Vec<Value>) -> Value {
    let val = values[0].clone();
    val.print();
    let _ = stdout().flush();

    let mut input = String::new();

    match stdin().read_line(&mut input) {
        Ok(_) => {},
        Err(err) => panic!("{}", err),
    }

    input = input.trim().to_string();

    return Value::Str(input);
}

fn copper_inputln(values: Vec<Value>) -> Value {
    let val = values[0].clone();
    val.println();
    let _ = stdout().flush();

    let mut input = String::new();

    match stdin().read_line(&mut input) {
        Ok(_) => {},
        Err(err) => panic!("{}", err),
    }

    input = input.trim().to_string();

    return Value::Str(input);
}

fn copper_abs(values: Vec<Value>) -> Value {
    let val = values[0].clone();

    match val {
        Value::None => return Value::Int(0),
        Value::Uint(x) => return Value::Uint(x),
        Value::Int(x) => return Value::Int(x.abs()),
        Value::Decimal(x) => return Value::Decimal(x.abs()),
        Value::Str(x) => return Value::Str(x),
        Value::Bool(_) => return Value::Bool(true),
        Value::Struct(_) => return Value::None,
    }
}

pub fn copper_type_to_string(values: Vec<Value>) -> Value {
    let val = values[0].clone();

    return Value::Str(val.type_to_string());
}


fn main() {
    let mut cmd_args: Vec<String> = std::env::args().collect();
    cmd_args.remove(0);

    if cmd_args.len() == 0 {
        //println!("copper [file names...]");
        //return;
        cmd_args.push(String::from("test.txt"));
    }


    let mut gen = CopperGen::new();

    gen.add_content("
        var CONST_INT = 5;
    ");

    let mut new_chunk = gen.generate_chunk(cmd_args);

    new_chunk.bind_native_function("print".to_string(), 1, &copper_print);
    new_chunk.bind_native_function("println".to_string(), 1, &copper_println);
    new_chunk.bind_native_function("input".to_string(), 1, &copper_input);
    new_chunk.bind_native_function("inputln".to_string(), 1, &copper_inputln);
    new_chunk.bind_native_function("abs".to_string(), 1, &copper_abs);
    new_chunk.bind_native_function("type_str".to_string(), 1, &copper_type_to_string);

    //new_chunk.disassemble();

    let mut vm = VM::new(&new_chunk);

    vm.interpret();
}
