use crate::{DbErr, QueryResult};
use sea_schema::migration::MigrationQueryResult;

impl MigrationQueryResult for QueryResult {
    type Error = DbErr;

    fn try_get_i64(&self, col: &str) -> Result<i64, Self::Error> {
        self.try_get("", col)
    }

    fn try_get_string(&self, col: &str) -> Result<String, Self::Error> {
        self.try_get("", col)
    }
}
