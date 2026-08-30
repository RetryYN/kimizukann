<!-- 全項目必須。空欄があれば pr-lint が Draft に戻します -->

## ID
<!-- brief / カード ID。ブランチ名と一致: 例 D2-diffuse-001 -->

writer: <identity>

## 種別
<!-- feat | fix | design | docs | test | ci | rfc | hotfix -->

## 参照
Refs: REQ-… / BD-… / DD-… / ADR-…

## 変更内容
<!-- 何を・なぜ。設計（DD）からの逸脱があれば明記（逸脱は原則 RFC） -->

## 触ったファイル
<!-- git diff --stat をそのまま貼る -->
```
```

## テスト
- 追加/変更したテスト（ID）:
- テスト commit が実装 commit より前か: yes / no（no なら理由）
- golden の変更: なし / あり（Claude 承認 #）

## 検証手順（reviewer が自分で実行する）
```
cargo test --workspace
cargo run -p kimizukann-sim-cli -- verify --suite <suite>
```

## CI
<!-- run URL。Draft のうちは空でよい -->

## 参考（自分の環境の結果。合否には使わない）

## チェックリスト
- [ ] 300 行以内（超える場合は分割理由を書いた）
- [ ] `#[ignore]` / `allow(clippy)` / `--no-verify` を追加していない
- [ ] 秘密情報を含まない
- [ ] trace（`scripts/gen_trace.py`）を更新した
