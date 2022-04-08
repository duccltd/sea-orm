use entity::sea_orm::migration::*;
pub use sea_schema::migration::prelude::*;

mod m20220120_000001_create_post_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    type Connection = DbConn;

    fn migrations() -> Vec<Box<dyn MigrationTrait<Self::Connection>>> {
        vec![Box::new(m20220120_000001_create_post_table::Migration)]
    }
}
