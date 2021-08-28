#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    None,
    Uint(u64),
    Int(i64),
    Decimal(f64),
    Bool(bool),
    Str(String),
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum ClassType {
    Any,
    Uint,
    Int,
    Decimal,
    Str,
    Bool,
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
            Value::None => panic!("Cannot add a value of 'none'"),
            //_ => panic!("Cannot add these values.")
        }
    };
}

impl Value {
    pub fn print(&self) {
        match self {
            Self::Int(x) => print!("{}", x),
            Self::Uint(x) => print!("{}", x),
            Self::Decimal(x) => print!("{}", x),
            Self::Bool(x) => print!("{}", x),
            Self::Str(x) => print!("{}", x),
            Self::None => print!("None"),
            //_ => print!("No Value.")
        }
    }

    pub fn print_type(&self) {
        match self {
            Self::Int(_) => print!("int"),
            Self::Uint(_) => print!("uint"),
            Self::Decimal(_) => print!("decimal"),
            Self::Bool(_) => print!("bool"),
            Self::Str(_) => print!("string"),
            Self::None => print!("none"),
            //_ => print!("unknown value")
        }
    }

    pub fn println(&self) {
        self.print();
        println!();
    }

    pub fn debug_print(&self) {
        print!("Value: ");
        println!("{:?}", self);
    }

    pub fn int_s(&self) -> i64 {
        match self {
            Self::Int(x) => return *x,
            Self::Uint(x) => return *x as i64,
            Self::Decimal(x) => return *x as i64,
            Self::Bool(_) => panic!("Cannot convert a value of 'bool' to 'int'."),
            Self::Str(_) => panic!("Cannot convert a value of 'string' to 'int'."),
            Self::None => 0,
            //_ => panic!("Unknown value used to convert to 'int'."),
        }
    }

    pub fn uint_s(&self) -> u64 {
        match self {
            Self::Int(x) => return *x as u64,
            Self::Uint(x) => return *x,
            Self::Decimal(x) => return *x as u64,
            Self::Bool(_) => panic!("Cannot convert a value of 'bool' to 'uint'."),
            Self::Str(_) => panic!("Cannot convert a value of 'string' to 'uint'."),
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
            Self::Str(_) => panic!("Cannot convert a value of 'string' to 'decimal'."),
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
            Self::Str(_) => panic!("Cannot convert a value of 'string' to 'bool'."),
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