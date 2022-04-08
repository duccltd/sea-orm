use crate::DbErr;
use sea_schema::migration::IntoMigrationError;

impl IntoMigrationError for DbErr {
    fn into_migration_error(str: String) -> Self {
        DbErr::Migration(str)
    }
}
