# Copper

What is copper? Copper is a scripting language made as learning project by mp768, designed to be simple and intergrated with as the main way to interface with your project. 

Please note that this project is being wrapped up, so it will not gain updates as it was meant for only learning purposes. Plus it will not be transformed to a library (because I'm too lazy to do that).

# How to use it?

You can include it in your project (If you mske it a library) by doing this:

```Rust
use codegen::CopperGen;
// for manipulating vm value
use value::Value;
use vm::VM;

// This function will and can be called in the language.
fn copper_print(value: Vec<Value>) -> Value {
    let val = value[0].clone();
    val.print();

    return Value::None;
}

fn main() {
    let mut gen = CopperGen::new();
    let mut chunk = gen.generate_chunk(vec!["your_script_here.txt"]);

    // #1 = the function name in copper
    // #2 = The amount of arguments for the function to take
    // #3 = the actually native function
    chunk.bind_native_function("print".to_string(), 1, &copper_print);

    let mut vm = VM::new(&chunk);

    // Starts the process of the virtual machine
    vm.interpret();
}
```


# Example Script

```
// The native function in action
print("Hello, World!");

for i: int = 0; i < 5; i += 1 {
    print("I: " + i);
    // or
    // print("I: " + string(i));
}
```