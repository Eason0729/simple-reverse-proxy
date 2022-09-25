#[derive(Debug)]
pub enum Level {
    Level(String, Vec<Level>), // TODO: level require level's name
    List(Value),
    Unspecified(Value),
}

#[derive(Debug)]
pub enum Value {
    String(String),
    Bool(bool),
    Number(f64),
}

pub enum Error {
    Unknown,
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        let value = value.trim().to_string();
        if value.starts_with("\"") && value.ends_with("\"") {
            let content = value
                .strip_prefix("\"")
                .unwrap()
                .strip_suffix("\"")
                .unwrap();
            return Value::String(content.to_string());
        }
        if value == "True" || value == "true" {
            return Value::Bool(true);
        }
        if value == "False" || value == "false" {
            return Value::Bool(false);
        }
        let number = value.parse::<f64>();
        if let Ok(n) = number {
            return Value::Number(n);
        }
        Value::String(value)
    }
}

impl From<&String> for Value {
    fn from(value: &String) -> Self {
        value.into()
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        value.into()
    }
}
