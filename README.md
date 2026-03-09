# MacWhisper Neo

macOS向け音声文字起こしデスクトップアプリ MVP。  
Tauri 2 + React 19 + Rust で構築。

## Quick Start

```bash
# 1. フロントエンド依存インストール
cd app && npm install

# 2. ターミナル1: Vite dev server 起動
cd app && npm run dev

# 3. ターミナル2: Tauri アプリ起動
cd src-tauri && cargo run
```

## 使い方

1. 左パネルの API Key に Anthropic API キーを入力
2. 音声/動画ファイルを中央の DropZone にドラッグ&ドロップ
3. 文字起こし結果がセグメント単位で表示される
4. Export ボタンで Markdown / TXT / JSON に保存

## 対応フォーマット

**Audio**: mp3, wav, m4a, aac, ogg, flac  
**Video**: mp4, mov, mkv, webm, avi

## Tech Stack

- **Frontend**: React 19, Vite, TypeScript, Zustand
- **Backend**: Tauri 2, Rust
- **Database**: SQLite (rusqlite)
- **STT Engine**: Anthropic API (MVP)

## プロジェクト構成

```
├── app/           React フロントエンド
├── src-tauri/     Rust バックエンド (Tauri 2)
├── agents/        AI コードレビュー役割定義
├── data/          ローカルDB
└── docs/          アーキテクチャドキュメント
```

## License

MIT
