# [DD-D5-environments-001][brief] DD-D5 詳細設計（4 環境プリセット JSON）の起草（writer=cursor-kimi, review=cursor-grok(敵対), merge=Claude）

前提: オーナー指示 2026-08-30『詳細設計まで進めて』の第 2 弾。上位正本は要件定義書 v0.2（REQ-SCOPE-01、REQ-ENV-01..04、REQ-SIM-07/10）・v0.5 §1.1/§7.3・BD-03/05/06・BD-08 §5・DD-D2。矛盾を見つけたら書き換えず D5-Qk として Claude に一括 escalate する

## スコープ（docs のみ。コード変更なし）
1. `docs/20_design/detail/DD-D5-environments.md` を新設し、以下を確定する
   - 環境レコード 7 フィールドの契約（DD §1。REQ-ENV-01）
   - 4 環境プリセットの内容: 初期物質総量 40_960_000_000 統一（REQ-ENV-03）、分布・拡散係数・流入・expected_niche_tags（DD §2）
   - 分布マップの compact patch 表現と決定的展開規則（DD §3）
   - schema 要件・UT 実数仕様 8 件・AT-D5-01..04 対応表（DD §4〜§6）
2. 未決事項 D5-Q1（local_waste の空間可変拡散）・D5-Q2（regime_id 不採用）・D5-Q3（流入プリセットなし）を DD §9 に列挙し、PR 作成と同時に Claude へ一括 escalate

## 触ってよいファイル（one-file-one-writer）
`docs/20_design/detail/DD-D5-environments.md`（新規）, `docs/50_records/briefs/DD-D5-environments-001.md`（本ファイル）, `docs/20_design/trace.md`（gen_trace 再生成のみ）。
**触らない**: `crates/**`, 他の BD/DD, 既存 schema, golden, DD-D4（PR #29 の管轄）

## 依存
- 初期配置の既定値は DD-D4 §3（PR #29、審査中）を参照。DD-D4 未マージでも本文は自立するよう参照は節番号のみ

## 後続（本 PR マージ後、別 brief）
- grok による D5 実装 PR（environment.rs / environment.schema.json / 4 プリセット JSON / d5_environments.rs。DD §8 の分割案に従う）

## 提出
`[DD-D5-environments-001][result] pr=<n> head=<sha>` を cursor-grok（審査依頼）と claude（D5-Qk 裁定依頼）へ post。レビュー観点（grok）: REQ-ENV-01..04 との整合・総量の算術・patch 展開の決定性・DD-D2 との係数整合
