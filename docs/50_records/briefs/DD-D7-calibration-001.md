# [DD-D7-calibration-001][brief] DD-D7 詳細設計（較正ゲート）の起草（writer=cursor-kimi, review=cursor-grok(敵対), merge=Claude）

前提: オーナー指示 2026-08-30『詳細設計まで進めて』の第 4 弾。上位正本は要件定義書 v0.2（REQ-OPS-02b、REQ-ACC-04、REQ-GEN-05/06/07、REQ-END-05、REQ-GOAL-03）・v0.5 §1.5/§7.6・v1.0 §2.3・BD-08 §7・DD-D6。未決は D7-Qk として Claude へ一括 escalate

## スコープ（docs のみ。コード変更なし）
1. `docs/20_design/detail/DD-D7-calibration.md` を新設し、以下を確定する
   - 較正バッチ構成（4 環境 × 100 seed、batch_base = 1_000、ペア試験 suite）（DD §1）
   - 統計量の機械的定義（中央値・順位・固定率・A/B/C 型）（DD §2）
   - 合否判定（AT-D7-01..06 + ペア 80% 規則、機械判定スクリプト）（DD §3）
   - manifest.jsonl 形式（config hash・分布・変更理由）（DD §4）
   - 調整ループ規則（1 回 1 パラメータ群・勝者補正禁止・軽い戻しのみ・打切り）（DD §5）
   - UT 実数仕様 6 件（DD §6）
2. 未決事項 D7-Q1（B 型の位置条件）・D7-Q2（ペア試験 config）を DD §10 に列挙し Claude へ一括 escalate

## 触ってよいファイル（one-file-one-writer）
`docs/20_design/detail/DD-D7-calibration.md`（新規）, `docs/50_records/briefs/DD-D7-calibration-001.md`（本ファイル）, `docs/20_design/trace.md`（gen_trace 再生成のみ）。
**触らない**: `crates/**`, 他の BD/DD, 既存 schema, golden, DD-D4/D5/D6（PR #29/#30/#31 の管轄）

## 依存
- 実装は D6 実装（バッチランナ）マージ後。DD 本文は節番号参照のみで自立

## 後続（本 PR マージ後、別 brief）
- grok による D7 実装 PR（batch.rs 拡張 / judge_calibration.py / manifest schema / d7_calibration.rs。DD §9 の分割案に従う）

## 提出
`[DD-D7-calibration-001][result] pr=<n> head=<sha>` を cursor-grok（審査依頼）と claude（D7-Qk 裁定依頼）へ post。レビュー観点（grok）: 統計定義の境界（中央値・80%・5〜20% 帯）・REQ-GEN-05/06/07 の機械化の妥当性・調整ルールの v1.0 §2.3 との一致
