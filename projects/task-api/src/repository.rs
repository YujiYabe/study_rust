use diesel::{
    Connection, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper,
    r2d2::{ConnectionManager, Pool},
    sqlite::SqliteConnection,
};

use crate::{
    domain::Task,
    schema::tasks::{self, dsl},
};

// r2d2は複数のDB接続をプールで管理する。ハンドラーごとに新規接続するより効率がよい。
type SqlitePool = Pool<ConnectionManager<SqliteConnection>>;

// Cloneの実装を自動生成する。cloneしてもDB接続そのものを複製するのではなく、
// プールへの参照カウントが増えるだけなので、各リクエストへ低コストで渡せる。
#[derive(Clone)]
pub struct TaskRepository {
    pool: SqlitePool,
}

// thiserrorは各列挙値のDisplayとstd::error::Errorを自動実装する。
// `#[from]`により、`?`を使ったとき元のエラーからこの型へ自動変換できる。
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("database pool error: {0}")]
    Pool(#[from] diesel::r2d2::PoolError),
    #[error("database query error: {0}")]
    Diesel(#[from] diesel::result::Error),
}

impl TaskRepository {
    /// SQLiteの接続プールを作り、API受付前にテーブルを準備する。
    pub fn connect(database_url: &str) -> Result<Self, RepositoryError> {
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        // `:memory:`は接続ごとに別DBになるので、テスト時だけ接続数を1本にする。
        let max_size = if database_url == ":memory:" { 1 } else { 5 };
        let pool = Pool::builder().max_size(max_size).build(manager)?;
        // Selfは現在のimpl対象、つまりTaskRepositoryの別名。
        let repository = Self { pool };
        repository.migrate()?;
        Ok(repository)
    }

    fn migrate(&self) -> Result<(), RepositoryError> {
        // `?`は失敗時にRepositoryErrorへ変換し、この関数から早期リターンする。
        let mut connection = self.pool.get()?;
        diesel::sql_query(
            r#"CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                description TEXT,
                completed BOOLEAN NOT NULL DEFAULT FALSE
            )"#,
        )
        .execute(&mut connection)?;
        // 成功したが返すデータはないことをユニット型`()`で表す。
        Ok(())
    }

    pub fn list(&self, completed: Option<bool>) -> Result<Vec<Task>, RepositoryError> {
        let mut connection = self.pool.get()?;
        // `into_boxed`により、条件に応じて後からWHERE句を追加できるクエリにする。
        let mut query = dsl::tasks.into_boxed();
        if let Some(completed) = completed {
            query = query.filter(dsl::completed.eq(completed));
        }
        Ok(query
            .order(dsl::id.asc())
            .select(Task::as_select())
            .load(&mut connection)?)
    }

    pub fn find(&self, id: i64) -> Result<Option<Task>, RepositoryError> {
        let mut connection = self.pool.get()?;
        // optional()はNotFoundエラーをNoneへ変え、その他のDBエラーはErrのまま返す。
        Ok(dsl::tasks
            .find(id)
            .select(Task::as_select())
            .first(&mut connection)
            .optional()?)
    }

    pub fn create(&self, title: &str, description: Option<&str>) -> Result<Task, RepositoryError> {
        let mut connection = self.pool.get()?;
        // INSERTと再取得を同じトランザクションにまとめ、途中失敗時は自動でロールバックする。
        Ok(connection.transaction(|connection| {
            diesel::insert_into(tasks::table)
                .values((
                    dsl::title.eq(title),
                    dsl::description.eq(description),
                    dsl::completed.eq(false),
                ))
                .execute(connection)?;
            dsl::tasks
                .order(dsl::id.desc())
                .select(Task::as_select())
                .first(connection)
        })?)
    }

    pub fn update(
        &self,
        id: i64,
        title: &str,
        description: Option<&str>,
        completed: bool,
    ) -> Result<Option<Task>, RepositoryError> {
        let mut connection = self.pool.get()?;
        let target = dsl::tasks.find(id);
        let changed = diesel::update(target)
            .set((
                dsl::title.eq(title),
                dsl::description.eq(description),
                dsl::completed.eq(completed),
            ))
            .execute(&mut connection)?;
        if changed == 0 {
            return Ok(None);
        }
        Ok(dsl::tasks
            .find(id)
            .select(Task::as_select())
            .first(&mut connection)
            .optional()?)
    }

    pub fn delete(&self, id: i64) -> Result<bool, RepositoryError> {
        let mut connection = self.pool.get()?;
        let deleted = diesel::delete(dsl::tasks.find(id)).execute(&mut connection)?;
        Ok(deleted > 0)
    }
}
