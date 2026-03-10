# コードレビュー結果

**実施日**: 2026-03-09  
**対象**: MacWhisper Neo MVP 実装（マルチプロバイダ対応・YouTube文字起こし・カスタムモデル追加まで）

---

## 1. 指摘事項と修正状況

### P0: Cloud系プロバイダが実質認証不能（APIキー未配線） — **修正済**

| 項目 | 内容 |
|------|------|
| **Issue** | `run_transcription` は `request.api_key` を受け取るが、Registry 側のプロバイダは起動時に空キーで生成されたまま。UI でキーを入力しても API 呼び出しに反映されず、401/403 が発生する |
| **Fix** | `transcribe.rs` で provider ごとに実行時に動的生成。`anthropic` / `openai_whisper` / `google_gemini` は `request.api_key` を必須化して新規インスタンス生成。`lm_studio` は endpoint のみ、`whisper_cpp` 等は registry から取得 |
| **Test** | 各 Provider を選択し、対応する API キーを入力して文字起こしが成功することを確認 |

---

### P1: OpenAI でモデル選択が無効化されている — **修正済**

| 項目 | 内容 |
|------|------|
| **Issue** | `openai_whisper.rs` の `transcribe` が `_model_id` を無視し、常に `whisper-1` を送信。UI で `gpt-4o-transcribe` / `mini` を選んでも反映されない |
| **Fix** | `_model_id` を `model_id` に戻し、フォームの `model` にそのまま設定 |
| **Test** | OpenAI を選択し、GPT-4o Transcribe / Mini を切り替えて文字起こし。レスポンス形式の差異があれば対応 |

---

### P1: 同梱 yt-dlp の本番パス解決が壊れる可能性 — **修正済**

| 項目 | 内容 |
|------|------|
| **Issue** | `bundle.resources` は `resources/yt-dlp` を指定するが、実行時に `resource_dir/resources/yt-dlp` を見に行っている。Tauri の bundle 配置差異により本番で見つからないリスク |
| **Fix** | `youtube.rs` の `resolve_ytdlp_path` で複数候補パスを探索（`resource_dir/yt-dlp`、`resource_dir/resources/yt-dlp`、`resource_dir/../Resources/yt-dlp`）。開発環境フォールバックは維持 |
| **Test** | `cargo tauri build` で .dmg を生成し、同梱 yt-dlp で YouTube ダウンロードが動作することを確認 |

---

### P2: YouTube 一時ファイルのクリーンアップ未実装 — **修正済**

| 項目 | 内容 |
|------|------|
| **Issue** | `/tmp/macwhisper-neo/*.mp3` を作成するが削除処理がなく、長期利用でディスク肥大化 |
| **Fix** | `cleanup_youtube_temp_file` コマンドを追加。`transcribeYoutube` の `finally` で文字起こし完了後に削除を呼び出す。パス検証（許可ディレクトリ外は拒否）を実装 |
| **Test** | YouTube URL で文字起こし後、`/tmp/macwhisper-neo` に該当 mp3 が残っていないことを確認 |

---

### P2: DB ファイルの派生物が Git 管理対象になっている — **修正済**

| 項目 | 内容 |
|------|------|
| **Issue** | `data/app.db-wal`、`data/app.db-shm` が変更対象に含まれる。`.gitignore` は `data/*.db` のみで漏れている |
| **Fix** | `.gitignore` に `data/*.db-wal`、`data/*.db-shm` を追加 |
| **Test** | `git status` で db 関連ファイルが表示されないことを確認 |

---

## 2. オープンクエスチョン

- **API キー保存**: 将来的に平文 SQLite ではなく macOS Keychain 保存に切り替えるか？（運用リスクを下げられる）

---

## 3. 総括

機能追加の方向性は妥当。**認証キー未配線（P0）** と **OpenAI モデル固定（P1）** を優先して修正済み。上記 5 件の指摘はすべて対応済み。

---

## 4. レビュー対象ファイル一覧（当時）

| カテゴリ | ファイル |
|----------|----------|
| バックエンド | `src-tauri/src/commands/transcribe.rs`, `providers.rs`, `youtube.rs`, `models.rs`, `settings.rs` |
| エンジン | `src-tauri/src/engine/anthropic.rs`, `openai_whisper.rs`, `gemini.rs`, `lm_studio.rs`, `mod.rs` |
| DB | `src-tauri/src/db/mod.rs`, `schema.rs` |
| フロントエンド | `app/src/stores/transcriptStore.ts`, `hooks/useTranscription.ts`, `components/HistoryPanel.tsx`, `DropZone.tsx`, `AddModelModal.tsx` |
| 設定 | `src-tauri/tauri.conf.json`, `.gitignore` |
