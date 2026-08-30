# [DD-D4-lineages-001][brief] DD-D4 詳細設計（4 系統プリセット・初期配置・終了判定）の起草（writer=cursor-kimi, review=cursor-grok(敵対), merge=Claude）

前提: オーナー指示 2026-08-30『詳細設計まで進めて』により、kimi が DD-D4→D12 を 1 モジュール 1 PR で連続起草する。本 PR はその第 1 弾。上位正本は要件定義書 v0.2（sign-off 済）・v0.5 統合案・BD-03/04/06/07・BD-08 §4。矛盾を見つけたら書き換えず D4-Qk として Claude に一括 escalate する（1 PR 1 往復裁定）

## スコープ（docs のみ。コード変更なし）
1. `docs/20_design/detail/DD-D4-lineages.md` を新設し、以下を確定する
   - 4 系統プリセット: 案 A ベクトル（REQ-GEN-03）・機構タグ割当・系統定数（DD §1）
   - 終了判定: 5 ラベルの判定手順・fixed_streak / tick0_ranking の状態・集計の数値要件（DD §2。REQ-END-01..05、BD-04 T1、BD-06 P11/P12）
   - 初期配置: default / explicit / random の 3 モードと interaction 消費固定方式（DD §3。REQ-SCOPE-03、BD-07 §3）
   - 遺伝的ばらつき: 離散アレル ±0.05 と mutation 消費（DD §4。REQ-GEN-08）
   - config schema 要件と UT 実数仕様 12 件・AT-D4-01..08 対応表（DD §5〜§7）
2. 未決事項 D4-Q1（アカバエ死亡閾値の解釈）・D4-Q2（density_bonus の効果）・D4-Q3（アレル適用単位と BD-07 消費回数）を DD §10 に列挙し、PR 作成と同時に Claude へ一括 escalate

## 触ってよいファイル（one-file-one-writer）
`docs/20_design/detail/DD-D4-lineages.md`（新規）, `docs/50_records/briefs/DD-D4-lineages-001.md`（本ファイル）, `docs/20_design/trace.md`（gen_trace 再生成のみ）。
**触らない**: `crates/**`, 他の BD/DD, schema, golden。`trace.md` の再生成で `diffuse_bench.rs` 由来の 3 行（#25 マージ分の正規ドリフト）が復活するが、これは gen_trace の正しい出力であり本 PR で正規化する

## 後続（本 PR マージ後、別 brief）
- grok による D4 実装 PR（termination.rs / placement.rs / プリセット JSON / d4_lineages.rs。DD §9 の分割案に従う）
- D4-Q1〜Q3 の裁定結果は DD-D4 0.2 改訂または実装 PR に反映

## 提出
`[DD-D4-lineages-001][result] pr=<n> head=<sha>` を cursor-grok（審査依頼）と claude（D4-Qk 裁定依頼）へ post。レビュー観点（grok）: 上位正本との数値一致（案 A 表・REQ-END 閾値・BD-06/07 との整合）・敵対的解釈の余地の潰し具合
