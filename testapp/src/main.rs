use std::env;

fn main() {
    // 整数の範囲を確認
    println!("u8 : {} ～ {}", u8::MIN, u8::MAX);
    println!("i8 : {} ～ {}", i8::MIN, i8::MAX);
    println!("i32: {} ～ {}", i32::MIN, i32::MAX);

    // 引数を使い、コンパイル時ではなく実行時に計算させる
    let value: u8 = env::args()
        .nth(1)
        .expect("整数を指定してください")
        .parse()
        .expect("u8の整数を指定してください");

    println!("{} + 1 = {}", value, value + 1);
}
