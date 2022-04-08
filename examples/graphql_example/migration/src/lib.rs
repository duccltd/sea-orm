pub use sea_schema::migration::*;

mod m20220101_000001_create_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    type Connection = DbConn;

    fn migrations() -> Vec<Box<dyn MigrationTrait<Self::Connection>>> {
        vec![Box::new(m20220101_000001_create_table::Migration)]
    }
}
