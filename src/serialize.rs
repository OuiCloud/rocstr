//! Serde Serialize and Deserialize implementation

use serde::Deserialize;
use serde::Serialize;

use crate::rocstr::RocStr;

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
        <&str>::deserialize(deserializer).map(Self::from)
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
        assert!(serialized.is_ok());

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

        assert!(deserialized.is_ok());

        let deserialized = deserialized.unwrap();
        assert_eq!(deserialized, expected);
    }
}
