use std::{fmt};
use crate::{value::{ Value, ClassType }};

#[derive(Clone)]
pub enum EnvEntry {
    Function(String, ClassType, usize, usize),
    NativeFunction(String, usize, &'static dyn Fn(Vec<Value>) -> Value),
    Variable(String, Value, ClassType, usize),
    Structure(CopperStruct),
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct CopperStruct {
    pub name: String,
    pub field_names: Vec<String>,
    pub field_values: Vec<Value>,
}

impl CopperStruct {
    pub fn new(name: String) -> Self {
        Self {
            name,
            field_names: Vec::new(),
            field_values: Vec::new(),
        }
    }

    pub fn set(&mut self, names: Vec<String>, value: Value) {
        let mut struct_place_holder = self.clone();
        let mut structure = &mut struct_place_holder;

        for i in 0..names.len()-1 {
            let name = names[i].clone();

            for s in 0..structure.field_names.len() {
                if structure.field_names[s] == name {
                    let value = &mut structure.field_values[s];
                    match value {
                        Value::Struct(cs) => {
                            structure = cs;
                            break;
                        }
                        _ => panic!("Expected to find a structure"),
                    }
                }
            }
        }

        for i in 0..structure.field_names.len() {
            if structure.field_names[i] == names[names.len()-1] {
                structure.field_values[i] = value.clone();
                *self = struct_place_holder.clone();
                return;
            }
        }


        panic!("Cannot set fields '{:?}' on structure as it doesn't exist!", names);
    }

    pub fn get(&self, name: String) -> Value {
        if self.field_names.contains(&name) {
            for i in 0..self.field_names.len() {
                if self.field_names[i] == name.clone() {
                    return self.field_values[i].clone();
                }
            }
        }

        panic!("Cannot get field '{}' on structure as it doesn't exist!", name);
    }

    pub fn insert(&mut self, name: String, value: Value) {
        if self.field_names.contains(&name) {
            for i in 0..self.field_names.len() {
                if self.field_names[i] == name {
                    self.field_values[i] = value;
                    return;
                }
            }
        }

        self.field_names.push(name);
        self.field_values.push(value);
    }
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

    pub fn add_struct(&mut self, structure: CopperStruct) {
        self.entries.push(EnvEntry::Structure(structure));
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
                _ => {},
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
                _ => {},
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
                _ => {},
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

        let mut entry = EnvEntry::Variable(name, val.clone(), ctype.clone(), self.current_scope);

        if let EnvEntry::Variable(_, value, _, _) = &mut entry {
            match ctype.clone() {
                ClassType::Any => {},
                ClassType::Uint => *value = Value::Uint(val.uint_s()),
                ClassType::Int => *value = Value::Int(val.int_s()),
                ClassType::Decimal => *value = Value::Decimal(val.decimal_s()),
                ClassType::Str => *value = Value::Str(val.string_s()),
                ClassType::Bool => *value = Value::Bool(val.bool_s()),
                ClassType::Struct(name) => {
                    if let Value::Struct(x) = value {
                        if x.name == name {
                            
                        } else {
                            panic!("Cannot convert the value of structure to another structure that is different");
                        }
                    } else {
                        panic!("Cannot convert a value to the struct.");
                    }
                }
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

        let ctype = match val.clone() {
            Value::None => ClassType::Any,
            Value::Uint(_) => ClassType::Uint,
            Value::Int(_) => ClassType::Int,
            Value::Decimal(_) => ClassType::Decimal,
            Value::Bool(_) => ClassType::Bool,
            Value::Str(_) => ClassType::Str,
            Value::Struct(cs) => ClassType::Struct(cs.name.clone()),
        };

        let entry = EnvEntry::Variable(name, val, ctype, self.current_scope);

        self.entries.push(entry);
    }

    pub fn get_struct(&self, name: String) -> CopperStruct {
        for i in &self.entries {
            if let EnvEntry::Structure(structure) = i {
                if name == *structure.name {
                    return structure.clone();
                }
            }
        }

        panic!("Cannot find a struct by the name of '{}'!", name);
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
                        match ctype.clone() {
                            ClassType::Any => *value = val.clone(),
                            ClassType::Uint => *value = Value::Uint(val.uint_s()),
                            ClassType::Int => *value = Value::Int(val.int_s()),
                            ClassType::Decimal => *value = Value::Decimal(val.decimal_s()),
                            ClassType::Str => *value = Value::Str(val.string_s()),
                            ClassType::Bool => *value = Value::Bool(val.bool_s()),
                            ClassType::Struct(name) => *value = Value::Struct(val.struct_s(name)),
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
            EnvEntry::Structure(structure) => {
                write!(f, "\tStructure '{}': \n", structure.name)?;
                
                for i in 0..structure.field_names.len() {
                    write!(f, "\t\tField '{}': {:?}\n", structure.field_names[i], structure.field_values[i])?;
                }

                write!(f, "\tEnd of structure\n")
            }
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