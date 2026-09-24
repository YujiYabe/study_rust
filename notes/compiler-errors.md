# 重要なコンパイルエラー

## 非同期処理

- `future cannot be sent between threads safely`: `tokio::spawn`するFutureが`Send`ではない値を
  `.await`の前後で保持している可能性がある。値のscopeを短くするか、対応する同期手段を選ぶ。
- `borrow of moved value`: `async move`へ値の所有権が移った後に外側で再利用している。共有が必要なら
  `Arc`をcloneして各taskへ渡す。ただし、単なるコンパイル回避のcloneにはしない。
- `MutexGuard`を`.await`越しに保持すると、`Send`制約違反または競合の長期化につながる。lock中に
  必要な値だけ操作し、guardをdropしてから非同期処理を待つ。
