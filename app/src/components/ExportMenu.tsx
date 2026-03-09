import { useState, useCallback } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import { useTranscriptStore } from "../stores/transcriptStore";
import { exportTranscript, getFileExtension } from "../lib/export";
import type { ExportFormat } from "../types";

const FORMATS: { label: string; value: ExportFormat }[] = [
  { label: "Markdown (.md)", value: "markdown" },
  { label: "Text (.txt)", value: "txt" },
  { label: "JSON (.json)", value: "json" },
];

export function ExportMenu() {
  const [open, setOpen] = useState(false);
  const current = useTranscriptStore((s) => s.current);

  const handleExport = useCallback(
    async (format: ExportFormat) => {
      if (!current) return;
      setOpen(false);

      const ext = getFileExtension(format);
      const baseName = current.fileName.replace(/\.[^/.]+$/, "");

      const filePath = await save({
        defaultPath: `${baseName}.${ext}`,
        filters: [{ name: ext.toUpperCase(), extensions: [ext] }],
      });

      if (!filePath) return;

      const content = exportTranscript(current, format);
      try {
        await writeTextFile(filePath, content);
      } catch (e) {
        useTranscriptStore.getState().setError(`Export failed: ${e}`);
      }
    },
    [current]
  );

  return (
    <div className="export-menu-wrapper">
      <button className="btn-export" onClick={() => setOpen(!open)}>
        Export
      </button>
      {open && (
        <div className="export-dropdown">
          {FORMATS.map((f) => (
            <button
              key={f.value}
              className="export-option"
              onClick={() => handleExport(f.value)}
            >
              {f.label}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
