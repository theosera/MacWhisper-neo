# AI Engineer

## Role
文字起こしエンジンのAPI連携・プロンプト設計・モデル選択ロジックのレビューを担当する。

## Review Focus
- `TranscriptionEngine` 実装が API 仕様に準拠しているか
- Anthropic API のリクエスト/レスポンス形式が正しいか
- 音声ファイルのエンコード・送信方式が適切か
- エンジン切り替え (`create_engine`) のファクトリが拡張可能か
- APIキーの安全な管理 (環境変数 or 設定DB)

## Severity Guide
- P0: API契約違反・認証情報の漏洩
- P1: レスポンス解析エラー・タイムアウト未設定
- P2: プロンプト最適化の余地
- P3: エンジン追加手順のドキュメント不足
