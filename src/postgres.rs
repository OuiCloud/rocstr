//! Postgres type implementation

extern crate std;

use std::boxed::Box;
use std::error::Error;

use bytes::BytesMut;
use postgres_types::FromSql;
use postgres_types::IsNull;
use postgres_types::ToSql;
use postgres_types::Type;

use crate::RocStr;

impl<'sql, const SIZE: usize> FromSql<'sql> for RocStr<SIZE> {
    fn from_sql(ty: &Type, raw: &'sql [u8]) -> Result<RocStr<SIZE>, Box<dyn Error + Sync + Send>> {
        <&str as FromSql>::from_sql(ty, raw).map(RocStr::from)
    }

    fn accepts(ty: &Type) -> bool {
        <&str as FromSql>::accepts(ty)
    }
}

impl<const SIZE: usize> ToSql for RocStr<SIZE> {
    fn to_sql(&self, ty: &Type, w: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        <&str as ToSql>::to_sql(&self.as_str(), ty, w)
    }

    fn accepts(ty: &Type) -> bool {
        <&str as ToSql>::accepts(ty)
    }

    fn to_sql_checked(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        postgres_types::__to_sql_checked(self, ty, out)
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::*;

    #[test]
    fn from_sql_for_rocstr_should_accept_all_postgres_char_type() {
        assert!(
            <RocStr::<16> as FromSql>::accepts(&Type::BPCHAR),
            "❌ FromSql for RocStr does not accept BPCHAR"
        );
        assert!(
            <RocStr::<16> as FromSql>::accepts(&Type::VARCHAR),
            "❌ FromSql for RocStr does not accept VARCHAR"
        );
        assert!(
            <RocStr::<16> as FromSql>::accepts(&Type::TEXT),
            "❌ FromSql for RocStr does not accept VARCHAR"
        );
    }

    #[test]
    fn to_sql_for_rocstr_should_accept_all_postgres_char_type() {
        assert!(
            <RocStr::<16> as ToSql>::accepts(&Type::BPCHAR),
            "❌ ToSql for RocStr does not accept BPCHAR"
        );
        assert!(
            <RocStr::<16> as ToSql>::accepts(&Type::VARCHAR),
            "❌ ToSql for RocStr does not accept VARCHAR"
        );
        assert!(
            <RocStr::<16> as ToSql>::accepts(&Type::TEXT),
            "❌ ToSql for RocStr does not accept VARCHAR"
        );
    }

    #[test]
    fn rocstr_from_sql_should_contain_the_field_str() {
        let expected = RocStr::<16>::from("foo");
        let raw = b"foo";
        let ty = Type::VARCHAR;

        let result = RocStr::<16>::from_sql(&ty, raw);
        assert!(result.is_ok());

        let value = result.unwrap();
        assert_eq!(value, expected);
    }

    #[test]
    fn rocstr_to_sql_checked_with_valid_type_should_success() {
        let value = RocStr::<16>::from("foo checked");
        let expected = Bytes::from_static(b"foo checked");
        let mut out = BytesMut::new();
        let ty = Type::BPCHAR;

        let result = value.to_sql_checked(&ty, &mut out);
        assert!(result.is_ok());
        assert_eq!(out, expected);
    }

    #[test]
    fn rocstr_to_sql_checked_with_an_invalid_type_should_fail() {
        let value = RocStr::<16>::from("foo checked");
        let mut out = BytesMut::new();
        let ty = Type::INT2;

        let result = value.to_sql_checked(&ty, &mut out);
        assert!(result.is_err());
    }
}
