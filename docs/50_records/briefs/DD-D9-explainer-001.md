# [DD-D9-explainer-001][brief] DD-D9 詳細設計（説明器）の起草（writer=cursor-kimi, review=cursor-grok(敵対), merge=Claude）

前提: オーナー指示 2026-08-30『詳細設計まで進めて』の第 6 弾。上位正本は BD-12 0.3・BD-01 r4・BD-03・BD-07 §4.1・要件定義書 v0.2（REQ-EVT-01..05、REQ-EXP-01..06）・BD-08 §9。BD-12 が「詳細設計で固定する」とした集計の実装詳細を本稿で固定する

## スコープ（docs のみ。コード変更なし）
1. `docs/20_design/detail/DD-D9-explainer.md` を新設し、以下を確定する
   - Explain 純関数の入出力契約（DD §1）
   - 転換点検出器の実装固定（母標準偏差・std=0 回避・集約の同率ルール）（DD §2）
   - 理由コードの集計窓（20 サンプル）と θ 初期値（DD §3）
   - 4 段出力 JSON 構造・禁止語リント適用点（DD §4/§5）
   - UT 実数仕様 8 件（DD §6）
2. 未決事項 D9-Q1（理由コード集計窓）を DD §8 に列挙し Claude へ escalate

## 触ってよいファイル（one-file-one-writer）
`docs/20_design/detail/DD-D9-explainer.md`（新規）, `docs/50_records/briefs/DD-D9-explainer-001.md`（本ファイル）, `docs/20_design/trace.md`（gen_trace 再生成のみ）。
**触らない**: `crates/**`, 他の BD/DD, 既存 schema, golden, DD-D4〜D8（PR #29〜#33 の管轄）

## 後続（本 PR マージ後、別 brief）
- grok による D9 実装 PR（crates/sim-analysis 新設。DD §7 の分割案に従う）

## 提出
`[DD-D9-explainer-001][result] pr=<n> head=<sha>` を cursor-grok（審査依頼）と claude（D9-Q1 裁定依頼）へ post。レビュー観点（grok）: BD-12 との整合（スコア式・種別・写像表を変更していないこと）・z 計算の境界（std=0）・集約の決定性
