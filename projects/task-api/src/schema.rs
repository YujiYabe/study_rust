// Dieselの`table!`マクロは、SQLテーブルをRustの型として表現するコードを生成する。
// そのため、存在しない列や型の合わないクエリの多くをコンパイル時に検出できる。
diesel::table! {
    tasks (id) {
        id -> BigInt,
        title -> Text,
        description -> Nullable<Text>,
        completed -> Bool,
    }
}
