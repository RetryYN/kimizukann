# [DD-D6-smoke-001][brief] DD-D6 詳細設計（煙試験バッチ）の起草（writer=cursor-kimi, review=cursor-grok(敵対), merge=Claude）

前提: オーナー指示 2026-08-30『詳細設計まで進めて』の第 3 弾。上位正本は要件定義書 v0.2（REQ-OPS-02a、REQ-OUT-01、REQ-SIM-14、REQ-DET-03）・v0.5 §7.2・BD-03/05/06・BD-08 §6。未決は D6-Qk として Claude へ一括 escalate（本稿では Q なしで閉じた）

## スコープ（docs のみ。コード変更なし）
1. `docs/20_design/detail/DD-D6-smoke.md` を新設し、以下を確定する
   - 煙試験バッチの実行仕様（`sim-cli batch --suite smoke`、seed 導出、seed 間並列、時間予算）（DD §2）
   - 分布 JSON 形式（run レコード + summary、state_hash 記録）（DD §3）
   - 失敗時の終了コード（panic/NumericError/系統数違反は記録して最後まで実行・終了コード 1）（DD §4）
   - UT 実数仕様 5 件・AT-D6-01/02 対応表（DD §5/§6）

## 触ってよいファイル（one-file-one-writer）
`docs/20_design/detail/DD-D6-smoke.md`（新規）, `docs/50_records/briefs/DD-D6-smoke-001.md`（本ファイル）, `docs/20_design/trace.md`（gen_trace 再生成のみ）。
**触らない**: `crates/**`, 他の BD/DD, 既存 schema, golden, DD-D4/D5（PR #29/#30 の管轄）

## 依存
- 実装は D3・D4・D5 の実装マージ後。DD 本文はそれらの節番号参照のみで自立

## 後続（本 PR マージ後、別 brief）
- grok による D6 実装 PR（batch.rs / batch_result.schema.json / d6_smoke.rs。DD §7 の分割案に従う）

## 提出
`[DD-D6-smoke-001][result] pr=<n> head=<sha>` を cursor-grok（審査依頼）と claude へ post。レビュー観点（grok）: REQ-OPS-02a との整合・seed 導出とレコード順の決定性・終了コード設計・verify との責務分離
