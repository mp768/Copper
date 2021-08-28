use crate::environment::{Environment, EnvEntry};
use crate::value::{ClassType, Value};
use crate::chunk::{Chunk, OpCode};

pub struct VM {
    pub chunk: *const Chunk,
    pub idx: usize,
    pub stack: Vec<Value>,
    pub call_stack: Vec<Value>,

    pub environment: Environment,

    pub function_starting_scope: Vec<usize>,
    pub function_jump_back: Vec<usize>, 
    pub function_return_types: Vec<ClassType>,
}

macro_rules! binary_compare {
    ($self:expr, $op:tt, $type:literal) => {
        binary_compare!(basic $self, $op, { panic!("Cannot compare for '{}' with 'string' type.", $type) }, { panic!("Cannot compare for '{}' with 'bool' type.", $type) }, $type)
    };

    (basic $self:expr, $op:tt, $string_block:expr, $bool_block:expr, $type:literal) => { 
        let (a, b, value_type) = $self.binary_op_vals();
        
        match value_type {
            Value::Uint(_) => $self.stack.push(Value::Bool(a.uint_s() $op b.uint_s())),
            Value::Int(_) => $self.stack.push(Value::Bool(a.int_s() $op b.int_s())),
            Value::Decimal(_) => $self.stack.push(Value::Bool(a.decimal_s() $op b.decimal_s())),
            Value::Str(_) => $string_block,
            Value::Bool(_) => $bool_block,
            Value::None => panic!("Cannot compare for '{}' with 'none' type.", $type),
        }
    };

    (string, bool; $self:expr, $op:tt, $str:literal, $boolean:literal, $type:literal) => {
        let (a, b, value_type) = $self.binary_op_vals();
        
        match value_type {
            Value::Uint(_) => $self.stack.push(Value::Bool(a.uint_s() $op b.uint_s())),
            Value::Int(_) => $self.stack.push(Value::Bool(a.int_s() $op b.int_s())),
            Value::Decimal(_) => $self.stack.push(Value::Bool(a.decimal_s() $op b.decimal_s())),
            Value::Str(_) => {
                if a == b && $str {
                    $self.stack.push(Value::Bool(a.string_s() $op b.string_s()));
                } else {
                    panic!("Cannot compare for '{}' with 'string' type and another type that isn't a 'string'.", $type)
                }
            },
            Value::Bool(_) => {
                if a == b && $boolean {
                    $self.stack.push(Value::Bool(a.bool_s() $op b.bool_s()));
                } else {
                    panic!("Cannot compare for '{}' with 'bool' type and another type that isn't a 'bool'.", $type)
                }
            },
            Value::None => panic!("Cannot compare for '{}' with 'none' type.", $type),
        }
    };
}

impl VM {
    pub fn new(chunk: *const Chunk) -> VM {
        return VM { chunk, idx: 0, environment: Environment::new(), stack: Vec::new(), call_stack: Vec::new(), function_starting_scope: Vec::new(), function_jump_back: Vec::new(), function_return_types: Vec::new() }
    }

    fn read_op(&mut self) -> OpCode {
        let result = unsafe { &(*self.chunk).code[self.idx] };
        self.idx += 1;
        return result.clone();
    }

    fn stack_pop(&mut self) -> Value {
        return match self.stack.pop() {
            Some(v) => v,
            None => panic!("Expected a value when popping from the stack."),
        }
    }

    fn binary_op_vals(&mut self) -> (Value, Value, Value) {
        let b = self.stack_pop();
        let a = self.stack_pop();

        let mut value_type: Value = Value::Uint(0);

        if value_type < a {
            value_type = a.clone();
        }

        if value_type < b {
            value_type = b.clone();
        }

        return (a, b, value_type);
    }

    pub fn interpret(&mut self) {
        let debugging = false;
        let debug_memory = false;

        self.call_stack.push(Value::Int(0));
        self.call_stack.push(Value::Int(0));
        self.call_stack.push(Value::Int(0));
        self.call_stack.clear();
        
        loop {
            if debug_memory && self.stack.len() % 1000 == 0 {
                println!("Current amount of values on stack: {}", self.stack.len());
            }

            if debugging {
                println!("====");
                println!("| Current Scope '{}'", self.environment.current_scope);
                print!("| Stack [ ");

                for i in &self.stack {
                    i.print();
                    print!(", ");
                }

                println!("]");

                print!("| Call Stack [ ");

                for i in &self.call_stack {
                    i.print();
                    print!(", ");
                }

                println!("]");

                println!("| Variables: ");

                for i in &self.environment.entries {
                    print!("| >{}", i);
                }
                println!("====\n\n");
            }

            match self.read_op() {
                OpCode::Return => {
                    if self.function_jump_back.len() != 0 {
                        self.idx = self.function_jump_back.pop().unwrap();
                        self.environment.remove_from_scope(self.function_starting_scope.pop().unwrap());
                        
                        let ctype = self.function_return_types.pop().unwrap();
                        
                        match ctype {
                            ClassType::Any => {},
                            ClassType::Uint => {
                                let value = self.stack_pop();
                                self.stack.push(Value::Uint(value.uint_s()));
                            },
                            ClassType::Int => {
                                let value = self.stack_pop();
                                self.stack.push(Value::Int(value.int_s()));
                            },
                            ClassType::Decimal => {
                                let value = self.stack_pop();
                                self.stack.push(Value::Decimal(value.decimal_s()));
                            },
                            ClassType::Str => {
                                let value = self.stack_pop();
                                self.stack.push(Value::Str(value.string_s()));
                            },
                            ClassType::Bool => {
                                let value = self.stack_pop();
                                self.stack.push(Value::Bool(value.bool_s()));
                            },
                        }
                    } else {
                        panic!("Cannot return out of a function.");
                    }
                },
                OpCode::EndScript => {
                    self.environment.entries.clear();
                    return;
                },
                OpCode::Push(value) => { 
                    self.stack.push(value.clone());
                },
                OpCode::Pop => match self.stack.pop() { _ => {} },
                OpCode::Add => { 
                    let (a, b, _) = self.binary_op_vals();
                    self.stack.push(a.add_s(&b));  
                },
                OpCode::Sub => { 
                    let (a, b, _) = self.binary_op_vals();
                    self.stack.push(a.sub_s(&b));  
                },
                OpCode::Mul => { 
                    let (a, b, _) = self.binary_op_vals();
                    self.stack.push(a.mul_s(&b));  
                },
                OpCode::Div => { 
                    let (a, b, _) = self.binary_op_vals();
                    self.stack.push(a.div_s(&b));  
                },
                OpCode::CallFunc(func_name) => {
                    let func = unsafe { (*self.chunk).functions.get_function(func_name.clone()) };

                    match func {
                        EnvEntry::Function(_, ctype, count, bytecode_pos) => {
                            if self.call_stack.len() != count {
                                for i in &self.call_stack {
                                    i.println();
                                }
                                panic!("Expected {} argument(s), but got {}", count, self.call_stack.len());
                            }

                            self.function_jump_back.push(self.idx);
                            self.function_starting_scope.push(self.environment.current_scope + 1);
                            self.function_return_types.push(ctype);

                            self.idx = bytecode_pos;
                        },
                        EnvEntry::NativeFunction(_, count, func) => {
                            if self.call_stack.len() != count {
                                for i in &self.call_stack {
                                    i.println();
                                }
                                panic!("Expected {} argument(s), but got {}", count, self.call_stack.len());
                            }
                            self.call_stack.reverse();
                            
                            let return_value = func(self.call_stack.clone());
                            self.call_stack.clear();
                            self.stack.push(return_value);
                        },
                        _ => panic!("Expected to get a function named '{}'!", func_name),
                    }
                },
                OpCode::CmpLess => { 
                    binary_compare!(self, <, "less");
                },
                OpCode::CmpLessEqual => { 
                    binary_compare!(self, <=, "less or equal");
                },
                OpCode::CmpGreater => { 
                    binary_compare!(self, >, "greater");
                },
                OpCode::CmpGreaterEqual => {
                    binary_compare!(self, >=, "greater or equal");
                },
                OpCode::CmpEqual => {
                    binary_compare!(string, bool; self, ==, true, true, "equal");
                },
                OpCode::CmpNotEqual => {
                    binary_compare!(string, bool; self, !=, true, false, "not equal");
                },
                OpCode::CmpAnd => {
                    let (a, b, value_type) = self.binary_op_vals();

                    match value_type {
                        Value::Bool(_) => {
                            if a == b {
                                self.stack.push(Value::Bool(a.bool_s() && b.bool_s()));
                            } else {
                                panic!("Cannot compare for 'and' with any other type except 'bool'");
                            }
                        },
                        _ => panic!("Cannot compare for 'and' with any other type except 'bool'"),
                    }
                },
                OpCode::CmpOr => {
                    let (a, b, value_type) = self.binary_op_vals();

                    match value_type {
                        Value::Bool(_) => {
                            if a == b {
                                self.stack.push(Value::Bool(a.bool_s() || b.bool_s()));
                            } else {
                                panic!("Cannot compare for 'or' with any other type except 'bool'");
                            }
                        },
                        _ => panic!("Cannot compare for 'or' with any other type except 'bool'"),
                    }
                },
                OpCode::Jmp(idx) => {
                    self.idx = idx;
                },
                OpCode::JmpIfFalse(idx) => {
                    if !self.stack_pop().bool_s() {
                        self.idx = idx;
                    }
                },
                OpCode::Negate => {
                    let val = self.stack_pop();

                    match val {
                        Value::Uint(_) => panic!("Cannot negate a value under the type 'uint'."),
                        Value::Int(x) => self.stack.push(Value::Int(-x)),
                        Value::Decimal(x) => self.stack.push(Value::Decimal(-x)),
                        Value::Str(_) => panic!("Cannot negate a value under the type 'string'."),
                        Value::Bool(_) => panic!("Cannot negate a value under the type 'bool'."),
                        Value::None => panic!("Cannot negate a value under the type 'none'."),
                    }
                },
                OpCode::Not => {
                    let val = self.stack_pop();

                    match val {
                        Value::None => panic!("Cannot 'not' a value under the type 'none'."),
                        Value::Uint(_) => self.stack.push(Value::Bool(!val.bool_s())),
                        Value::Int(_) => self.stack.push(Value::Bool(!val.bool_s())),
                        Value::Decimal(_) => self.stack.push(Value::Bool(!val.bool_s())),
                        Value::Bool(_) => self.stack.push(Value::Bool(!val.bool_s())),
                        Value::Str(_) => panic!("Cannot 'not' a value under the type 'string'."),
                    }
                }
                OpCode::ArgumentStore(name, ctype) => {
                    self.environment.add_variable(name, ctype, self.call_stack.pop().unwrap());
                },
                OpCode::Store(name, ctype) => {
                    let val = self.stack_pop();
                    self.environment.add_variable(name, ctype, val);
                },
                OpCode::InferStore(name) => {
                    let val = self.stack_pop();
                    self.environment.add_infer_variable(name, val);
                }
                OpCode::Load(name) => {
                    let entry = self.environment.get_variable(name);
                    
                    if let EnvEntry::Variable(_, value, _, _) = entry {
                        self.stack.push(value);
                    }
                },
                OpCode::Assign(name) => {
                    let val = self.stack_pop();
                    self.environment.assign_variable(name, val);
                },
                OpCode::PopToCall => {
                    let value = self.stack_pop();
                    self.call_stack.push(value);
                },
                OpCode::StartScope => {
                    self.environment.current_scope += 1;
                },
                OpCode::EndScope => {
                    if self.environment.current_scope != 0 {
                        self.environment.current_scope -= 1;
                    }
                    
                    self.environment.remove_from_scope(self.environment.current_scope + 1);
                },
            }
        }
    }
}