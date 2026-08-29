# H2 レビュー証拠ゲートの Secret 設定

オーナーがローカルの `~/.helix-bus/keys/attest.secret`（Windows ではユーザープロファイル配下）に一度だけ署名鍵を生成し、値を表示・投稿・コミットせずに GitHub Secret へ登録する。

```powershell
$secretPath = Join-Path ([Environment]::GetFolderPath('UserProfile')) ".helix-bus\keys\attest.secret"
New-Item -ItemType Directory -Force (Split-Path $secretPath) | Out-Null
if (-not (Test-Path -LiteralPath $secretPath)) {
  $bytes = New-Object byte[] 32
  [Security.Cryptography.RandomNumberGenerator]::Fill($bytes)
  [IO.File]::WriteAllText($secretPath, [Convert]::ToBase64String($bytes))
}
gh secret set HELIX_ATTEST_SECRET < $secretPath
```

`HELIX_ATTEST_SECRET` は Actions の `review-gate` だけへ渡す。鍵のローテーション時は Secret を更新し、既存のレビュー票を stale として再レビューする。

`pr_lint.py` は GitHub の pull-request files API（merge-base を使う三点比較）の統計を正本として本文の `diff --stat` と突合する。同期用の merge commit（`HEAD~1`）ではなく、PR が実際に変更したファイル集合を比較するためである。
