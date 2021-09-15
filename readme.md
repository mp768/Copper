# Copper

What is copper? Copper is a scripting language made as a learning project by mp768, designed to be simple, intergrated with, and as the main way to interface with your project. 

Please note that this project is being wrapped up, so it will not gain updates as it was for learning purposes. Plus it will not be transformed to a library (because I'm too lazy to do that).

# How to use it?

You can include it in your project (If you make it a library) by doing this:

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

# What will happen to copper?

Well, I'll be using the experience of making copper to help develop a new programming language that goes out of my comfort zone. This new language will be compiled using llvm and have an option to compile to c (for portablilty reasons). The reason? I want to make something that I find interesting and cool, I was never really that invested in the idea of making a scripting language, but it was a stepping point towards how I should structure things. I plan for this new project to get a lot of investment from me, so expect it to be at least higher quality then this one (which isn't hard to do).