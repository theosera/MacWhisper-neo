import { useState } from "react";
import { addCustomModel } from "../lib/tauri";

interface AddModelModalProps {
  providerId: string;
  providerName: string;
  onClose: () => void;
  onSuccess: () => void;
}

export function AddModelModal({
  providerId,
  providerName,
  onClose,
  onSuccess,
}: AddModelModalProps) {
  const [modelId, setModelId] = useState("");
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [maxFileSizeMb, setMaxFileSizeMb] = useState("100");
  const [error, setError] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError("");

    if (!modelId.trim() || !name.trim()) {
      setError("モデル ID と名前は必須です");
      return;
    }

    const size = parseInt(maxFileSizeMb, 10);
    if (isNaN(size) || size <= 0) {
      setError("ファイルサイズは正の整数で入力してください");
      return;
    }

    setIsSubmitting(true);
    try {
      await addCustomModel({
        providerId,
        modelId: modelId.trim(),
        name: name.trim(),
        description: description.trim(),
        maxFileSizeMb: size,
      });
      onSuccess();
    } catch (err) {
      setError(`追加に失敗: ${err}`);
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>カスタムモデル追加 - {providerName}</h2>
          <button className="modal-close-btn" onClick={onClose}>
            ✕
          </button>
        </div>

        <form onSubmit={handleSubmit} className="modal-form">
          {error && <div className="modal-error">{error}</div>}

          <label>
            モデル ID <span className="required">*</span>
            <input
              type="text"
              value={modelId}
              onChange={(e) => setModelId(e.target.value)}
              placeholder="claude-3-5-sonnet-20241022"
              required
            />
            <span className="input-hint">
              API ドキュメントに記載されている正確な ID
            </span>
          </label>

          <label>
            表示名 <span className="required">*</span>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Claude 3.5 Sonnet (2024-10-22)"
              required
            />
          </label>

          <label>
            説明（任意）
            <input
              type="text"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="最新の Sonnet モデル"
            />
          </label>

          <label>
            最大ファイルサイズ (MB)
            <input
              type="number"
              value={maxFileSizeMb}
              onChange={(e) => setMaxFileSizeMb(e.target.value)}
              min="1"
              max="1000"
            />
          </label>

          <div className="modal-actions">
            <button
              type="button"
              className="modal-btn modal-btn-cancel"
              onClick={onClose}
              disabled={isSubmitting}
            >
              キャンセル
            </button>
            <button
              type="submit"
              className="modal-btn modal-btn-primary"
              disabled={isSubmitting}
            >
              {isSubmitting ? "追加中..." : "追加"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
