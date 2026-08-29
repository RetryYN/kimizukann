# kimizukann — Cursor サンドボックス内プロジェクト

## 境界（絶対ルール）
- 作業対象は **このフォルダ配下のみ**。親 `cursor-work` の他プロジェクトも含め、外のパスを読み書き・実行しない
- WSL（`\wsl$`、`wsl.exe`）、VPS（`ssh helix-worker`）、`HELIX*`、`Desktop`、`OneDrive` には触れない
  - HELIX-HARNESS の正本は WSL 側 `~/HELIX-HARNESS`。ここには存在しないし、複製もしない
- `git push`、グローバルインストール、レジストリ・環境変数・サービス変更は実行前にユーザーへ確認
- 秘密情報（`.env`、トークン、鍵）はコミットしない

## 実行環境
- Windows ネイティブ（PowerShell / Git Bash）。WSL は使わない
