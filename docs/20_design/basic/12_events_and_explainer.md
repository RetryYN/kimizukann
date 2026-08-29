# BD-12 ドメインイベント・転換点・説明器

- 版: 0.1（起草 cursor-kimi、2026-08-30）
- 入力: `docs/10_requirements/要件定義書_検証版_v0.2.md`（sign-off 済）。台帳の正本は BD-03 §1 / BD-05 §3、台帳ダイジェストの正本は BD-01 r3 §5、保存は BD-10
- 完成条件: REQ-EVT-01〜05・REQ-EXP-01〜06 の全件に対応（§6 の対応表）
- 数値は「確定 / 初期仮説（D8 の較正で確定、OPEN-04）」を明記する。文言本文（禁止語リスト・4 段の文体）は gemini の [TERM] 領域とし、本章は構造のみ定義する

## 1. イベントの 3 層

| 層 | 内容 | 保存 | 参照 |
|---|---|---|---|
| フロー台帳 | `LedgerEntry { tick, cell_index, lineage, from_pool, to_pool, amount, reason }`、reason は ReasonCode 7 種（Intake/Maintenance/Starvation/Death/Reproduction/Emission/Diffusion。BD-03 が正本） | region 単位に集約して LedgerSave へ。セル単位の全 tick 履歴は保存しない | REQ-SIM-05, REQ-EVT-04 |
| 転換点 | §3 のスコアで検出する注目イベント（§2 の種別）。`event_id / tick / kind / region_ids / score / evidence_refs` を持つ | 保存 32 件上限（REQ-EVT-02） | REQ-EVT-01〜05 |
| スタンプ | 転換点のうち皿上にピン表示するもの（3 件）。UI からの `explain` クエリの対象 | LedgerSave のスタンプ列 | REQ-EVT-02, REQ-UI-07 |

- 転換点・スタンプは**表示専用**。検出の有無・on/off が state hash に影響しない（AT-D8-04）。Explain は純関数 `(SnapshotView, LedgerView, query) -> bytes` で、入力を変えずイベントを消費しない。確定。参照: REQ-EVT-05, REQ-EXP-03, BD-01 r3 §4

## 2. 転換点の種別（REQ-EVT-03。確定）

急増／急減／資源枯渇／初死骸利用／逆転／絶滅／固定候補の 7 種（捕食・分岐は対象外、REQ-OUT-01）。region は 4 連結成分・最大 16（REQ-EVT-04）。各種別の検出トリガ:

| 種別 | トリガ（台帳・時系列からの機械定義） |
|---|---|
| 急増 / 急減 | 系統 × pool の 10 tick 平均系列で \|z\| が閾値超え（§3） |
| 資源枯渇 | region の nutrient が θ（初期中央値の 10%）割れ + 当該 region で Starvation/Maintenance エントリ増 |
| 初死骸利用 | use_carcass 系統の carcass 由来 Intake エントリの初出 |
| 逆転 | 終了時 1 位が tick 0 順位（BD-05 §1 の tick0_ranking）で 3 位以下（REQ-END-04b と同じ定義） |
| 絶滅 | 系統の総生体量 < ε（REQ-END-02 と同じ定義） |
| 固定候補 | 1 系統が総生体量 70% 以上（REQ-END-03 の fixed_streak 途中） |

## 3. 転換点スコア（REQ-EVT-01/02）

- 時系列: `biomass[L], nutrient, carcass, waste` のセル合計を 10 tick 平均、窓 20 サンプルで z。確定（REQ-EVT-01）
- スコア = `0.5·|z|_clip3 + 0.2·継続率 + 0.2·影響生体量比 + 0.1·新規性 + 種別ボーナス（≤ 0.15）`。確定（式は REQ-EVT-01）。各項の定義（初期仮説、D8 較正で確定、OPEN-04）:
  - `|z|_clip3` = min(|z|, 3) / 3（0〜1 に正規化）
  - 継続率 = 窓内で \|z\| ≥ 1 のサンプル割合
  - 影響生体量比 = 対象系統の当該窓の変化量 ÷ 総生体量
  - 新規性 = 同一 run で同種別が未出なら 1、出済みなら 0.5
  - 種別ボーナス: 初死骸利用・逆転・絶滅 = 0.15、資源枯渇・固定候補 = 0.10、急増・急減 = 0.05
- 検出閾値 score ≥ 1.2、近接同種の集約半径 30 tick、上限は皿上 3／タイムライン 12／保存 32 件。確定（REQ-EVT-02）。空転・満杯の健全性は AT-D8-03 の分布帯で判定
- スコア計算は解析 crate で浮動小数点を許可するが、コア状態へ逆流しない（REQ-CON-02、BD-07 §4.1 の例外行）

## 4. 理由コードとレバー写像（REQ-EXP-03/04）

- 説明の根拠はフロー台帳のドメインイベント列のみから導く理由コード。テンプレート + 理由コードのみで生成し、生成 AI・外部 solver を使わない。確定。参照: REQ-EXP-03, REQ-OUT-05
- 理由コード（top_contributors の候補）: `RESOURCE_SKEW`（資源偏在: 台帳の Intake が少数 region に集中）/ `REPRO_DRIVEN`（繁殖由来: Reproduction エントリが増加の主因）/ `NICHE_MISMATCH`（ニッチ不適合: 当該系統の Intake 効率が環境の期待ニッチと不整合）/ `LOW_DISPERSION`（分散不足: 初期配置の占有セル数が少ない）/ `NONE`（該当なし）。各コードの集計定義は詳細設計（DD-D9-*）で固定する。初期仮説
- レバー写像表（8 行以内。確定、REQ-EXP-04）:

| 理由コード | もしもレバー |
|---|---|
| RESOURCE_SKEW | 配置 |
| REPRO_DRIVEN | 個体数 |
| NICHE_MISMATCH | 適応方針（札） |
| LOW_DISPERSION | ばらつき |
| NONE | 配置 |

## 5. 説明器の出力構造（REQ-EXP-01/02/05/06）

- 「今どうなってる？」は **観測した事実 → 有力な解釈 → まだ不明な点 → 次の一手** の 4 段固定。確定。参照: REQ-EXP-01
- 事実（台帳・観測値）／モデル推論（理由コード由来）／未知（データ不足）を表示上で区別する（ラベルまたは区画）。確定。参照: REQ-EXP-02
- 絶滅時も原因候補（理由コード上位）と再実験の入口（もしもレバー）を必ず示す。確定。参照: REQ-EXP-05
- 単一 seed からの断定表現を禁止する禁止語リスト（「毎回こうなる」等）は文言リントで機械検査。リスト本文は gemini が [TERM] で管理。確定。参照: REQ-EXP-06

## 6. REQ 対応表

| REQ | 対応 | 検証 |
|---|---|---|
| REQ-EVT-01 | §3（スコア式・時系列定義） | UT（固定入力で期待値） |
| REQ-EVT-02 | §3（閾値・半径・上限） | UT + AT-D8-03 |
| REQ-EVT-03 | §2（7 種別） | UT |
| REQ-EVT-04 | §1（region 集約・履歴非保存） | UT + MEAS（BD-10 §5） |
| REQ-EVT-05 | §1（表示専用・hash 非干渉） | AT-D8-04 |
| REQ-EXP-01 | §5（4 段固定） | UT（出力構造）+ USER |
| REQ-EXP-02 | §5（事実/推論/未知の区別） | INSP + USER |
| REQ-EXP-03 | §4（テンプレート + 理由コードのみ） | INSP + UT |
| REQ-EXP-04 | §4（写像表 5 行） | UT |
| REQ-EXP-05 | §5（絶滅時の原因候補と入口） | UT + USER |
| REQ-EXP-06 | §5（禁止語リント） | INSP（文言リント） |
