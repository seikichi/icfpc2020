# Galaxy PAD

1. 親プロジクトで `cargo build`
2. 親プロジェクトの `Cargo.toml` の依存関コメントアウト (残っていると wasm へのビルド失敗)
3. pad プロジェクトで `wasm-pack build`
4. www プロジェクトで `npm install`
5. www プロジェクトで `npm start`
