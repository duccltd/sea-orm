use crate::{ConnectionTrait, DatabaseConnection, DbBackend, DbErr, QueryResult, Statement};
use sea_schema::migration::{MigrationConnection, MigrationDbBackend, MigrationStatementBuilder};

#[async_trait::async_trait]
impl MigrationConnection for DatabaseConnection {
    type Connection = DatabaseConnection;

    type QueryResult = QueryResult;

    type Error = DbErr;

    async fn query_one<S>(&self, stmt: &S) -> Result<Option<Self::QueryResult>, Self::Error>
    where
        S: MigrationStatementBuilder + Sync,
    {
        let builder = MigrationConnection::get_database_backend(self);
        let (sql, values) = stmt.build(&builder);
        let stmt = Statement::from_sql_and_values(
            ConnectionTrait::get_database_backend(self),
            &sql,
            values,
        );
        ConnectionTrait::query_one(self, stmt).await
    }

    async fn query_all<S>(&self, stmt: &S) -> Result<Vec<Self::QueryResult>, Self::Error>
    where
        S: MigrationStatementBuilder + Sync,
    {
        let builder = MigrationConnection::get_database_backend(self);
        let (sql, values) = stmt.build(&builder);
        let stmt = Statement::from_sql_and_values(
            ConnectionTrait::get_database_backend(self),
            &sql,
            values,
        );
        ConnectionTrait::query_all(self, stmt).await
    }

    async fn exec_stmt<S>(&self, stmt: &S) -> Result<(), DbErr>
    where
        S: MigrationStatementBuilder + Sync,
    {
        let builder = MigrationConnection::get_database_backend(self);
        let (sql, values) = stmt.build(&builder);
        let stmt = Statement::from_sql_and_values(
            ConnectionTrait::get_database_backend(self),
            &sql,
            values,
        );
        self.execute(stmt).await.map(|_| ())
    }

    fn get_database_backend(&self) -> sea_schema::migration::MigrationDbBackend {
        match ConnectionTrait::get_database_backend(self) {
            DbBackend::MySql => MigrationDbBackend::MySql,
            DbBackend::Postgres => MigrationDbBackend::Postgres,
            DbBackend::Sqlite => MigrationDbBackend::Sqlite,
        }
    }

    fn get_connection(&self) -> &DatabaseConnection {
        self
    }
}
