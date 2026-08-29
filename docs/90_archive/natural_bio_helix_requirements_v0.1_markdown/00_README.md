# NATURAL BIO（仮称） HELIX要件パッケージ v0.1.0 — Markdown版

基準日: 2026-07-23

## 内容

- `NATURAL_BIO_HELIX_企画要求要件定義_統合版.md`: 人間向け企画書・要求定義書・要件定義書の統合Markdown
- `machine/helix_product_manifest.json`: HELIX製品マニフェスト
- `machine/source_registry.json`: 会話要求源台帳
- `machine/requirements_catalog.json`: 完全要求カタログ
- `machine/traceability_matrix.json`: ソース↔要求追跡
- `machine/release_stages.json`: 段階リリース
- `machine/reference_scenarios.json`: 基準シナリオ
- `machine/simulation_core_model.json`: コア概念モデル
- `machine/mcp_tool_contracts.json`: MCP操作契約
- `machine/validation_report.json`: 漏れ・重複・受入条件検証
- `machine/package_inventory.json`: 配布物のサイズ・SHA-256台帳
- `schemas/requirements_catalog.schema.json`: 要求カタログschema
- `tools/validate_package.py`: 再検証ツール

## 検証結果

- 会話要求源: 168件
- 要求: 232件
- 基準シナリオ: 10件
- MCP契約: 10件
- Markdown構文: Pandoc parse PASS
- 検証状態: PASS

## 正本関係

人間向け説明の正本はルートのMarkdown、実装・HELIX接続の正本は `machine/*.json` とする。要求変更時は source registry、requirements catalog、traceability、release assignment、acceptance criteriaを同一変更で更新する。
