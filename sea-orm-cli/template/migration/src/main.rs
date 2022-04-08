use migration::Migrator;
use entity::sea_orm::migration::*;
use sea_schema::migration::prelude::*;

#[async_std::main]
async fn main() {
    cli::run_cli(Migrator).await;
}
