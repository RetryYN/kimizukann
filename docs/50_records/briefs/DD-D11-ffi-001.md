# [DD-D11-ffi-001][brief] DD-D11 詳細設計（FFI 境界・reference_scenarios ゲート）の起草（writer=cursor-kimi, review=cursor-grok(敵対), merge=Claude）

前提: オーナー指示 2026-08-30『詳細設計まで進めて』の第 8 弾。上位正本は BD-05 §12（FFI 7 操作の正本）・BD-10 §2・BD-11 §4・要件定義書 v0.2（REQ-CON-01/05/08、REQ-DET-06/09、REQ-ACC-05、REQ-NFR-06）・BD-08 §9

## スコープ（docs のみ。コード変更なし）
1. `docs/20_design/detail/DD-D11-ffi.md` を新設し、以下を確定する
   - FFI crate（crates/sim-ffi）の実装構造: catch_unwind・handle 同時 1・バッファプロトコル（DD §1）
   - reference_scenarios.json の形式と model_version ゲート（DD §2）
   - チュートリアル seed 受入基準（DD §2.2）
   - UT 実数仕様 6 件（DD §3）
2. 未決事項 D11-Q1（panic 捕捉時の返却コード。推奨: KZ_ERR_INTERNAL=8 を semver minor で追加）を DD §5 に列挙し Claude へ escalate

## 触ってよいファイル（one-file-one-writer）
`docs/20_design/detail/DD-D11-ffi.md`（新規）, `docs/50_records/briefs/DD-D11-ffi-001.md`（本ファイル）, `docs/20_design/trace.md`（gen_trace 再生成のみ）。
**触らない**: `crates/**`, `app/**`, 他の BD/DD, 既存 schema, golden, DD-D4〜D10（PR #29〜#35 相当の管轄）

## 後続（本 PR マージ後、別 brief）
- grok による D11 実装 PR（crates/sim-ffi 新設 + scripts/check_reference_scenarios.py。DD §4 の分割案に従う）

## 提出
`[DD-D11-ffi-001][result] pr=<n> head=<sha>` を cursor-grok（審査依頼）と claude（D11-Q1 裁定依頼）へ post。レビュー観点（grok）: BD-05 §12.3 シグネチャを変更していないこと・§12.5 事前/事後条件との整合・バッファ 2 段プロトコルの検証可能性
