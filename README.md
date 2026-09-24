# study_rust

Go経験者がRustらしい設計と実装を身につけるための学習用リポジトリです。

学習内容と演習課題は[カリキュラム](./CURRICULUM.md)を参照してください。

## コミット前の品質チェック

このリポジトリでは`pre-commit`を使い、コミット前にフォーマット、型検査、Clippy、テストを
自動実行します。`pipx`を用意したうえで、初回セットアップを実行してください。

```console
make setup
```

このコマンドは`rustfmt`とClippyの導入、pre-commitの導入、`.githooks`内のGit hook登録、
Cargo依存crateの取得を行います。利用可能なMakeコマンドは`make help`で確認できます。

以降は`git commit`時に次の検査が実行されます。

```console
cargo fmt --all --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

コミットせずに全検査を実行する場合は次のコマンドを使います。

```console
pre-commit run --all-files
```

Makefileから同じ検査をまとめて実行することもできます。

```console
make verify
```

Task APIの起動には次のコマンドを使用します。

```console
make run
```

待受先やSQLiteファイルを変更する場合は、Make変数を指定します。

```console
make run BIND_ADDRESS=0.0.0.0:8080 DATABASE_URL=my-tasks.db
```
