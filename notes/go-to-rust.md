# GoからRustへ

## 非同期処理とWeb API

- goroutineはGoランタイムが実行する軽量スレッドだが、Rustの`Future`はpollされるまで進まない。
  Tokioのexecutorがreadyになるたびにpollし、`.await`中は他のtaskへ実行を譲る。
- `async`はCPU処理を高速化する機能ではなく、主にネットワークやDBなどI/O待ちの間に別の仕事を
  進めるための仕組み。CPU負荷の高い処理は`spawn_blocking`やRayonへ分離する。
- async-graphqlの`#[Object]`と`SimpleObject`はresolverと返却型からGraphQL Schemaを生成する。
  Actix Webは`/graphql`へのHTTP通信とSchemaの共有を担当する。
- DieselのSQLite APIは同期処理なので、非同期handlerから直接呼ばず`web::block`でblocking専用threadへ
  分離する。接続はr2d2のpoolへ任せ、domainの値をHTTPの型から分離する。

```text
request -> Tokio runtime -> Future::poll
                              | Ready(value) -> response
                              ` Pending      -> I/O readiness後に再poll
```
