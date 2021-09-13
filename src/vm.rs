use crate::environment::{EnvEntry, Environment};
use crate::value::{ClassType, Value};
use crate::chunk::{Chunk, OpCode};

pub struct VM<'a> {
    pub chunk: &'a Chunk,
    pub idx: usize,
    pub stack: Vec<Value>,
    pub call_stack: Vec<Value>,

    pub environment: Environment,

    pub function_starting_scope: Vec<usize>,
    pub function_jump_back: Vec<usize>, 
    pub function_return_types: Vec<ClassType>,
}

// Set of macros for making the repetitive task of comparing with binary less tedious.
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
            Value::Struct(_) => panic!("Cannot compare for '{}' with structures.", $type),
            Value::None => panic!("Cannot compare for '{}' with 'none' type.", $type),
        }
    };

    (string, bool; $self:expr, $op:tt, $type:literal) => {
        let (a, b, value_type) = $self.binary_op_vals();
        
        match value_type {
            Value::Uint(_) => $self.stack.push(Value::Bool(a.uint_s() $op b.uint_s())),
            Value::Int(_) => $self.stack.push(Value::Bool(a.int_s() $op b.int_s())),
            Value::Decimal(_) => $self.stack.push(Value::Bool(a.decimal_s() $op b.decimal_s())),
            Value::Str(_) => {
                $self.stack.push(Value::Bool(a.string_s() $op b.string_s()));
            },
            Value::Bool(_) => {
                $self.stack.push(Value::Bool(a.bool_s() $op b.bool_s()));
            },
            Value::Struct(_) => panic!("Cannot compare for '{}' with structures.", $type),
            Value::None => panic!("Cannot compare for '{}' with 'none' type.", $type),
        }
    };
}

impl<'a> VM<'a> {
    pub fn new(chunk: &'a Chunk) -> VM {
        return VM { chunk, idx: 0, environment: Environment::new(), stack: Vec::new(), call_stack: Vec::new(), function_starting_scope: Vec::new(), function_jump_back: Vec::new(), function_return_types: Vec::new() }
    }

    fn read_op(&mut self) -> OpCode {
        let result = self.chunk.code[self.idx].clone();
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

        let mut value_type: Value = Value::None;

        if value_type < a {
            value_type = a.clone();
        }

        if value_type < b {
            value_type = b.clone();
        }

        return (a, b, value_type);
    }

    pub fn interpret(&mut self) {
        // These are set for debugging the internal process of the interpeter
        const DEBUGGING: bool = false;

        // This is set for debugging current amount of values used in memory.
        const DEBUG_MEMORY: bool = false;

        // If set to true, then it means it will print amount of values on the stack every 1000 objects.
        // Turn to false if you want to see the amount of values for every op in the vm.
        const DEBUG_MEMORY_LEN_1000: bool = false;

        // Debug what the current scope of the vm enviroment is.
        const DEBUG_SCOPE: bool = false;
        
        loop {
            if DEBUG_MEMORY && (!DEBUG_MEMORY_LEN_1000 || self.stack.len() % 1000 == 0) {
                println!("Current amount of values on stack: {}", self.stack.len());
            }

            if DEBUG_SCOPE {
                println!("Current Scope: {}", self.environment.current_scope);
            }

            // This is the full debugging experience.
            if DEBUGGING {
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
                        let jmp_back = self.function_starting_scope.pop().unwrap();
                        self.environment.remove_from_scope(jmp_back);
                        self.environment.current_scope = jmp_back - 1;
                        
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
                            ClassType::Struct(x) => {
                                let value = self.stack_pop();
                                self.stack.push(Value::Struct(value.struct_s(x)));
                            }
                        }
                    } else {
                        panic!("Cannot return out of the script, only in function.");
                    }
                },
                OpCode::StructSet(name, sets) => {
                    let mut variable = self.environment.get_variable(name.clone());
                    let mut value = self.stack_pop();
                    
                    if let EnvEntry::Variable(_, val, _, _) = &mut variable {
                        if let Value::Struct(cs) = val {
                            cs.set(sets, value);
                            value = Value::Struct(cs.clone());
                        } else {
                            panic!("Expected to set a struct");
                        }
                    }

                    self.environment.assign_variable(name, value);
                }
                OpCode::StructSetByIndex(index) => {
                    let structure = self.stack_pop();
                    let value = self.stack_pop();

                    let structure = match structure {
                        Value::Struct(cs) => {
                            let mut cs = cs;
                            if !(cs.field_values.len() <= index) {
                                cs.field_values[index] = value;
                            } else {
                                self.stack.pop();
                            }

                            cs
                        }
                        _ => panic!("Cannot assign by index as not a struct"),
                    };

                    self.stack.push(Value::Struct(structure));
                }
                OpCode::StructGet(name) => {
                    let value = self.stack_pop();
                    //println!("name: {}, val: {:?}", name.clone(), value.clone());
                    match value {
                        Value::Struct(cs) => {
                            //println!("Struct: "); cs.get(name.clone()).println();
                            self.stack.push(cs.get(name));
                        }
                        _ => panic!("Expected a struct!"),
                    }
                }
                OpCode::NewStruct(name) => {
                    self.stack.push(Value::Struct(
                        self.chunk.functions.get_struct(name) 
                    ));
                }
                OpCode::TransformToType(ctype) => {
                    let val = self.stack_pop();
                    match ctype {
                        ClassType::Any => self.stack.push(val),
                        ClassType::Uint => self.stack.push(Value::Uint(val.uint_s())),
                        ClassType::Int => self.stack.push(Value::Int(val.int_s())),
                        ClassType::Decimal => self.stack.push(Value::Decimal(val.decimal_s())),
                        ClassType::Str => self.stack.push(Value::Str(val.string_s())),
                        ClassType::Bool => self.stack.push(Value::Bool(val.bool_s())),
                        ClassType::Struct(x) => self.stack.push(Value::Struct(val.struct_s(x))),
                    }
                }
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
                    let func = self.chunk.functions.get_function(func_name.clone());

                    match func {
                        EnvEntry::Function(_, ctype, count, bytecode_pos) => {
                            if self.call_stack.len() != count {
                                for i in &self.call_stack {
                                    i.println();
                                }
                                panic!("Expected {} argument(s), but got {}", count, self.call_stack.len());
                            }

                            self.function_jump_back.push(self.idx);
                            self.function_starting_scope.push(self.environment.current_scope+1);
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
                    binary_compare!(string, bool; self, ==, "equal");
                },
                OpCode::CmpNotEqual => {
                    binary_compare!(string, bool; self, !=, "not equal");
                },
                OpCode::CmpAnd => {
                    let (a, b, _) = self.binary_op_vals();

                    self.stack.push(Value::Bool(a.bool_s() && b.bool_s()));
                    
                },
                OpCode::CmpOr => {
                    let (a, b, _) = self.binary_op_vals();

                    self.stack.push(Value::Bool(a.bool_s() || b.bool_s()));
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
                        Value::Struct(x) => panic!("Cannot negate a value under the type 'struct {}'.", x.name),
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
                        Value::Struct(x) => panic!("Cannot 'not' a value under the type 'struct {}'.", x.name),
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
                    self.environment.remove_from_scope(self.environment.current_scope);

                    if self.environment.current_scope != 0 {
                        self.environment.current_scope -= 1;
                    }
                },
            }
        }
    }
}