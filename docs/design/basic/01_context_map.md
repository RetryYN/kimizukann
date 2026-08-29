# BD-01 コンテキストマップ

参照: REQ-GOAL-02, REQ-CON-01, REQ-CON-05, REQ-CON-08, REQ-OUT-04, REQ-OUT-05, REQ-VIS-04, REQ-NFR-04

## 1. 境界づけられたコンテキスト
| コンテキスト | 責務 | 実体（crate / dir） | 持たないもの |
|---|---|---|---|
| **SimCore** | 決定論的な世界更新、保存則、終了判定、state hash、save/load | `crates/sim-types`, `crates/sim-core` | wall clock、乱数の外部入力、浮動小数点、I/O、描画、文章 |
| **Calibration** | seed バッチ、分布集計、上位互換判定、manifest、チュートリアル seed 候補抽出 | `crates/sim-cli`（batch/verify）、`docs/calib/` | 世界更新のロジック（SimCore を呼ぶだけ）、UI |
| **Explain** | フロー台帳→ドメインイベント→転換点スコア→理由コード→テンプレート文 | `crates/sim-explain`（新設予定）、`docs/contracts/explanation_contract.md` | 状態の変更（読み取り専用）、生成 AI |
| **Presentation** | Flutter UI、再生スケジューラ、描画スナップショット、カード保存、ローカル指標ログ | `app/` | シミュレーション計算、ネットワーク |
| **Distribution** | APK ビルド・署名・配布メモ、テスター運用、検証レポート | `.github/workflows/flutter.yml`, `docs/dist/` | アプリのロジック |

## 2. 関係と依存方向（矢印の向きにのみ依存してよい）
```
sim-types ◄── sim-core ◄── sim-explain ◄── sim-ffi (C ABI) ◄── app (Flutter)
                 ▲                                ▲
                 └──── sim-cli (Calibration) ─────┘
```
- **逆方向の依存は禁止**（例: sim-core が sim-cli の型を使う、sim-types が sim-core を参照する）。CI で `cargo-deny` の bans と `scripts/check_deps.py`（Cargo.toml の dependencies を走査）で検査する
- **腐敗防止層 = sim-ffi**: Presentation は Rust の型を直接見ない。7 操作（create/load/step/snapshot/explain/save/destroy）と固定レイアウトのバッファのみ（REQ-CON-01/08、REQ-VIS-04）。バッファ規約: 呼出側がバッファを渡し、容量不足は `required_len` を返す
- **Explain は読み取り専用**: `StateSnapshot` と台帳を入力に取り、SimCore の状態を変えない（REQ-EVT-05）。検出 on/off で state hash が一致することを AT で保証
- **Presentation → SimCore への入力は config と step(n) だけ**。時刻・速度・一時停止は Presentation のスケジューラが持つ（REQ-CON-05）

## 3. 各境界を越えるデータ（契約の所在）
| 境界 | データ | 契約 |
|---|---|---|
| Presentation → sim-ffi | config JSON、seed、step 数、save blob | BD-05 §FFI、`docs/contracts/schema/config.schema.json`、`save.schema.json` |
| sim-ffi → Presentation | StateSnapshot バッファ（固定レイアウト）、explain 出力（JSON）、save blob、エラーコード | BD-05 §FFI、`explanation_contract.md` |
| Calibration → SimCore | config、seed 集合 | 同上 |
| Calibration → docs | manifest（config hash、分布、変更理由） | BD-08（D6/D7 の AT）、`result.schema.json` |
| SimCore → Explain | 台帳（LedgerEntry: tick, cell, lineage, ReasonCode, amount）、ドメインイベント | BD-12 |

## 4. コンテキストごとの決定性責務
| コンテキスト | 決定性の保証 | 検証 |
|---|---|---|
| SimCore | 同一 config/seed → 同一 hash（三経路・クロス OS） | AT-D1/D2/D8（BD-08） |
| Explain | 同一台帳 → 同一イベント列・同一文（テンプレートのみ） | AT-D9 |
| Presentation | 速度・一時停止・中断が hash に影響しない | AT-D12（REQ-DET-07） |
| Calibration | 同一 seed 集合 → 同一 manifest | AT-D7 |

## 5. 未決（ADR で決める）
- sim-explain を独立 crate にするか sim-core 内モジュールにするか → ADR-0007 予定（判断基準: Explain が SimCore の内部型を必要とするか）
- sim-ffi を sim-cli と分けるか → ADR-0008 予定
