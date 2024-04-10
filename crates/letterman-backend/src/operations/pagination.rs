use diesel::{
    mysql::Mysql,
    query_builder::{Query, QueryFragment, QueryId},
    query_dsl::methods::LoadQuery,
    sql_types::Integer,
    MysqlConnection, QueryResult, RunQueryDsl,
};

const DEFAULT_PAGE_SIZE: i32 = 10;

pub trait Paginate: Sized {
    fn paginate(self, page: i32) -> Paginated<Self>;
}

impl<T> Paginate for T {
    fn paginate(self, page: i32) -> Paginated<Self> {
        Paginated {
            query: self,
            page,
            page_size: DEFAULT_PAGE_SIZE,
            offset: (page - 1) * DEFAULT_PAGE_SIZE,
        }
    }
}

#[derive(Debug, Clone, Copy, QueryId)]
pub struct Paginated<T> {
    query: T,
    page: i32,
    page_size: i32,
    offset: i32,
}

impl<T> Paginated<T> {
    pub fn page_size(self, page_size: i32) -> Self {
        Paginated {
            page_size,
            offset: (self.page - 1) * page_size,
            ..self
        }
    }

    pub fn load_and_count_pages<'a, U>(
        self,
        conn: &mut MysqlConnection,
    ) -> QueryResult<(Vec<U>, i32)>
    where
        Self: LoadQuery<'a, MysqlConnection, (U, i32)>,
    {
        let results = self.load::<(U, i32)>(conn)?;
        let total = results.first().map(|x| x.1).unwrap_or(0);
        let records = results.into_iter().map(|x| x.0).collect();
        Ok((records, total))
    }
}

impl<T> QueryFragment<Mysql> for Paginated<T>
where
    T: QueryFragment<Mysql>,
{
    fn walk_ast<'b>(
        &'b self,
        mut pass: diesel::query_builder::AstPass<'_, 'b, Mysql>,
    ) -> diesel::prelude::QueryResult<()> {
        pass.push_sql("SELECT *, COUNT(*) OVER () FROM(");

        self.query.walk_ast(pass.reborrow())?;
        pass.push_sql(") as paged_query_with LIMIT ");
        pass.push_bind_param::<Integer, _>(&self.page_size)?;
        pass.push_sql(" OFFSET ");
        pass.push_bind_param::<Integer, _>(&self.offset)?;
        Ok(())
    }
}

impl<T: Query> Query for Paginated<T> {
    type SqlType = (T::SqlType, diesel::sql_types::Integer);
}

impl<T> RunQueryDsl<MysqlConnection> for Paginated<T> {}
