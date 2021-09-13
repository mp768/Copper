use crate::environment::CopperStruct;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    None,
    Uint(u64),
    Int(i64),
    Decimal(f64),
    Str(String),
    Bool(bool),
    Struct(CopperStruct),
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ClassType {
    Any,
    Uint,
    Int,
    Decimal,
    Str,
    Bool,
    Struct(String),
}

macro_rules! binary_op_with_value {
    ($this:tt, $other:tt, $op:tt, $to_do_bool:block, $what_to_do_with_string:block) => {
        let mut type_used = Value::Uint(0);

        if *$this > type_used {
            type_used = $this.clone();
        }

        if *$other > type_used {
            type_used = $other.clone();
        }

        match type_used {
            Value::Uint(_) => return Value::Uint($this.uint_s() $op $other.uint_s()),
            Value::Int(_) => return Value::Int($this.int_s() $op $other.int_s()),
            Value::Decimal(_) => return Value::Decimal($this.decimal_s() $op $other.decimal_s()),
            Value::Bool(_) => $to_do_bool,
            Value::Str(_) => $what_to_do_with_string,
            Value::Struct(_) => panic!("Cannot add value to a struct."),
            Value::None => panic!("Cannot add a value of 'none'"),
            //_ => panic!("Cannot add these values.")
        }
    };
}

impl Value {
    fn print_struct(&self, increment: u32) {
        match self {
            Self::Struct(x) => {
                println!("Struct '{}'", x.name);
                for i in 0..x.field_names.len() {
                    for _ in 0..increment {
                        print!(" ");
                    }

                    print!("'{}': ", x.field_names[i]);
                    x.field_values[i].print_struct(increment+4);
                    println!();
                }
            }
            _ => self.print(),
        }
    }

    pub fn type_to_string(&self) -> String {
        match self {
            Self::Int(_) => "int",
            Self::Uint(_) => "uint",
            Self::Decimal(_) => "decimal",
            Self::Bool(_) => "bool",
            Self::Struct(cs) => cs.name.as_str(),
            Self::Str(_) => "string",
            Self::None => "none",
        }.to_string()
    }

    pub fn print(&self) {
        match self {
            Self::Int(x) => print!("{}", x),
            Self::Uint(x) => print!("{}", x),
            Self::Decimal(x) => print!("{}", x),
            Self::Bool(x) => print!("{}", x),
            Self::Str(x) => print!("{}", x),
            Self::Struct(_) => self.print_struct(4),
            Self::None => print!("NONE"),
            //_ => print!("No Value.")
        }
    }

    pub fn print_type(&self) {
        print!("{}", self.type_to_string());
    }

    pub fn println(&self) {
        self.print(); println!();
    }

    pub fn struct_s(&self, name: String) -> CopperStruct {
        match self {
            Value::Int(_) => panic!("Cannot convert a value of 'int' to 'struct {}'.", name),
            Value::Uint(_) => panic!("Cannot convert a value of 'uint' to 'struct {}'.", name),
            Value::Decimal(_) => panic!("Cannot convert a value of 'decimal' to 'struct {}'.", name),
            Value::Bool(_) => panic!("Cannot convert a value of 'bool' to 'struct {}'.", name),
            Value::Str(_) => panic!("Cannot convert a value of 'string' to 'struct {}'.", name),
            Value::Struct(x) => return x.clone(),
            Value::None => panic!("Cannot convert a value of 'none' to 'struct {}'.", name),
            //_ => panic!("Unknown value used to convert to 'int'."),
        }
    }

    pub fn int_s(&self) -> i64 {
        match self {
            Value::Int(x) => return *x,
            Value::Uint(x) => return *x as i64,
            Value::Decimal(x) => return *x as i64,
            Value::Bool(_) => panic!("Cannot convert a value of 'bool' to 'int'."),
            Value::Str(x) => return x.trim().parse().unwrap(),
            Value::Struct(_) => panic!("Cannot convert a struct to any value."),
            Value::None => 0,
            //_ => panic!("Unknown value used to convert to 'int'."),
        }
    }

    pub fn uint_s(&self) -> u64 {
        match self {
            Self::Int(x) => return *x as u64,
            Self::Uint(x) => return *x,
            Self::Decimal(x) => return *x as u64,
            Self::Bool(_) => panic!("Cannot convert a value of 'bool' to 'uint'."),
            Self::Str(x) => return x.trim().parse().unwrap(),
            Value::Struct(_) => panic!("Cannot convert a struct to any value."),
            Self::None => 0,
            //_ => panic!("Unknown value used to convert to 'int'."),
        }
    }

    pub fn decimal_s(&self) -> f64 {
        match self {
            Self::Int(x) => return *x as f64,
            Self::Uint(x) => return *x as f64,
            Self::Decimal(x) => return *x,
            Self::Bool(_) => panic!("Cannot convert a value of 'bool' to 'decimal'."),
            Self::Str(x) => return x.trim().parse().unwrap(),
            Value::Struct(_) => panic!("Cannot convert a struct to any value."),
            Self::None => 0.0,
            //_ => panic!("Unknown value used to convert to 'int'."),
        }
    }

    pub fn string_s(&self) -> String {
        match self {
            Self::Int(x) => return x.to_string(),
            Self::Uint(x) => return x.to_string(),
            Self::Decimal(x) => return x.to_string(),
            Self::Bool(x) => return x.to_string(),
            Self::Str(x) => return x.clone(),
            Value::Struct(cs) => {
                let mut string = String::new();
                string.push_str(format!("struct '{}': ", cs.name).as_str());
                
                for i in 0..cs.field_names.len() {
                    string.push_str(format!("'{}': ", cs.field_names[i]).as_str());
                    string.push_str(cs.field_values[i].string_s().as_str());

                    if i != cs.field_names.len()-1 {
                        string.push_str(", ");
                    }
                }

                return string;
            },
            Self::None => String::from(""),
            //_ => panic!("Unknown value used to convert to 'int'."),
        }
    }

    pub fn bool_s(&self) -> bool {
        match self {
            Self::Int(_) => panic!("Cannot convert a value of 'int' to 'bool'."),
            Self::Uint(_) => panic!("Cannot convert a value of 'uint' to 'bool'."),
            Self::Decimal(_) => panic!("Cannot convert a value of 'decimal' to 'bool'."),
            Self::Bool(x) => return *x,
            Self::Str(x) => return x.trim().parse().unwrap(),
            Value::Struct(_) => panic!("Cannot convert a struct to any value."),
            Self::None => false,
            //_ => panic!("Unknown value used to convert to 'int'."),
        }
    }

    pub fn div_s(&self, other: &Value) -> Value {
        binary_op_with_value!(self, other, /, { panic!("Cannot divide with a bool type in operation.") }, { panic!("Cannot divide with a string type in operation.") });
    }

    pub fn mul_s(&self, other: &Value) -> Value {
        binary_op_with_value!(self, other, *, { panic!("Cannot multiple with a bool type in operation.") }, { panic!("Cannot multiple with a string type in operation.") });
    }

    pub fn sub_s(&self, other: &Value) -> Value {
        binary_op_with_value!(self, other, -, { panic!("Cannot subtract with a bool type in operation.") }, { panic!("Cannot subtract with a string type in operation.") });
    }

    pub fn add_s(&self, other: &Value) -> Value {
        binary_op_with_value!(self, other, +, { panic!("Cannot add with a bool type in operation.") }, {
            let mut str = self.string_s();
            str.push_str(&other.string_s());
            return Value::Str(str);
        });
    }
}