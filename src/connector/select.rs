use crate::{Connection, Database, Entity, Model, QueryErr, Select};

impl<E: 'static> Select<'_, E>
where
    E: Entity,
{
    pub async fn one(self, db: &Database) -> Result<E::Model, QueryErr> {
        let builder = db.get_query_builder_backend();
        let row = db.get_connection().query_one(self.build(builder)).await?;
        Ok(<E as Entity>::Model::from_query_result(row)?)
    }

    pub async fn all(self, db: &Database) -> Result<Vec<E::Model>, QueryErr> {
        let builder = db.get_query_builder_backend();
        let rows = db.get_connection().query_all(self.build(builder)).await?;
        let mut models = Vec::new();
        for row in rows.into_iter() {
            models.push(<E as Entity>::Model::from_query_result(row)?);
        }
        Ok(models)
    }
}