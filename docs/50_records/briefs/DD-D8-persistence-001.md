# [DD-D8-persistence-001][brief] DD-D8 詳細設計（保存・中断再開）の起草（writer=cursor-kimi, review=cursor-grok(敵対), merge=Claude）

前提: オーナー指示 2026-08-30『詳細設計まで進めて』の第 5 弾。上位正本は BD-10 0.3・BD-01 r4 §5・BD-05 §10/§12/§13/§14・要件定義書 v0.2（REQ-DET-02/06、REQ-CON-08、REQ-EVT-04、REQ-NFR-02/06）・BD-08 §8。未決は D8-Qk として Claude へ一括 escalate

## スコープ（docs のみ。コード変更なし）
1. `docs/20_design/detail/DD-D8-persistence.md` を新設し、以下を確定する
   - save.rs の実装構造（core はバイト列のみ、正規化ライタ自前）（DD §1）
   - load 検証順と 4 エラー経路（DD §2。BD-10 §2 の展開）
   - 書込み・2 世代ローテーションの契約（DD §3。Presentation 側手順を固定）
   - 台帳保持ポリシの問題定義と推奨案（DD §4 → D8-Q1）
   - migration 運用・UT 実数仕様 8 件・AT-D8-01..05 対応（DD §5〜§7）
2. 未決事項 D8-Q1（台帳保持ポリシ）・D8-Q2（cells 件数不整合のエラー種別）を DD §8 に列挙し Claude へ一括 escalate

## 触ってよいファイル（one-file-one-writer）
`docs/20_design/detail/DD-D8-persistence.md`（新規）, `docs/50_records/briefs/DD-D8-persistence-001.md`（本ファイル）, `docs/20_design/trace.md`（gen_trace 再生成のみ）。
**触らない**: `crates/**`, 他の BD/DD, 既存 schema, golden, DD-D4〜D7（PR #29〜#32 の管轄）

## 後続（本 PR マージ後、別 brief）
- grok による D8 実装 PR（save.rs / save.schema.json 更新 / d8_save.rs。DD §9 の分割案に従う）

## 提出
`[DD-D8-persistence-001][result] pr=<n> head=<sha>` を cursor-grok（審査依頼）と claude（D8-Qk 裁定依頼）へ post。レビュー観点（grok）: BD-10 との逐語整合（12 フィールド・正規化・検証順）・台帳保持ポリシの問題定義の妥当性・UT の網羅性
