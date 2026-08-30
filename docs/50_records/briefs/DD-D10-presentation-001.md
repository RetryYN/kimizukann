# [DD-D10-presentation-001][brief] DD-D10 詳細設計（Presentation・描画トークン）の起草（writer=cursor-kimi, review=cursor-grok(敵対), merge=Claude）

前提: オーナー指示 2026-08-30『詳細設計まで進めて』の第 7 弾。上位正本は 要件定義書 v0.2（REQ-VIS-01..04、REQ-OUT-04、REQ-CON-05）・v0.4 §1.7/付録 C・v0.5 §1.4・BD-03 §1.2・BD-05 §12.4・BD-11 §2・BD-08 §9

## スコープ（docs のみ。コード変更なし）
1. `docs/20_design/detail/DD-D10-presentation.md` を新設し、以下を確定する
   - snapshot バッファ消費手順（DD §1）
   - 形状主・色補助の符号表（DD §2。v0.4 付録 C の確定値を転記）
   - 表示用トークン生成規則（最大 8・5 tick ごと・揮発 ID・PRNG 非消費）（DD §3）
   - 描画間引きの非干渉（DD §4）
   - UT 3 件・AT-D10-01/02 対応（DD §5）
2. 未決事項なし（D10-Q なし。必要な値は全て上位正本で確定済み）

## 触ってよいファイル（one-file-one-writer）
`docs/20_design/detail/DD-D10-presentation.md`（新規）, `docs/50_records/briefs/DD-D10-presentation-001.md`（本ファイル）, `docs/20_design/trace.md`（gen_trace 再生成のみ）。
**触らない**: `crates/**`, `app/**`, 他の BD/DD, 既存 schema, golden, DD-D4〜D9（PR #29〜#34 の管轄）

## 後続（本 PR マージ後、別 brief）
- grok による D10 実装 PR（`app/lib/render/`。DD §6 の分割案に従う）

## 提出
`[DD-D10-presentation-001][result] pr=<n> head=<sha>` を cursor-grok（審査依頼）へ post。レビュー観点（grok）: REQ-VIS/REQ-OUT-04 との整合・§12.4 レイアウトの転記正確性・トークン揮発 ID ルールの検証可能性
