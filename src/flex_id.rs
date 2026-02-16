use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// A flexible ID type that can deserialize from number, string, or null
/// This handles inconsistent API responses from IPTV providers
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FlexId {
    Number(i64),
    String(String),
    Null,
}

impl Default for FlexId {
    fn default() -> Self {
        FlexId::Null
    }
}

impl FlexId {
    /// Create a FlexId from a number
    pub fn from_number(n: i64) -> Self {
        FlexId::Number(n)
    }
    
    /// Create a FlexId from a string
    pub fn from_string(s: String) -> Self {
        FlexId::String(s)
    }
    
    /// Get as i64 if this is a number
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            FlexId::Number(n) => Some(*n),
            FlexId::String(s) => s.parse().ok(),
            FlexId::Null => None,
        }
    }
    
    /// Get as &str if this is a string
    pub fn as_str(&self) -> Option<&str> {
        match self {
            FlexId::String(s) => Some(s),
            FlexId::Number(_n) => None, // Don't convert number to string
            FlexId::Null => None,
        }
    }
    
    /// Get as string (converts numbers to string)
    pub fn to_string_value(&self) -> Option<String> {
        match self {
            FlexId::Number(n) => Some(n.to_string()),
            FlexId::String(s) => Some(s.clone()),
            FlexId::Null => None,
        }
    }
    
    /// Check if this is null
    pub fn is_null(&self) -> bool {
        matches!(self, FlexId::Null)
    }
    
    /// Check if this has a value
    pub fn is_some(&self) -> bool {
        !self.is_null()
    }
}

impl fmt::Display for FlexId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FlexId::Number(n) => write!(f, "{}", n),
            FlexId::String(s) => write!(f, "{}", s),
            FlexId::Null => write!(f, "null"),
        }
    }
}

impl Serialize for FlexId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            FlexId::Number(n) => serializer.serialize_i64(*n),
            FlexId::String(s) => serializer.serialize_str(s),
            FlexId::Null => serializer.serialize_none(),
        }
    }
}

impl<'de> Deserialize<'de> for FlexId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, Visitor};
        
        struct FlexIdVisitor;
        
        impl<'de> Visitor<'de> for FlexIdVisitor {
            type Value = FlexId;
            
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a number, string, or null")
            }
            
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(FlexId::Number(v))
            }
            
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(FlexId::Number(v as i64))
            }
            
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                // Try to parse as number first
                if let Ok(n) = v.parse::<i64>() {
                    Ok(FlexId::Number(n))
                } else {
                    Ok(FlexId::String(v.to_string()))
                }
            }
            
            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if let Ok(n) = v.parse::<i64>() {
                    Ok(FlexId::Number(n))
                } else {
                    Ok(FlexId::String(v))
                }
            }
            
            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(FlexId::Null)
            }
            
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(FlexId::Null)
            }
        }
        
        deserializer.deserialize_any(FlexIdVisitor)
    }
}

/// Helper function to deserialize a flexible u64
pub fn deserialize_flex_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let flex = FlexId::deserialize(deserializer)?;
    Ok(flex.as_i64().unwrap_or(0) as u64)
}

/// Helper function to deserialize a flexible f32
pub fn deserialize_flex_f32<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    
    let flex = FlexId::deserialize(deserializer)?;
    match flex {
        FlexId::Number(n) => Ok(n as f32),
        FlexId::String(s) => s.parse().map_err(|_| D::Error::custom("invalid float")),
        FlexId::Null => Ok(0.0),
    }
}

/// Helper function to deserialize a flexible Option<f32>
pub fn deserialize_flex_option_f32<'de, D>(deserializer: D) -> Result<Option<f32>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    
    let flex = FlexId::deserialize(deserializer)?;
    match flex {
        FlexId::Number(n) => Ok(Some(n as f32)),
        FlexId::String(s) => s.parse().map(Some).map_err(|_| D::Error::custom("invalid float")),
        FlexId::Null => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_number_deserialize() {
        let json = "123";
        let id: FlexId = serde_json::from_str(json).unwrap();
        assert_eq!(id, FlexId::Number(123));
        assert_eq!(id.as_i64(), Some(123));
    }
    
    #[test]
    fn test_string_deserialize() {
        let json = r#""abc123""#;
        let id: FlexId = serde_json::from_str(json).unwrap();
        assert_eq!(id, FlexId::String("abc123".to_string()));
    }
    
    #[test]
    fn test_numeric_string_deserialize() {
        let json = r#""456""#;
        let id: FlexId = serde_json::from_str(json).unwrap();
        assert_eq!(id, FlexId::Number(456));
    }
    
    #[test]
    fn test_null_deserialize() {
        let json = "null";
        let id: FlexId = serde_json::from_str(json).unwrap();
        assert_eq!(id, FlexId::Null);
    }
}
