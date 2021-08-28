use std::{collections::btree_map::Entry, fmt};
use crate::value::{ Value, ClassType };

#[derive(Clone)]
pub enum EnvEntry {
    Function(String, ClassType, usize, usize),
    NativeFunction(String, usize, &'static dyn Fn(Vec<Value>) -> Value),
    Variable(String, Value, ClassType, usize),
}

#[derive(Clone)]
pub struct Environment {
    pub entries: Vec<EnvEntry>,
    pub current_scope: usize,
}

impl Environment {
    pub fn new() -> Self {
        return Self { entries: Vec::new(), current_scope: 0 };
    }
    
    pub fn add_function(&mut self, name: String, ctype: ClassType, arg_count: usize, position: usize) {
        for i in &self.entries {
            match i {
                EnvEntry::Function(func_name, _, _, _) => if name == *func_name {
                    panic!("Cannot assign a function with name '{}' as one already exists.", name);
                },
                EnvEntry::NativeFunction(func_name, _, _) => if name == *func_name {
                    panic!("Cannot assign a function with name '{}' as one already exists.", name);
                },
                EnvEntry::Variable(_, _, _, _) => {},
            }
        }

        self.entries.push(EnvEntry::Function(name, ctype, arg_count, position));
    }
    
    pub fn add_native_function(&mut self, name: String, argument: usize, func: &'static dyn Fn(Vec<Value>)  -> Value) {
        for i in &self.entries {
            match i {
                EnvEntry::Function(func_name, _, _, _) => if name == *func_name {
                    panic!("Cannot assign a function with name '{}' as one already exists.", name);
                },
                EnvEntry::NativeFunction(func_name, _, _) => if name == *func_name {
                    panic!("Cannot assign a function with name '{}' as one already exists.", name);
                },
                EnvEntry::Variable(_, _, _, _) => {},
            }
        }

        self.entries.push(EnvEntry::NativeFunction(name, argument, func));
    }

    pub fn get_function(&self, name: String) -> EnvEntry {
        for i in &self.entries {
            match i {
                EnvEntry::Function(func_name, _, _, _) => if name == *func_name {
                    return i.clone();
                },
                EnvEntry::NativeFunction(func_name, _, _) => if name == *func_name {
                    return i.clone();
                },
                EnvEntry::Variable(_, _, _, _) => {},
            }
        }

        panic!("Couldn't find a variable by the name of '{}'.", name);
    }

    pub fn remove_from_scope(&mut self, scope_to_remove: usize) {
        let mut entries_removed = 0;

        for i in 0..self.entries.len() {
            let i = i - entries_removed;
            match self.entries[i] {
                EnvEntry::Variable(_, _, _, scope) => {
                    if scope >= scope_to_remove {
                        self.entries.remove(i);
                        entries_removed += 1;
                    }
                },
                _ => {},
            }
        }
    }
    
    pub fn add_variable(&mut self, name: String, ctype: ClassType, val: Value) {
        for v in 0..self.entries.len() {
            let i = &self.entries[v];
            match i {
                EnvEntry::Variable(var_name, _, _, var_scope) => 
                if name == *var_name && *var_scope == self.current_scope {
                    self.entries.remove(v);
                    break;
                },
                _ => {},
            }
        }

        let mut entry = EnvEntry::Variable(name, val.clone(), ctype, self.current_scope);

        if let EnvEntry::Variable(_, value, _, _) = &mut entry {
            match ctype {
                ClassType::Any => {},
                ClassType::Uint => *value = Value::Uint(val.uint_s()),
                ClassType::Int => *value = Value::Int(val.int_s()),
                ClassType::Decimal => *value = Value::Decimal(val.decimal_s()),
                ClassType::Str => *value = Value::Str(val.string_s()),
                ClassType::Bool => *value = Value::Bool(val.bool_s()),
            }
        }

        self.entries.push(entry);
    }

    pub fn add_infer_variable(&mut self, name: String, val: Value) {
        for v in 0..self.entries.len() {
            let i = &self.entries[v];
            match i {
                EnvEntry::Variable(var_name, _, _, var_scope) => 
                if name == *var_name && *var_scope == self.current_scope {
                    self.entries.remove(v);
                    break;
                },
                _ => {},
            }
        }

        let ctype = match val {
            Value::None => ClassType::Any,
            Value::Uint(_) => ClassType::Uint,
            Value::Int(_) => ClassType::Int,
            Value::Decimal(_) => ClassType::Decimal,
            Value::Bool(_) => ClassType::Bool,
            Value::Str(_) => ClassType::Str,
        };

        let entry = EnvEntry::Variable(name, val, ctype, self.current_scope);

        self.entries.push(entry);
    }
    
    pub fn get_variable(&self, name: String) -> EnvEntry {
        for r in 0..self.current_scope+1 {
            let r = self.current_scope - r;
            for i in &self.entries {
                if let EnvEntry::Variable(var_name, _, _, scope) = i {
                    if name == *var_name && r == *scope {
                        return i.clone();
                    }
                }
            }
        }
        
        panic!("Couldn't get a variable by the name of '{}'", &name);
    }
    
    pub fn assign_variable(&mut self, name: String, val: Value) {
        for r in 0..self.current_scope+1 {
            let r = self.current_scope - r;
            for i in &mut self.entries {
                if let EnvEntry::Variable(var_name, value, ctype, scope) = i {
                    if *var_name == name && *scope == r {
                        match ctype {
                            ClassType::Any => *value = val.clone(),
                            ClassType::Uint => *value = Value::Uint(val.uint_s()),
                            ClassType::Int => *value = Value::Int(val.int_s()),
                            ClassType::Decimal => *value = Value::Decimal(val.decimal_s()),
                            ClassType::Str => *value = Value::Str(val.string_s()),
                            ClassType::Bool => *value = Value::Bool(val.bool_s()),
                        }
                        return;
                    }
                }   
            }
        }
    }
}

impl fmt::Display for EnvEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnvEntry::Function(name, _, _, position) => {
                write!(f, "\tFunction: name['{}'], bytecode_position[{}]\n", name, position)
            },
            EnvEntry::NativeFunction(name,  _, _) => {
                write!(f, "\tNative Function: name['{}']\n", name)
            },
            EnvEntry::Variable(name, val, ctype, scope) => {
                write!(f, "\tVariable: name['{}'], type[{:?}], val[{:?}], scope[{}]\n", name, ctype, val, scope)
            },
        }
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{\n")?;
        for i in &self.entries {
            write!(f, "{}", i)?;
        }

        write!(f, "}}")
    }
}