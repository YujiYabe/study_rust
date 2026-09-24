use std::{env, io, time::Duration};

use actix_web::{
    App, HttpServer,
    dev::Service,
    http::{header::HeaderName, header::HeaderValue},
    middleware::Logger,
    web::Data,
};
use task_api::{TaskRepository, configure, create_schema};
use uuid::Uuid;

// `#[actix_web::main]`は非同期なmainを実行するActixランタイムの起動コードを生成する。
#[actix_web::main]
async fn main() -> io::Result<()> {
    // RUST_LOG未指定時にもリクエストログが見えるよう、既定のフィルターを設定する。
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Dieselへ渡すSQLite URLはファイルパス。`:memory:`ならメモリ上に作成される。
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "tasks.db".into());
    let address = env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1:3000".into());

    // DB初期化のエラーを、mainの戻り値であるio::Errorへ変換する。
    let repository = TaskRepository::connect(&database_url)
        .map_err(|error| io::Error::other(error.to_string()))?;
    // スキーマにもDataを使い、全ワーカーから同じ型情報とリポジトリを共有する。
    let schema = Data::new(create_schema(repository));
    log::info!("task API listening on {address}");

    HttpServer::new(move || {
        // moveクロージャーがDataを所有する。各ワーカーにはcloneした参照を渡す。
        App::new()
            .app_data(schema.clone())
            .wrap(Logger::default())
            .wrap_fn(|mut request, service| {
                // ミドルウェアでリクエストIDを発行し、リクエストとレスポンスの両方へ設定する。
                let request_id = HeaderValue::from_str(&Uuid::new_v4().to_string())
                    .expect("UUID is always a valid HTTP header value");
                request
                    .headers_mut()
                    .insert(HeaderName::from_static("x-request-id"), request_id.clone());
                let future = service.call(request);
                async move {
                    let mut response = future.await?;
                    response
                        .headers_mut()
                        .insert(HeaderName::from_static("x-request-id"), request_id);
                    Ok(response)
                }
            })
            .configure(configure)
    })
    .bind(&address)?
    // Ctrl+C受信時は新規受付を止め、最大10秒まで処理中リクエストの完了を待つ。
    .shutdown_timeout(Duration::from_secs(10).as_secs())
    .run()
    .await
}
