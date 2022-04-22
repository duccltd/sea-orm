use crate::{
    error::*, ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, Insert, IntoActiveModel,
    Iterable, PrimaryKeyTrait, SelectModel, SelectorRaw, Statement, TryFromU64,
};
use sea_query::{
    Alias, Expr, FromValueTuple, Iden, InsertStatement, IntoColumnRef, Query, ValueTuple,
};
use std::{future::Future, marker::PhantomData};

/// Defines a structure to perform INSERT operations in an ActiveModel
#[derive(Debug)]
pub struct Inserter<A>
where
    A: ActiveModelTrait,
{
    primary_key: Option<ValueTuple>,
    query: InsertStatement,
    model: PhantomData<A>,
}

/// The result of an INSERT operation on an ActiveModel
#[derive(Debug)]
pub struct InsertResult<A>
where
    A: ActiveModelTrait,
{
    /// The id performed when AUTOINCREMENT was performed on the PrimaryKey
    pub last_insert_id: <<<A as ActiveModelTrait>::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType,
}

impl<A> Insert<A>
where
    A: ActiveModelTrait,
{
    /// Execute an insert operation
    #[allow(unused_mut)]
    pub fn exec<'a, C>(self, db: &'a C) -> impl Future<Output = Result<InsertResult<A>, DbErr>> + '_
    where
        C: ConnectionTrait,
        A: 'a,
    {
        // so that self is dropped before entering await
        let mut query = self.query;
        if db.support_returning() && <A::Entity as EntityTrait>::PrimaryKey::iter().count() > 0 {
            let mut returning = Query::select();
            returning.columns(
                <A::Entity as EntityTrait>::PrimaryKey::iter().map(|c| c.into_column_ref()),
            );
            query.returning(returning);
        }
        Inserter::<A>::new(self.primary_key, query).exec(db)
    }

    /// Execute an insert operation and return the inserted model (use `RETURNING` syntax if database supported)
    pub fn exec_with_returning<'a, C>(
        self,
        db: &'a C,
    ) -> impl Future<Output = Result<<A::Entity as EntityTrait>::Model, DbErr>> + '_
    where
        <A::Entity as EntityTrait>::Model: IntoActiveModel<A>,
        C: ConnectionTrait,
        A: 'a,
    {
        Inserter::<A>::new(self.primary_key, self.query).exec_with_returning(db)
    }
}

impl<A> Inserter<A>
where
    A: ActiveModelTrait,
{
    /// Instantiate a new insert operation
    pub fn new(primary_key: Option<ValueTuple>, query: InsertStatement) -> Self {
        Self {
            primary_key,
            query,
            model: PhantomData,
        }
    }

    /// Execute an insert operation
    pub fn exec<'a, C>(self, db: &'a C) -> impl Future<Output = Result<InsertResult<A>, DbErr>> + '_
    where
        C: ConnectionTrait,
        A: 'a,
    {
        let builder = db.get_database_backend();
        exec_insert(self.primary_key, builder.build(&self.query), db)
    }

    /// Execute an insert operation and return the inserted model (use `RETURNING` syntax if database supported)
    pub fn exec_with_returning<'a, C>(
        self,
        db: &'a C,
    ) -> impl Future<Output = Result<<A::Entity as EntityTrait>::Model, DbErr>> + '_
    where
        <A::Entity as EntityTrait>::Model: IntoActiveModel<A>,
        C: ConnectionTrait,
        A: 'a,
    {
        exec_insert_with_returning::<A, _>(self.primary_key, self.query, db)
    }
}

async fn exec_insert<A, C>(
    primary_key: Option<ValueTuple>,
    statement: Statement,
    db: &C,
) -> Result<InsertResult<A>, DbErr>
where
    C: ConnectionTrait,
    A: ActiveModelTrait,
{
    type PrimaryKey<A> = <<A as ActiveModelTrait>::Entity as EntityTrait>::PrimaryKey;
    type ValueTypeOf<A> = <PrimaryKey<A> as PrimaryKeyTrait>::ValueType;
    let last_insert_id_opt = match db.support_returning() {
        true => {
            let cols = PrimaryKey::<A>::iter()
                .map(|col| col.to_string())
                .collect::<Vec<_>>();

            // https://github.com/SeaQL/sea-orm/pull/531/files#diff-f72f2956c0efb280b2360edac5cd5ef1cfe72dc03cb9f45a1770d96909e81218R126-R128
            let vec = db.query_all(statement).await?;
            let res = vec.get(0).unwrap();

            res.try_get_many("", cols.as_ref()).ok()
        }
        false => {
            let last_insert_id = db.execute(statement).await?.last_insert_id();
            ValueTypeOf::<A>::try_from_u64(last_insert_id).ok()
        }
    };
    let last_insert_id = match primary_key {
        Some(value_tuple) => FromValueTuple::from_value_tuple(value_tuple),
        None => match last_insert_id_opt {
            Some(last_insert_id) => last_insert_id,
            None => return Err(DbErr::Exec("Fail to unpack last_insert_id".to_owned())),
        },
    };
    Ok(InsertResult { last_insert_id })
}

async fn exec_insert_with_returning<A, C>(
    primary_key: Option<ValueTuple>,
    mut insert_statement: InsertStatement,
    db: &C,
) -> Result<<A::Entity as EntityTrait>::Model, DbErr>
where
    <A::Entity as EntityTrait>::Model: IntoActiveModel<A>,
    C: ConnectionTrait,
    A: ActiveModelTrait,
{
    let db_backend = db.get_database_backend();
    let found = match db.support_returning() {
        true => {
            let mut returning = Query::select();
            returning.exprs(<A::Entity as EntityTrait>::Column::iter().map(|c| {
                let col = Expr::col(c);
                let col_def = ColumnTrait::def(&c);
                let col_type = col_def.get_column_type();
                match col_type.get_enum_name() {
                    Some(_) => col.as_enum(Alias::new("text")),
                    None => col.into(),
                }
            }));
            insert_statement.returning(returning);
            SelectorRaw::<SelectModel<<A::Entity as EntityTrait>::Model>>::from_statement(
                db_backend.build(&insert_statement),
            )
            // https://github.com/SeaQL/sea-orm/pull/531/files#diff-f72f2956c0efb280b2360edac5cd5ef1cfe72dc03cb9f45a1770d96909e81218R180
            .all(db)
            .await?
        }
        false => {
            // https://github.com/SeaQL/sea-orm/pull/531/files#diff-f72f2956c0efb280b2360edac5cd5ef1cfe72dc03cb9f45a1770d96909e81218R189
            use crate::QuerySelect;

            let insert_res =
                exec_insert::<A, _>(primary_key, db_backend.build(&insert_statement), db).await?;
            <A::Entity as EntityTrait>::find_by_id(insert_res.last_insert_id)
                .limit(1)
                // https://github.com/SeaQL/sea-orm/pull/531/files#diff-f72f2956c0efb280b2360edac5cd5ef1cfe72dc03cb9f45a1770d96909e81218R210
                .all(db)
                .await?
        }
    };

    let found = found.get(0);

    match found {
        Some(model) => Ok(model.clone()),
        None => Err(DbErr::Exec("Failed to find inserted item".to_owned())),
    }
}
