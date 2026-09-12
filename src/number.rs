use std::fmt::{Display, Formatter, Result as FmtResult};

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
            Number::Byte(b) => Self::from(b),
            Number::Short(s) => Self::from(s),
            Number::Int(i) => Self::from(i),
            Number::Long(l) => l,
            Number::Float(f) => f as Self,
            Number::Double(d) => d as Self,
        }
    }
}

impl From<Number> for i32 {
    fn from(num: Number) -> Self {
        match num {
            Number::Byte(b) => Self::from(b),
            Number::Short(s) => Self::from(s),
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
            Number::Byte(b) => Self::from(b),
            Number::Short(s) => Self::from(s),
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
            Number::Byte(b) => Self::from(b),
            Number::Short(s) => Self::from(s),
            Number::Int(i) => Self::from(i),
            Number::Long(l) => l as Self,
            Number::Float(f) => Self::from(f),
            Number::Double(d) => d,
        }
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
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
        if let Some(n) = num.as_i64() {
            Self::Long(n)
        } else if let Some(n) = num.as_f64() {
            Self::Double(n)
        } else if let Some(n) = num.as_u64() {
            if let Ok(signed) = i64::try_from(n) {
                Self::Long(signed)
            } else {
                Self::Double(n as f64)
            }
        } else {
            // Fallback.
            Self::Byte(0)
        }
    }
}
