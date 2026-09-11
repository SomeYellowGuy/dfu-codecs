use std::fmt;
use std::fmt::{Display, Formatter};

/// Represents a Java number of a primitive type.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Number {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
}

impl From<Number> for i64 {
    fn from(num: Number) -> Self {
        match num {
            Number::Byte(b) => b as Self,
            Number::Short(s) => s as Self,
            Number::Int(i) => i as Self,
            Number::Long(l) => l,
            Number::Float(f) => f as Self,
            Number::Double(d) => d as Self,
        }
    }
}

impl From<Number> for i32 {
    fn from(num: Number) -> Self {
        match num {
            Number::Byte(b) => b as Self,
            Number::Short(s) => s as Self,
            Number::Int(i) => i,
            Number::Long(l) => l as Self,
            Number::Float(f) => f as Self,
            Number::Double(d) => d as Self,
        }
    }
}

impl From<Number> for i16 {
    fn from(num: Number) -> Self {
        i32::from(num) as Self
    }
}

impl From<Number> for i8 {
    fn from(num: Number) -> Self {
        i32::from(num) as Self
    }
}

impl From<Number> for u8 {
    fn from(num: Number) -> Self {
        i32::from(num) as Self
    }
}

impl From<Number> for f32 {
    fn from(num: Number) -> Self {
        match num {
            Number::Byte(b) => b as Self,
            Number::Short(s) => s as Self,
            Number::Int(i) => i as Self,
            Number::Long(l) => l as Self,
            Number::Float(f) => f,
            Number::Double(d) => d as Self,
        }
    }
}

impl From<Number> for f64 {
    fn from(num: Number) -> Self {
        match num {
            Number::Byte(b) => b as Self,
            Number::Short(s) => s as Self,
            Number::Int(i) => i as Self,
            Number::Long(l) => l as Self,
            Number::Float(f) => f as Self,
            Number::Double(d) => d,
        }
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Byte(v) => write!(f, "{v}"),
            Self::Short(v) => write!(f, "{v}"),
            Self::Int(v) => write!(f, "{v}"),
            Self::Long(v) => write!(f, "{v}"),
            Self::Float(v) => write!(f, "{v}"),
            Self::Double(v) => write!(f, "{v}"),
        }
    }
}

#[cfg(feature = "json")]
impl From<Number> for serde_json::Value {
    #[inline]
    fn from(num: Number) -> Self {
        match num {
            Number::Byte(n) => n.into(),
            Number::Short(n) => n.into(),
            Number::Int(n) => n.into(),
            Number::Long(n) => n.into(),
            Number::Float(n) => n.into(),
            Number::Double(n) => n.into(),
        }
    }
}

#[cfg(feature = "json")]
impl From<serde_json::Number> for Number {
    #[inline]
    fn from(num: serde_json::Number) -> Self {
        match num {
            n if n.is_i64() => Self::Long(n.as_i64().unwrap()),
            n if n.is_f64() => Self::Double(n.as_f64().unwrap()),
            n => {
                let n = n.as_u64().unwrap();
                if let Ok(signed) = i64::try_from(n) {
                    Self::Long(signed)
                } else {
                    Self::Double(n as f64)
                }
            }
        }
    }
}
