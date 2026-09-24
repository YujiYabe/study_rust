use actix_web::{HttpResponse, web};
use async_graphql::{
    Context, EmptySubscription, Error, ErrorExtensions, Object, Result, Schema,
    http::{GraphQLPlaygroundConfig, playground_source},
};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};

use crate::{
    domain::{CreateTask, Task, UpdateTask, validate_title},
    repository::{RepositoryError, TaskRepository},
};

/// スキーマはGraphQL API全体の型情報とリゾルバー（データ取得処理）を保持する。
/// QueryRootは読み取り、MutationRootは更新、EmptySubscriptionは購読機能なしを表す。
pub type ApiSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

/// リポジトリをGraphQLのコンテキストへ登録してスキーマを組み立てる。
pub fn create_schema(repository: TaskRepository) -> ApiSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(repository)
        .finish()
}

/// Actix WebへGraphQLエンドポイントと開発用Playgroundを登録する。
pub fn configure(config: &mut web::ServiceConfig) {
    config
        .route("/graphql", web::post().to(graphql))
        .route("/graphql", web::get().to(playground))
        .route("/health", web::get().to(health));
}

async fn graphql(schema: web::Data<ApiSchema>, request: GraphQLRequest) -> GraphQLResponse {
    // GraphQLRequestをasync-graphql本体へ渡し、結果をActix用レスポンスへ変換する。
    schema.execute(request.into_inner()).await.into()
}

async fn playground() -> HttpResponse {
    // ブラウザでクエリやミューテーションを試せるHTML画面を返す。本番では無効化も検討する。
    let html = playground_source(GraphQLPlaygroundConfig::new("/graphql"));
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

async fn health() -> HttpResponse {
    HttpResponse::NoContent().finish()
}

/// 値を読み取るGraphQLリゾルバーをまとめるルートオブジェクト。
pub struct QueryRoot;

// `#[Object]`マクロはimpl内の非同期メソッドをGraphQLフィールドとしてスキーマへ公開する。
#[Object]
impl QueryRoot {
    /// 全タスクを返す。`completed`を指定すると完了状態で絞り込む。
    async fn tasks(&self, context: &Context<'_>, completed: Option<bool>) -> Result<Vec<Task>> {
        // コンテキストはリクエストごとの情報やスキーマ作成時に登録した共有データを保持する。
        let repository = context.data_unchecked::<TaskRepository>().clone();
        // Dieselは同期APIなので、ブロッキング処理専用スレッドで実行する。
        web::block(move || repository.list(completed))
            .await
            .map_err(blocking_error)?
            .map_err(repository_error)
    }

    /// IDに対応するタスクを返す。存在しない場合はGraphQLのnullになる。
    async fn task(&self, context: &Context<'_>, id: i64) -> Result<Option<Task>> {
        let repository = context.data_unchecked::<TaskRepository>().clone();
        web::block(move || repository.find(id))
            .await
            .map_err(blocking_error)?
            .map_err(repository_error)
    }
}

/// データを変更するGraphQLリゾルバーをまとめるルートオブジェクト。
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_task(&self, context: &Context<'_>, input: CreateTask) -> Result<Task> {
        // 入力検証の失敗もHTTP 200内のGraphQLエラーとして返し、code拡張で種類を示す。
        let title = validate_title(&input.title)
            .map_err(|error| graphql_error(error.to_string(), "VALIDATION_ERROR"))?
            .to_owned();
        let description = input.description;
        let repository = context.data_unchecked::<TaskRepository>().clone();
        web::block(move || repository.create(&title, description.as_deref()))
            .await
            .map_err(blocking_error)?
            .map_err(repository_error)
    }

    async fn update_task(&self, context: &Context<'_>, id: i64, input: UpdateTask) -> Result<Task> {
        let title = validate_title(&input.title)
            .map_err(|error| graphql_error(error.to_string(), "VALIDATION_ERROR"))?
            .to_owned();
        let repository = context.data_unchecked::<TaskRepository>().clone();
        let result = web::block(move || {
            repository.update(id, &title, input.description.as_deref(), input.completed)
        })
        .await
        .map_err(blocking_error)?
        .map_err(repository_error)?;
        result.ok_or_else(|| graphql_error("task not found", "NOT_FOUND"))
    }

    /// 削除できた場合はtrue、対象が存在しない場合はfalseを返す。
    async fn delete_task(&self, context: &Context<'_>, id: i64) -> Result<bool> {
        let repository = context.data_unchecked::<TaskRepository>().clone();
        web::block(move || repository.delete(id))
            .await
            .map_err(blocking_error)?
            .map_err(repository_error)
    }
}

fn repository_error(error: RepositoryError) -> Error {
    // 詳細はサーバーログへ残し、SQLや接続情報をクライアントには公開しない。
    log::error!("database operation failed: {error}");
    graphql_error("an internal error occurred", "INTERNAL_ERROR")
}

fn blocking_error(error: actix_web::error::BlockingError) -> Error {
    log::error!("blocking task failed: {error}");
    graphql_error("an internal error occurred", "INTERNAL_ERROR")
}

fn graphql_error(message: impl Into<String>, code: &'static str) -> Error {
    // extensionsはGraphQL標準のエラーにアプリ固有情報を追加する場所。
    Error::new(message).extend_with(|_, extensions| extensions.set("code", code))
}
