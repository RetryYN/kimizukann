# キミ図鑑（kimizukann）

生命史シミュレーション（Rust コア + Flutter UI）。検証版を AI チームで開発中。

## ディレクトリ
| パス | 内容 | 正本性 |
|---|---|---|
| `docs/00_product/` | 製品企画 v0.3、統合案 v0.5、検証版計画 v1.0 | 企画の正本 |
| `docs/10_requirements/` | 要件定義書（sign-off 済み。変更は RFC） | 要件の正本 |
| `docs/20_design/` | 基本設計 `basic/`、詳細設計 `detail/`、ADR `adr/`、RFC `rfc/`、`trace.md`（自動生成） | 設計の正本 |
| `docs/30_contracts/` | 契約索引 `simulation_contract.md`（本文は `20_design/basic/05_contract.md`）、JSON schema、golden | 実装が従う契約 |
| `docs/40_process/` | GitHub 運用ルール、体制（役割分担・ネットワーク）、ハーネス案、モデル特性 | 進め方の正本 |
| `docs/50_records/` | 開発ログ、brief、レビュー記録、相談会議事録 `meetings/` | 記録（審査不要） |
| `docs/90_archive/` | 旧要件パッケージ、却下案 | 参照のみ。実装入力にしない |
| `crates/` | Rust workspace（sim-types / sim-core / sim-cli。sim-explain / sim-ffi は予定） | — |
| `scripts/` | trace 生成、依存方向チェック等 | CI から呼ぶ |
| `.github/` | workflows、PR テンプレ、CODEOWNERS | — |

## 開発の流れ
要件（10）→ 基本設計（20/basic、AT 設計込み）→ 詳細設計（20/detail、UT 設計込み）→ 実装（crates）。PR 経由のみ、規則は `docs/40_process/GitHub運用ルール.md`。

## ビルド
```
cargo test --manifest-path crates/Cargo.toml --workspace
cargo run --manifest-path crates/sim-cli/Cargo.toml -- verify --suite week1
python scripts/gen_trace.py
```
