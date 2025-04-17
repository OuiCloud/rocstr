//! Serde Serialize and Deserialize implementation

use serde::Deserialize;
use serde::Serialize;

use crate::rocstr::RocStr;

struct RocStrVisitor<const SIZE: usize>;

impl<const SIZE: usize> Serialize for RocStr<SIZE> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_str().serialize(serializer)
    }
}

impl<'de, const SIZE: usize> Deserialize<'de> for RocStr<SIZE> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(RocStrVisitor::<SIZE>)
    }
}

#[cfg(not(feature = "std"))]
mod no_std_rocstr {
    use core::fmt;

    use serde::de::Visitor;

    impl<'de, const SIZE: usize> Visitor<'de> for RocStrVisitor<SIZE> {
        type Value = RocStr<SIZE>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a valid utf-8 string")
        }

        fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            match v {
                true => Ok(RocStr::<SIZE>::from("true")),
                false => Ok(RocStr::<SIZE>::from("false")),
            }
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(RocStr::from(v).reshape::<SIZE>())
        }

        fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(RocStr::from(v).reshape::<SIZE>())
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(RocStr::from(v).reshape::<SIZE>())
        }

        fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(RocStr::from(v).reshape::<SIZE>())
        }

        fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let mut buffer = [0; 4];
            let encoded = v.encode_utf8(&mut buffer);
            Ok(RocStr::from(encoded as &str))
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(RocStr::from(v))
        }

        fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(RocStr::from(v))
        }
    }
}

#[cfg(feature = "std")]
mod standard_rocstr {
    extern crate std;

    use serde::de::Visitor;

    use super::RocStr;
    use super::RocStrVisitor;

    use core::fmt;
    use std::string::String;

    impl<'de, const SIZE: usize> Visitor<'de> for RocStrVisitor<SIZE> {
        type Value = RocStr<SIZE>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a valid utf-8 string")
        }

        fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            match v {
                true => Ok(RocStr::<SIZE>::from("true")),
                false => Ok(RocStr::<SIZE>::from("false")),
            }
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(RocStr::from(v).reshape::<SIZE>())
        }

        fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(RocStr::from(v).reshape::<SIZE>())
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(RocStr::from(v).reshape::<SIZE>())
        }

        fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(RocStr::from(v).reshape::<SIZE>())
        }

        fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let mut buffer = [0; 4];
            let encoded = v.encode_utf8(&mut buffer);
            Ok(RocStr::from(encoded as &str))
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(RocStr::from(v))
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(RocStr::from(v))
        }

        fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(RocStr::from(v))
        }
    }
}

mod tests {
    use super::*;

    #[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
    struct SerdeStruct {
        id: u64,
        name: RocStr<64>,
    }

    #[test]
    fn serialized_to_json_should_be_the_values_as_json() {
        let value = SerdeStruct {
            id: 42,
            name: "foo".into(),
        };

        let serialized = serde_json::to_string(&value);
        assert!(serialized.is_ok(), "❌ {}", serialized.err().unwrap());

        let serialized = serialized.unwrap();
        assert_eq!(serialized, r#"{"id":42,"name":"foo"}"#);
    }

    #[test]
    fn deserialized_from_json_should_contain_the_values() {
        let expected = SerdeStruct {
            id: 42,
            name: "foo".into(),
        };

        let deserialized = serde_json::from_str::<SerdeStruct>(r#"{"id":42,"name":"foo"}"#);

        assert!(deserialized.is_ok(), "❌ {}", deserialized.err().unwrap());

        let deserialized = deserialized.unwrap();
        assert_eq!(deserialized, expected);
    }

    #[test]
    fn serialized_to_toml_should_be_the_values_as_toml() {
        let value = SerdeStruct {
            id: 42,
            name: "foo".into(),
        };

        let serialized = toml::to_string(&value);
        assert!(serialized.is_ok(), "❌ {}", serialized.err().unwrap());

        let serialized = serialized.unwrap();
        assert_eq!(serialized, "id = 42\nname = \"foo\"\n");
    }

    #[test]
    fn deserialized_from_toml_should_contain_the_values() {
        let expected = SerdeStruct {
            id: 42,
            name: "foo".into(),
        };

        let deserialized = toml::from_str::<SerdeStruct>("id = 42\nname = \"foo\"\n");

        match deserialized {
            Ok(deserialized) => assert_eq!(deserialized, expected),
            Err(e) => panic!("❌ {e}"),
        }
    }
}
