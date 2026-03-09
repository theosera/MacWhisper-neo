# CLAUDE.md

## Purpose
MacWhisperライクなmacOS音声文字起こしデスクトップアプリのMVP。
音声/動画ファイルをドラッグ&ドロップで読み込み、文字起こしエンジンでセグメント付きテキストを生成し、ローカルDBに保存する。

## Source Of Truth
- 仕様と実装の整合は本ファイルを最優先で確認する
- 役割分担の詳細は `agents/` 配下を参照する
- アーキテクチャ仕様は `docs/architecture.md` を参照する

## Tech Stack
- **Frontend**: React 19 + Vite + TypeScript + Zustand
- **Backend**: Tauri 2 + Rust
- **DB**: SQLite (rusqlite bundled)
- **STT Engine**: Anthropic API (MVP) → whisper.cpp (後日追加)
- **HTTP**: reqwest (rustls-tls)

## Baseline Structure
- `app/` — UI (React + Vite)
- `src-tauri/` — Tauri commands + Rust core
- `src-tauri/src/engine/` — 差し替え可能な文字起こしエンジン
- `data/` — ローカルDB・出力データ
- `agents/` — AIコードレビュー用役割定義
- `docs/` — アーキテクチャドキュメント

## Key Design Decisions
- `TranscriptionEngine` トレイトでエンジンを抽象化。`engine_id` で切り替え可能
- Tauri コマンドは全て `async` でUIスレッドをブロックしない
- `AppError` 統一エラー型で全レイヤーのエラーを一元管理
- セグメント (start_ms, end_ms, text) 単位でDBに保存・表示
- エクスポートはフロントエンド側で Markdown / TXT / JSON を生成

## Agent Roles
- `agents/engineering/backend-architect.md`
- `agents/engineering/frontend-developer.md`
- `agents/engineering/ai-engineer.md`
- `agents/testing/api-tester.md`
- `agents/testing/test-results-analyzer.md`
- `agents/project-management/sprint-prioritizer.md`

## Review Workflow
1. `backend-architect` が設計逸脱・DB整合性を確認
2. `frontend-developer` がUX/API接続の齟齬を確認
3. `ai-engineer` がエンジン実装とAPI契約を確認
4. `api-tester` がコマンドI/O契約・エラーハンドリングを確認
5. `test-results-analyzer` がテストカバレッジと結果を確認
6. `sprint-prioritizer` が優先順位と技術的負債を確認

## Review Output Format
レビュー結果は必ず以下形式で記録する。
- `Severity`: `P0` / `P1` / `P2` / `P3`
- `File`: リポジトリ相対パス
- `Issue`: 何が壊れるか
- `Fix`: 最小修正案
- `Test`: 再発防止のテスト観点

## Definition Of Done (MVP)
- 音声/動画ファイルをD&Dで読み込み、文字起こし結果を表示できる
- セグメント単位で開始時刻・終了時刻・本文が表示される
- 結果を Markdown / TXT / JSON で保存できる
- 履歴がSQLiteへ保存・再表示できる
- 3ペインUI (履歴/本文/メタ情報) が動作する
- エンジンが差し替え可能な設計になっている
- 主要コマンドに最低1つの正常系テスト観点がある
