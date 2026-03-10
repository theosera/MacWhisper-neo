use serde::Serialize;
use std::path::PathBuf;
use std::process::Command;
use tauri::Manager;

use crate::error::AppError;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeDownloadResult {
    pub path: String,
    pub file_name: String,
    pub video_id: String,
    pub title: String,
}

/// YouTube URL から video_id を抽出する
fn extract_video_id(url: &str) -> Result<String, AppError> {
    let url = url.trim();

    // youtube.com/watch?v=VIDEO_ID
    if let Some(pos) = url.find("watch?v=") {
        let rest = &url[pos + 8..];
        let id: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_').collect();
        if id.len() >= 11 {
            return Ok(id);
        }
    }

    // youtu.be/VIDEO_ID
    if url.contains("youtu.be/") {
        if let Some(pos) = url.find("youtu.be/") {
            let rest = &url[pos + 9..];
            let id: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_').collect();
            if id.len() >= 11 {
                return Ok(id);
            }
        }
    }

    // youtube.com/shorts/VIDEO_ID
    if url.contains("/shorts/") {
        if let Some(pos) = url.find("/shorts/") {
            let rest = &url[pos + 8..];
            let id: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_').collect();
            if id.len() >= 11 {
                return Ok(id);
            }
        }
    }

    // youtube.com/embed/VIDEO_ID
    if url.contains("/embed/") {
        if let Some(pos) = url.find("/embed/") {
            let rest = &url[pos + 7..];
            let id: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_').collect();
            if id.len() >= 11 {
                return Ok(id);
            }
        }
    }

    Err(AppError::YoutubeDownload(
        "有効な YouTube URL ではありません。watch?v=, youtu.be/, shorts/ 形式に対応しています。".to_string()
    ))
}

/// 同梱された yt-dlp バイナリのパスを解決する
fn resolve_ytdlp_path(app: &tauri::AppHandle) -> Result<PathBuf, AppError> {
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|e| AppError::YoutubeDownload(format!("リソースディレクトリの解決に失敗: {e}")))?;
    // Tauri bundle.resources の配置差異に対応
    let candidates = [
        resource_dir.join("yt-dlp"),
        resource_dir.join("resources").join("yt-dlp"),
        resource_dir.join("..").join("Resources").join("yt-dlp"),
    ];

    for path in &candidates {
        if path.exists() {
            return Ok(path.to_path_buf());
        }
    }

    // 開発環境用フォールバック: src-tauri/resources/yt-dlp
    let dev_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join("yt-dlp");
    if dev_path.exists() {
        return Ok(dev_path);
    }

    Err(AppError::YoutubeDownload(
        "yt-dlp バイナリが見つかりません。アプリのリソースが破損している可能性があります。".to_string(),
    ))
}

/// 一時ディレクトリのパスを生成する
fn temp_dir() -> PathBuf {
    std::env::temp_dir().join("macwhisper-neo")
}

#[tauri::command]
pub async fn download_youtube(
    url: String,
    app: tauri::AppHandle,
) -> Result<YoutubeDownloadResult, AppError> {
    let video_id = extract_video_id(&url)?;
    let ytdlp_path = resolve_ytdlp_path(&app)?;

    // 実行権限を確認・付与
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&ytdlp_path)?;
        let mut perms = metadata.permissions();
        if perms.mode() & 0o111 == 0 {
            perms.set_mode(perms.mode() | 0o755);
            std::fs::set_permissions(&ytdlp_path, perms)?;
        }
    }

    let output_dir = temp_dir();
    std::fs::create_dir_all(&output_dir)?;

    let output_template = output_dir
        .join(format!("{video_id}.%(ext)s"))
        .to_string_lossy()
        .to_string();

    // yt-dlp でタイトルを取得
    let title_output = Command::new(&ytdlp_path)
        .args(["--no-playlist", "--get-title", "--encoding", "utf-8", &url])
        .output()
        .map_err(|e| AppError::YoutubeDownload(format!("yt-dlp の起動に失敗: {e}")))?;

    let title = if title_output.status.success() {
        String::from_utf8_lossy(&title_output.stdout).trim().to_string()
    } else {
        video_id.clone()
    };

    // yt-dlp で音声のみダウンロード (mp3)
    let output = Command::new(&ytdlp_path)
        .args([
            "--no-playlist",
            "--extract-audio",
            "--audio-format", "mp3",
            "--audio-quality", "0",
            "--output", &output_template,
            "--encoding", "utf-8",
            "--no-progress",
            &url,
        ])
        .output()
        .map_err(|e| AppError::YoutubeDownload(format!("yt-dlp の起動に失敗: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let user_msg = if stderr.contains("Video unavailable") || stderr.contains("Private video") {
            "この動画は非公開または利用できません。".to_string()
        } else if stderr.contains("Sign in") {
            "年齢制限付き動画のため、ダウンロードできません。".to_string()
        } else if stderr.contains("HTTP Error 429") {
            "リクエスト制限に達しました。しばらく待ってから再試行してください。".to_string()
        } else {
            format!("YouTube 音声のダウンロードに失敗しました: {}", stderr.chars().take(200).collect::<String>())
        };
        return Err(AppError::YoutubeDownload(user_msg));
    }

    let mp3_path = output_dir.join(format!("{video_id}.mp3"));
    if !mp3_path.exists() {
        return Err(AppError::YoutubeDownload(
            "ダウンロードは完了しましたが、MP3 ファイルが見つかりません。".to_string()
        ));
    }

    let file_name = format!("{title}.mp3");

    Ok(YoutubeDownloadResult {
        path: mp3_path.to_string_lossy().to_string(),
        file_name,
        video_id,
        title,
    })
}

#[tauri::command]
pub fn cleanup_youtube_temp_file(path: String) -> Result<(), AppError> {
    let target = PathBuf::from(path);
    if !target.exists() {
        return Ok(());
    }
    let base = temp_dir();

    let canonical_base = base.canonicalize()?;
    let canonical_target = target.canonicalize()?;
    if !canonical_target.starts_with(&canonical_base) {
        return Err(AppError::Validation(
            "cleanup対象が許可ディレクトリ外です".to_string(),
        ));
    }

    if canonical_target.is_file() {
        std::fs::remove_file(canonical_target)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_video_id_watch() {
        let id = extract_video_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ").unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_video_id_short_url() {
        let id = extract_video_id("https://youtu.be/dQw4w9WgXcQ").unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_video_id_shorts() {
        let id = extract_video_id("https://www.youtube.com/shorts/dQw4w9WgXcQ").unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_video_id_embed() {
        let id = extract_video_id("https://www.youtube.com/embed/dQw4w9WgXcQ").unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_video_id_with_params() {
        let id = extract_video_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=120s").unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_video_id_invalid() {
        assert!(extract_video_id("https://example.com").is_err());
        assert!(extract_video_id("not a url").is_err());
    }
}
