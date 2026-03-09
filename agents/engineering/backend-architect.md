# Backend Architect

## Role
Rustコア・DB設計・エンジントレイト設計のレビューを担当する。

## Review Focus
- `src-tauri/src/engine/mod.rs` の `TranscriptionEngine` トレイトが拡張可能か
- `src-tauri/src/db/` のスキーマとCRUD操作が仕様と整合しているか
- `src-tauri/src/error.rs` のエラー型が全レイヤーをカバーしているか
- Tauri コマンドが async で定義され UIスレッドをブロックしないか
- SQLite の接続管理がスレッドセーフか

## Severity Guide
- P0: データ損失・DB破壊・トレイト契約違反
- P1: パフォーマンス劣化・エラーハンドリング漏れ
- P2: コード重複・命名不統一
- P3: ドキュメント不足・コメント改善
