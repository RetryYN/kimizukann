# 基本設計書（外部設計）— 構成と完成条件

入力: `docs/10_requirements/要件定義書_検証版_v0.2.md`（sign-off 済み）。各章は担当が起草し、章ごとに「検証方法」列を持つ。**AT 設計（BD-08）が書かれ、trace で全 P0 要求が章と AT に紐付くまで基本設計は未完成**。

| 章 | ファイル | 内容 | 起草 | 審査 | 完成条件 |
|---|---|---|---|---|---|
| BD-01 | `01_context_map.md` | 境界づけられたコンテキスト（SimCore / Calibration / Presentation / Distribution）、依存方向、腐敗防止層（FFI） | Claude | grok | 依存方向が CI（cargo-deny + 自作チェック）で検査可能 |
| BD-02 | `02_glossary.md` | ユビキタス言語: 語 / 定義 / 型 / 単位 / 値域 / 出典 REQ | gemini | kimi | 要件定義書・契約・コードの識別子が全て載る。未登録語 0 |
| BD-03 | `03_domain_model.md` | 集約（World/Cell/Ledger/Rng/Termination/Save）、値オブジェクト、**不変条件を式で**、違反時の挙動 | kimi | Claude | 各不変条件に property test の雛形（入力生成・assert）が付く |
| BD-04 | `04_state_machines.md` | Run / Cell 占有 / Lineage-in-cell の状態遷移表（状態×イベント→次状態・ガード・アクション） | kimi | grok | 全状態×全イベントが表に埋まり、生成テストが書ける |
| BD-05 | `05_contract.md` | 既存 `docs/30_contracts/simulation_contract.md` を本章として再編。公開 API（Rust シグネチャ）、7 操作 FFI（C ABI・バッファ規約）、schema、事前/事後条件、semver 規則 | kimi | Claude | `cargo public-api` の出力と一致。各 pub 項目に REQ 参照 |
| BD-06 | `06_numeric_model.md` | scale、各量の上限、phase ごとの最大中間値と使用ビット幅の**証明表**、丸め、余り、NumericError の条件 | kimi | grok | 64×64×8 系統×2000 tick で i128 に収まる計算が表で示され、上限値が UT の境界値になる |
| BD-07 | `07_determinism_model.md` | 走査順、4 ストリームの用途割当と 1 tick の消費回数表、禁止構造（HashMap/浮動小数点）の lint 設定、三経路・クロス OS の検証手順 | kimi | grok | 消費回数表が UT で検査できる。lint 設定が CI にある |
| BD-08 | `08_acceptance_tests.md` | **AT 設計**: 段階 D2〜D12 ごとに AT-ID / 対応 REQ / 入力 / 期待 / 判定方法。gate 自己試験用 mutation 一覧 | kimi | Claude | 全 P0 REQ に ≥1 AT。各 AT に REQ と章参照 |
| BD-09 | `09_performance_budget.md` | 段階ごとの予算（REQ-NFR-01/02）、計測方法（criterion / 実機）、基準端末（OPEN-03 決定済: 2021 年以降の全スマホ→床端末で導出） | Claude | kimi | 予算値が CI しきい値になる |
| BD-10 | `10_persistence.md` | SaveEnvelope、schema_version / model_version の bump 規則、migration 方針とテスト | kimi | Claude | 旧 save 読込テストの設計がある |
| BD-11 | `11_ui_flow.md` | 一巡の画面遷移（REQ-UI-01）、スケジューラ規則（REQ-UI-03 の 7 要素）、FFI 呼出順、中断復帰 | grok | kimi | 遷移が表で書かれ、AT-D12 に対応 |
| BD-12 | `12_events_and_explainer.md` | ドメインイベント一覧、フロー台帳レコード、転換点スコア、理由コード→レバー写像 | kimi | gemini（文言） | REQ-EVT/EXP の全件が対応 |
| ADR | `../adr/NNNN-*.md` | 既存決定の記録（i64 固定小数点 / xoshiro / SHA-256 / GNU toolchain / Android 1 本 / Flutter） | Claude | — | 既存決定を網羅 |
| trace | `../trace.md` | REQ → BD 章 → AT → DD → UT の対応表 | Claude | CI | P0 で AT 無し = 0 |

## 書式の規則
- 各章の各項目に `参照: REQ-…` を必ず付ける。REQ に無い設計判断は ADR を書く
- 数値は「確定 / 初期仮説（Dn で確定）」を明記
- 章間の参照は ID（BD-06 §3 など）。散文で「前述の」と書かない
- 完成条件を満たさない章はレビューに出さない

## 順序
BD-02（用語）と BD-03（ドメイン）を先行 → BD-05/06/07（契約・数値・決定性）→ BD-08（AT）→ 残り。BD-08 が揃った時点で詳細設計（DD-D2-*）に着手できる。
