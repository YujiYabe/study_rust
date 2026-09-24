# Task API（第8週）

Actix Web、async-graphql、Diesel、SQLiteを使ったGraphQLタスク管理APIです。Dieselの同期DB処理は
`web::block`でblocking専用threadへ移し、HTTP workerを停止させない構成です。

## 起動

リポジトリのルートで次を実行します。

```console
cargo run -p task-api
```

既定では `127.0.0.1:3000` で待ち受け、`tasks.db` に保存します。環境変数
`BIND_ADDRESS` と `DATABASE_URL` で変更できます。

```console
DATABASE_URL='my-tasks.db' BIND_ADDRESS='0.0.0.0:8080' cargo run -p task-api
```

## API

| Method | Path | 内容 | 成功時 |
| --- | --- | --- | --- |
| `GET` | `/health` | ヘルスチェック | `204` |
| `GET` | `/graphql` | GraphQL Playground | `200` |
| `POST` | `/graphql` | Query／Mutation | `200` |

ブラウザで `http://127.0.0.1:3000/graphql` を開くとPlaygroundから操作できます。
VS CodeのREST Client拡張を使う場合は、[`requests.http`](./requests.http)を開き、
各リクエストの「Send Request」を上から順に実行してください。作成したタスクのIDは、
後続の取得・更新・削除リクエストへ自動で引き継がれます。

### Query

```console
curl -s http://127.0.0.1:3000/graphql \
  -H 'content-type: application/json' \
  -d '{"query":"{ tasks(completed: false) { id title description completed } }"}'
```

### Mutation

```console
curl -s http://127.0.0.1:3000/graphql \
  -H 'content-type: application/json' \
  -d '{"query":"mutation { createTask(input: { title: \"async-graphqlを学ぶ\", description: \"第8週\" }) { id title completed } }"}'
```

GraphQLの操作は`tasks`、`task`、`createTask`、`updateTask`、`deleteTask`です。
エラーはGraphQL標準の`errors`配列で返し、`extensions.code`で
`VALIDATION_ERROR`、`NOT_FOUND`、`INTERNAL_ERROR`を判別できます。全レスポンスには
`x-request-id` が付き、`RUST_LOG` でログレベルを変更できます。Ctrl+Cを受けると新規受付を
止め、処理中のリクエストを待って終了します。

## 確認

```console
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
