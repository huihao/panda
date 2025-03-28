use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef}, Result};
use url::Url;

#[derive(Debug, Clone)]
pub struct DateTimeWrapper(pub DateTime<Utc>);

impl FromSql for DateTimeWrapper {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Text(s) => {
                let s = std::str::from_utf8(s).map_err(|_| FromSqlError::InvalidType)?;
                DateTime::parse_from_rfc3339(s)
                    .map(|dt| DateTimeWrapper(dt.with_timezone(&Utc)))
                    .map_err(|_| FromSqlError::InvalidType)
            }
            ValueRef::Integer(i) => {
                Ok(DateTimeWrapper(Utc.timestamp_opt(i, 0).unwrap()))
            }
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

impl ToSql for DateTimeWrapper {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.0.to_rfc3339()))
    }
}

impl From<DateTime<Utc>> for DateTimeWrapper {
    fn from(dt: DateTime<Utc>) -> Self {
        DateTimeWrapper(dt)
    }
}

impl From<DateTimeWrapper> for DateTime<Utc> {
    fn from(wrapper: DateTimeWrapper) -> Self {
        wrapper.0
    }
}

#[derive(Debug, Clone)]
pub struct UrlWrapper(pub Url);

impl FromSql for UrlWrapper {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Text(text) => {
                let text = std::str::from_utf8(text)
                    .map_err(|_| FromSqlError::InvalidType)?;
                let url = Url::parse(text)
                    .map_err(|_| FromSqlError::InvalidType)?;
                Ok(UrlWrapper(url))
            }
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

impl ToSql for UrlWrapper {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.0.to_string()))
    }
} 