use std::f32::consts::E;

#[derive(Debug)]
pub enum Error {
    Unknown,
    FieldNameNotFound,
    InvaildOperation,
    MisMatchType,
    MisMatchStructure,
    ParsingProvidedStruct,
}

#[derive(Debug)]
pub enum Level {
    Level(String, Vec<Level>),
    List(Value),
    Unspecified(Value),
}

#[derive(Debug)]
pub enum Value {
    String(String),
    Bool(bool),
    Number(f64),
}

impl Level {
    pub fn level(&self, path: Vec<&str>) -> Result<&Level, Error> {
        let mut current_level = self;
        for name in path {
            current_level = current_level.next_level(name)?;
        }
        Ok(current_level)
    }

    pub fn field_name(&self, path: Vec<String>) -> Result<&String, Error> {
        if let Level::Level(name, _) = self {
            Ok(name)
        } else {
            Err(Error::MisMatchStructure)
        }
    }

    pub fn value(&self, path: Vec<&str>) -> Result<&Value, Error> {
        let current_level = self.level(path)?.unwrap_level()?;
        return match current_level {
            Level::Unspecified(value) => Ok(value),
            _ => Err(Error::MisMatchStructure),
        };
    }

    pub fn list(&self, path: Vec<&str>) -> Result<Vec<&Value>, Error> {
        let current_level = self.level(path)?;
        return if let Level::Level(_, lists) = current_level {
            let mut result = vec![];
            for element in lists {
                if let Level::List(value) = element {
                    result.push(value);
                }
            }
            Ok(result)
        } else {
            Err(Error::MisMatchStructure)
        };
    }

    pub fn struct_list<'a, S>(&'a self, path: Vec<&str>) -> Result<Vec<S>, Error>
    where
        S: TryFrom<&'a Level>,
    {
        let current_level = self.level(path)?;
        return if let Level::Level(_, lists) = current_level {
            let mut result = vec![];
            for source in lists {
                result.push(
                    source
                        .try_into()
                        .map_err(|_| Error::ParsingProvidedStruct)?,
                )
            }
            Ok(result)
        } else {
            Err(Error::MisMatchStructure)
        };
    }

    fn unwrap_level(&self) -> Result<&Level, Error> {
        return if let Level::Level(_, levels) = self {
            Ok(&levels[0])
        } else {
            Err(Error::MisMatchStructure)
        };
    }

    fn next_level(&self, name: &str) -> Result<&Level, Error> {
        if let Level::Level(_, value) = self {
            for padding in value {
                if let Level::Level(exp_name, value) = padding {
                    if &name == exp_name {
                        return Ok(padding);
                    }
                }
            }
            Err(Error::Unknown)
        } else {
            Err(Error::InvaildOperation)
        }
    }
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
        value.clone().into()
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

impl TryInto<bool> for &Value {
    type Error = Error;

    fn try_into(self) -> Result<bool, Self::Error> {
        if let Value::Bool(a) = self {
            return Ok(*a);
        }
        Err(Error::MisMatchType)
    }
}

impl TryInto<f64> for &Value {
    type Error = Error;

    fn try_into(self) -> Result<f64, Self::Error> {
        if let Value::Number(a) = self {
            return Ok(*a);
        }
        Err(Error::MisMatchType)
    }
}

impl TryInto<i64> for &Value {
    type Error = Error;

    fn try_into(self) -> Result<i64, Self::Error> {
        if let Value::Number(a) = self {
            return Ok(*a as i64);
        }
        Err(Error::MisMatchType)
    }
}

impl TryInto<String> for &Value {
    type Error = Error;

    fn try_into(self) -> Result<String, Self::Error> {
        if let Value::String(a) = self {
            return Ok(a.clone());
        }
        Err(Error::MisMatchType)
    }
}
