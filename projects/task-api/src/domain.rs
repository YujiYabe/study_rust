use async_graphql::{InputObject, SimpleObject};

/// HTTP層やSQLite固有の型を持たない、アプリケーションのタスク表現。
///
/// `derive`は、指定したトレイト（型に共通の振る舞い）をコンパイラに自動実装させる属性。
/// ここでは次の機能が生成される。
///
/// - `Debug`: `{:?}`を使ってデバッグ表示できる
/// - `Clone`: 所有権を移動せず、明示的に値を複製できる
/// - `PartialEq` / `Eq`: テストなどで二つの`Task`を比較できる
/// - `SimpleObject`: 各フィールドをGraphQLから取得できるオブジェクト型にする
/// - `Queryable`: DieselがSQLの検索結果をフィールドの順番に従って`Task`へ変換できる
/// - `Selectable`: DieselのSELECT句を型安全に組み立てられる
#[derive(Debug, Clone, PartialEq, Eq, SimpleObject, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::schema::tasks)]
pub struct Task {
    // SQLiteのINTEGERと対応する符号付き64 bit整数。
    pub id: i64,
    // Stringは文字列データを所有する。Taskが有効な間はtitleも有効になる。
    pub title: String,
    // Option<T>は値がある`Some(T)`と、ない`None`を型で表す。SQLiteではNULLに対応する。
    pub description: Option<String>,
    pub completed: bool,
}

/// GraphQLの作成ミューテーションが受け取る入力型。
/// `InputObject`はGraphQL入力をこの構造体へ変換するコードとスキーマ定義を生成する。
#[derive(Debug, InputObject)]
pub struct CreateTask {
    pub title: String,
    pub description: Option<String>,
}

/// 更新はタスク全体を置き換えるため、`completed`も必須にしている。
#[derive(Debug, InputObject)]
pub struct UpdateTask {
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
}

/// 保存前にタイトルを正規化し、ドメイン上の制約を検査する。
/// 借用した`&str`を返すため、検査だけのために文字列を複製しない。
pub fn validate_title(title: &str) -> Result<&str, ValidationError> {
    // `trim`は新しいStringを作らず、元の文字列の一部分を`&str`として借用する。
    let title = title.trim();
    if title.is_empty() {
        return Err(ValidationError::EmptyTitle);
    }
    if title.chars().count() > 100 {
        return Err(ValidationError::TitleTooLong);
    }
    // Resultは成功をOk(T)、失敗をErr(E)で表す。呼び出し側は両方を処理する必要がある。
    Ok(title)
}

/// `thiserror::Error`は`std::error::Error`と表示用の`Display`を自動実装する。
/// 各`#[error(...)]`の文字列が、`error.to_string()`の結果になる。
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("title must not be empty")]
    EmptyTitle,
    #[error("title must be 100 characters or fewer")]
    TitleTooLong,
}
