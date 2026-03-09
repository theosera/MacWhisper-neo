# Multi-Provider Design (Obsidian Smart Composer Style)

## 目標

MacWhisper Neo を複数の文字起こしエンジンに対応させる。
Obsidian の Smart Composer と同様に、ユーザーが実行時にプロバイダとモデルを選択可能にする。

## アーキテクチャ

### 1. TranscriptionProvider トレイト（既存を拡張）

```rust
pub trait TranscriptionProvider: Send + Sync {
    fn provider_id(&self) -> &str;
    fn provider_name(&self) -> &str;
    fn available_models(&self) -> Vec<ModelInfo>;
    async fn transcribe(
        &self, 
        audio_path: &str, 
        model_id: &str,
        language: Option<&str>
    ) -> Result<TranscriptionResult, AppError>;
}

pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub max_file_size_mb: u32,
}
```

### 2. Provider 実装一覧（初期）

#### API-based
- **Anthropic API** (現在)
  - `claude-sonnet-4-20250514`
  - `claude-opus-4-1` (後日)

- **OpenAI Whisper API** (新規)
  - `whisper-1`

- **Google Cloud Speech-to-Text** (後日)
- **AssemblyAI** (後日)

#### Local
- **LM Studio** (新規、ローカルLLM経由)
  - 接続先: `http://localhost:1234/v1/audio/transcriptions` (OpenAI互換)
  
- **whisper.cpp** (既存スタブ、後日)
  - ローカルバイナリ実行

### 3. ProviderRegistry（Obsidian型）

```rust
pub struct ProviderRegistry {
    providers: HashMap<String, Box<dyn TranscriptionProvider>>,
}

impl ProviderRegistry {
    pub fn register(&mut self, provider: Box<dyn TranscriptionProvider>) { }
    pub fn get(&self, provider_id: &str) -> Option<&dyn TranscriptionProvider> { }
    pub fn list_providers(&self) -> Vec<ProviderInfo> { }
}

pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub category: ProviderCategory,
}

pub enum ProviderCategory {
    ApiCloud,     // API経由（クラウド）
    ApiLocal,     // API互換（ローカル）
    LocalBinary,  // ローカルバイナリ
}
```

### 4. UI フロー（設定パネル）

```
HistoryPanel
  ├─ API Key入力
  ├─ Provider選択ドロップダウン
  │   ├─ Anthropic API
  │   ├─ OpenAI Whisper API
  │   ├─ Google Cloud Speech
  │   └─ LM Studio (local)
  │
  ├─ Model選択ドロップダウン
  │   └─ (選択されたProviderの available_models)
  │
  ├─ Language選択
  └─ [Settings] ボタン
      └─ API Key / Endpoint 設定画面
```

### 5. DB スキーマ拡張

```sql
-- settings テーブルにキーを追加
INSERT INTO settings (key, value) VALUES
  ('transcription_provider_id', 'anthropic'),
  ('transcription_model_id', 'claude-sonnet-4-20250514'),
  ('anthropic_api_key', '...'),
  ('openai_api_key', '...'),
  ('lm_studio_endpoint', 'http://localhost:1234'),
  ('language', 'auto');
```

### 6. 実装順序（優先度）

**Phase 1 (MVP拡張)**
1. ProviderRegistry パターン実装
2. OpenAI Whisper API 実装（テスト用途で有用）
3. Settings パネル UI

**Phase 2 (ローカル対応)**
4. LM Studio 統合
5. whisper.cpp スタブ → 実装

**Phase 3 (クラウド拡張)**
6. Google Cloud Speech-to-Text
7. AssemblyAI

---

## 実装の流れ

```
1. TranscriptionProvider トレイト拡張
2. ProviderRegistry の実装
3. 既存 AnthropicProvider を新トレイトに適合
4. OpenAI Whisper Provider の実装
5. API キー管理用 Settings パネル の UI
6. Provider/Model 選択 UI
7. LM Studio Provider (後日)
```

---

## Obsidian Smart Composer との類似性

| 項目 | Obsidian | MacWhisper Neo |
|---|---|---|
| プロバイダ選択 | LLM Provider dropdown | Transcription Provider dropdown |
| モデル選択 | Model dropdown (per provider) | Model dropdown (per provider) |
| API Key管理 | Settings modal | Settings panel |
| ローカル対応 | Ollama integration | LM Studio integration |
| 実行時切り替え | ドロップダウン | ドロップダウン |
