# トレース表（自動生成: scripts/gen_trace.py）

- 入力: `docs\10_requirements\要件定義書_検証版_v0.2.md`
- 要求数: 120

| REQ | 優先 | 段階 | 検証 | 設計での参照 | コードでの参照 |
|---|---|---|---|---|---|
| REQ-GOAL-01 | P0 | — | INSP | — | — |
| REQ-GOAL-02 | P0 | D0 | INSP（ADR） | 01_context_map.md | — |
| REQ-GOAL-03 | P0 | D7 | → REQ-ACC-04 | 08_acceptance_tests.md | — |
| REQ-SCOPE-01 | P0 | D5 | AT（schema + 参照検査） | 02_glossary.md, 05_contract.md, 08_acceptance_tests.md | — |
| REQ-SCOPE-02 | P0 | D4 | AT | 02_glossary.md, 08_acceptance_tests.md | — |
| REQ-SCOPE-09 | P0 | D4/D12 | INSP（プリセット定義に型ラベル）+ USER | — | — |
| REQ-SCOPE-03 | P0 | D12 | AT（config schema）+USER | 02_glossary.md, 05_contract.md, 07_determinism_model.md, 08_acceptance_tests.md, 11_ui_flow.md | — |
| REQ-SCOPE-04 | P0 | D4 | AT（ReasonCode 網羅） | 02_glossary.md, 03_domain_model.md, 05_contract.md, 08_acceptance_tests.md | — |
| REQ-SCOPE-05 | P0 | D4 | AT（use_carcass 系統の存在） | 02_glossary.md, 05_contract.md, 08_acceptance_tests.md | — |
| REQ-SCOPE-06 | P0 | D7 | AT | 03_domain_model.md, 05_contract.md, 06_numeric_model.md, 08_acceptance_tests.md | — |
| REQ-SCOPE-07 | P0 | D12 | → REQ-UI-01 | 02_glossary.md | — |
| REQ-SCOPE-08 | P0 | D12 | AT+USER | 02_glossary.md, 08_acceptance_tests.md | — |
| REQ-OUT-01 | P0 | D6 | INSP + AT（系統数不変） | 02_glossary.md, 03_domain_model.md, 05_contract.md, 07_determinism_model.md, 08_acceptance_tests.md | — |
| REQ-OUT-02 | P0 | D12 | INSP + AT（権限なし） | 08_acceptance_tests.md | — |
| REQ-OUT-03 | P0 | — | INSP | 02_glossary.md | — |
| REQ-OUT-04 | P0 | D10 | AT | 01_context_map.md, 03_domain_model.md, 05_contract.md, 07_determinism_model.md, 08_acceptance_tests.md | — |
| REQ-OUT-05 | P0 | D12 | AT（ネット権限なし・依存なし） | 01_context_map.md, 05_contract.md, 08_acceptance_tests.md, 11_ui_flow.md | — |
| REQ-USER-01 | P0 | 配布 | INSP（募集リスト） | — | — |
| REQ-USER-02 | P0 | 配布 | INSP | — | — |
| REQ-USER-03 | P0 | D12 | USER | — | — |
| REQ-USER-04 | P1 | — | INSP | — | — |
| REQ-CON-01 | P0 | D0/D12 | INSP + AT（public-api） | 0004-rust-flutter-android.md, 01_context_map.md, 04_state_machines.md, 05_contract.md, 08_acceptance_tests.md, 11_ui_flow.md | — |
| REQ-CON-08 | P0 | D12 | UT（容量不足・不一致ケース） | 01_context_map.md, 02_glossary.md, 03_domain_model.md, 05_contract.md, 11_ui_flow.md | — |
| REQ-CON-02 | P0 | D0〜 | UT + INSP（clippy: f32/f64 禁止 lint） | 0001-fixed-point-i64.md, 02_glossary.md, 03_domain_model.md, 05_contract.md, 06_numeric_model.md, 07_determinism_model.md | — |
| REQ-CON-03 | P0 | 配布 | INSP | 0004-rust-flutter-android.md | — |
| REQ-CON-04 | P0 | ハーネス | AT（CI） | 0005-toolchain-gnu-ci-primary.md, 08_acceptance_tests.md | — |
| REQ-CON-05 | P0 | D12 | INSP + AT | 01_context_map.md, 03_domain_model.md, 04_state_machines.md, 05_contract.md, 07_determinism_model.md, 08_acceptance_tests.md, 11_ui_flow.md | — |
| REQ-CON-06 | P1 | — | INSP | — | — |
| REQ-CON-07 | P0 | — | INSP | — | — |
| REQ-SIM-01 | P0 | D0 | UT（型）+ INSP | 02_glossary.md, 03_domain_model.md, 05_contract.md, 06_numeric_model.md | — |
| REQ-SIM-02 | P0 | D3 | UT | 02_glossary.md, 03_domain_model.md, 04_state_machines.md, 05_contract.md, 06_numeric_model.md, 08_acceptance_tests.md | — |
| REQ-SIM-03a | P0 | D2 | UT | 02_glossary.md, 03_domain_model.md, 04_state_machines.md, 05_contract.md, 06_numeric_model.md, 07_determinism_model.md | — |
| REQ-SIM-03b | P0 | D4 | UT + AT（空き家が発生する seed が存在） | 02_glossary.md, 03_domain_model.md, 04_state_machines.md, 05_contract.md, 08_acceptance_tests.md | — |
| REQ-SIM-04 | P0 | D1 | UT（順序）+ mutation 自己試験 | 02_glossary.md, 03_domain_model.md, 04_state_machines.md, 05_contract.md, 08_acceptance_tests.md | — |
| REQ-SIM-05 | P0 | D1 | UT（property: 全変換で保存）+ AT | 0001-fixed-point-i64.md, 0006-remainder-to-primary-output.md, 02_glossary.md, 03_domain_model.md, 04_state_machines.md, 05_contract.md, 06_numeric_model.md, 08_acceptance_tests.md | — |
| REQ-SIM-06 | P0 | D1/D2 | AT（1 セル・64×64） | 0006-remainder-to-primary-output.md, 02_glossary.md, 03_domain_model.md, 05_contract.md, 06_numeric_model.md, 08_acceptance_tests.md | — |
| REQ-SIM-07 | P0 | D5 | AT | 02_glossary.md, 03_domain_model.md, 05_contract.md, 06_numeric_model.md, 07_determinism_model.md, 08_acceptance_tests.md | — |
| REQ-SIM-08 | P0 | D1/D3 | UT + property（0 ≤ energy ≤ 1） | 02_glossary.md, 03_domain_model.md, 04_state_machines.md, 05_contract.md, 06_numeric_model.md | — |
| REQ-SIM-09 | P0 | D1 | UT | 02_glossary.md, 04_state_machines.md, 07_determinism_model.md | — |
| REQ-SIM-10 | P0 | D2 | AT（対称性）+ UT | 02_glossary.md, 03_domain_model.md, 04_state_machines.md, 05_contract.md, 06_numeric_model.md, 07_determinism_model.md, 08_acceptance_tests.md | — |
| REQ-SIM-11 | P0 | D3 | UT | 02_glossary.md, 03_domain_model.md, 05_contract.md, 06_numeric_model.md, 07_determinism_model.md | — |
| REQ-SIM-12 | P0 | D3 | UT | 02_glossary.md, 03_domain_model.md, 04_state_machines.md, 05_contract.md, 06_numeric_model.md, 07_determinism_model.md | — |
| REQ-SIM-13 | P0 | D1 | property + UT（上限・上限+1） | 02_glossary.md, 03_domain_model.md, 05_contract.md, 06_numeric_model.md | — |
| REQ-SIM-14 | P0 | D2/D6 | INSP（numeric-model）+ AT（D6 煙試験で panic 0） | 03_domain_model.md, 06_numeric_model.md, 08_acceptance_tests.md | — |
| REQ-GEN-01 | P0 | D0 | AT（config schema additionalProperties=false） | 02_glossary.md, 03_domain_model.md, 05_contract.md, 06_numeric_model.md, 08_acceptance_tests.md | — |
| REQ-GEN-02 | P0 | D12 | INSP + USER | 02_glossary.md, 05_contract.md, 11_ui_flow.md | — |
| REQ-GEN-03 | P0 | D4 | INSP（config golden） | 02_glossary.md | — |
| REQ-GEN-04 | P0 | D4 | UT（プリセット検査） | 06_numeric_model.md | — |
| REQ-GEN-05 | P0 | D7 | AT（D7 分布判定） | 08_acceptance_tests.md | — |
| REQ-GEN-06 | P0 | D7 | AT | 02_glossary.md, 08_acceptance_tests.md | — |
| REQ-GEN-07 | P1 | D7 | AT | — | — |
| REQ-GEN-08 | P1 | D4/D12 | UT（上限・上限+1）+ INSP | 02_glossary.md, 03_domain_model.md, 05_contract.md, 07_determinism_model.md, 11_ui_flow.md | — |
| REQ-ENV-01 | P0 | D5 | AT（schema） | 02_glossary.md, 05_contract.md, 08_acceptance_tests.md | — |
| REQ-ENV-02 | P0 | D5 | AT（4 config が schema 通過、load→save→load 同一） | 02_glossary.md, 08_acceptance_tests.md, 11_ui_flow.md | — |
| REQ-ENV-03 | P0 | D5 | UT | 02_glossary.md | — |
| REQ-ENV-04 | P1 | D5/D7 | AT（schema） | — | — |
| REQ-DET-01 | P0 | D1 | AT | 0003-state-hash-sha256.md, 07_determinism_model.md, 08_acceptance_tests.md | — |
| REQ-DET-02 | P0 | D1/D8 | AT（代表 tick + ランダム tick 数点） | 01_context_map.md, 02_glossary.md, 03_domain_model.md, 04_state_machines.md, 05_contract.md, 07_determinism_model.md, 08_acceptance_tests.md, 11_ui_flow.md | — |
| REQ-DET-03 | P0 | D2/D12 | AT（CI xos）+ MEAS（実機） | 0001-fixed-point-i64.md, 0005-toolchain-gnu-ci-primary.md, 07_determinism_model.md, 08_acceptance_tests.md | — |
| REQ-DET-04a | P0 | D0 | UT（既知ベクトル） | 0002-prng-xoshiro256ss.md, 02_glossary.md, 03_domain_model.md, 05_contract.md, 07_determinism_model.md | — |
| REQ-DET-04b | P0 | D2 | UT（消費回数カウント） | 02_glossary.md, 05_contract.md, 07_determinism_model.md | — |
| REQ-DET-04c | P0 | D2 | UT + AT（三経路） | 02_glossary.md, 03_domain_model.md, 05_contract.md, 07_determinism_model.md, 08_acceptance_tests.md | — |
| REQ-DET-04d | P0 | D0〜 | INSP（clippy disallowed_types / disallowed_methods） | 07_determinism_model.md | — |
| REQ-DET-05 | P0 | D1 | UT（golden） | 0002-prng-xoshiro256ss.md, 02_glossary.md, 03_domain_model.md, 05_contract.md, 06_numeric_model.md, 07_determinism_model.md, 08_acceptance_tests.md | — |
| REQ-DET-06 | P0 | D8 | AT（schema）+ UT | 02_glossary.md, 03_domain_model.md, 04_state_machines.md, 05_contract.md, 08_acceptance_tests.md, 11_ui_flow.md | — |
| REQ-DET-07 | P0 | D12 | AT | 03_domain_model.md, 05_contract.md, 07_determinism_model.md, 08_acceptance_tests.md, 11_ui_flow.md | — |
| REQ-DET-08 | P0 | D12 | AT + USER | 08_acceptance_tests.md, 11_ui_flow.md | — |
| REQ-DET-09 | P0 | D11 | AT（reference_scenarios の model_version 照合） | 02_glossary.md, 05_contract.md, 08_acceptance_tests.md | — |
| REQ-END-01 | P0 | D4 | AT（schema enum） | 02_glossary.md, 03_domain_model.md, 04_state_machines.md, 05_contract.md, 08_acceptance_tests.md | — |
| REQ-END-02 | P0 | D4 | UT + AT | 02_glossary.md, 03_domain_model.md, 04_state_machines.md, 05_contract.md, 06_numeric_model.md, 08_acceptance_tests.md | — |
| REQ-END-03 | P0 | D4 | UT + AT | 01_context_map.md, 02_glossary.md, 03_domain_model.md, 04_state_machines.md, 05_contract.md, 08_acceptance_tests.md | — |
| REQ-END-04a | P0 | D4 | UT（最小例）+ AT | 02_glossary.md, 03_domain_model.md, 04_state_machines.md, 05_contract.md, 08_acceptance_tests.md | — |
| REQ-END-04b | P0 | D4 | UT（同率ケース含む） | 02_glossary.md, 04_state_machines.md, 05_contract.md | — |
| REQ-END-04c | P0 | D4 | UT（同時成立ケース） | 02_glossary.md, 03_domain_model.md, 04_state_machines.md, 05_contract.md, 07_determinism_model.md | — |
| REQ-END-05 | P0 | D7 | AT（D7 分布） | 08_acceptance_tests.md | — |
| REQ-EVT-02 | P0 | D8 | UT + AT（100 seed で空転・満杯なし） | 02_glossary.md, 08_acceptance_tests.md | — |
| REQ-EVT-03 | P0 | D8 | UT | 02_glossary.md | — |
| REQ-EVT-04 | P0 | D8 | UT + MEAS（保存 ≤ 5 MB） | 01_context_map.md, 02_glossary.md, 03_domain_model.md, 05_contract.md | — |
| REQ-EVT-05 | P0 | D8 | AT（検出 off/on で hash 一致） | 03_domain_model.md, 07_determinism_model.md, 08_acceptance_tests.md | — |
| REQ-EXP-01 | P0 | D9 | UT（出力構造）+ USER | 04_state_machines.md, 05_contract.md, 08_acceptance_tests.md | — |
| REQ-EXP-02 | P0 | D9/D12 | INSP + USER（質問 3） | — | — |
| REQ-EXP-03 | P0 | D9 | INSP + UT | 02_glossary.md, 05_contract.md | — |
| REQ-EXP-04 | P0 | D9 | UT | 02_glossary.md, 11_ui_flow.md | — |
| REQ-EXP-05 | P0 | D9/D12 | UT + USER | 04_state_machines.md, 05_contract.md | — |
| REQ-EXP-06 | P1 | D9 | INSP（文言リント） | 11_ui_flow.md | — |
| REQ-UI-01 | P0 | D12 | USER + AT（画面遷移テスト） | 02_glossary.md, 08_acceptance_tests.md, 11_ui_flow.md, README.md | — |
| REQ-UI-02 | P0 | D12 | INSP（文言チェックリスト） | 11_ui_flow.md | — |
| REQ-UI-03 | P0 | D12 | AT（模擬 30〜120 Hz フレーム列。7 要素それぞれを独立ケースにする） | 01_context_map.md, 04_state_machines.md, 05_contract.md, 08_acceptance_tests.md, 11_ui_flow.md, README.md | — |
| REQ-UI-04a | P0 | D12 | MEAS（実機） | 11_ui_flow.md | — |
| REQ-UI-04b | P0 | D12 | MEAS（実機） | 11_ui_flow.md | — |
| REQ-UI-05 | P0 | D12 | AT | 08_acceptance_tests.md, 11_ui_flow.md | — |
| REQ-UI-06 | P0 | D12 | UT（判定）+ INSP | 02_glossary.md, 08_acceptance_tests.md, 11_ui_flow.md | — |
| REQ-UI-07 | P0 | D12 | AT（schema）+ USER | 02_glossary.md, 08_acceptance_tests.md, 11_ui_flow.md | — |
| REQ-UI-08 | P0 | D12 | AT + USER | 08_acceptance_tests.md, 11_ui_flow.md | — |
| REQ-UI-09 | P1 | D12 | UT + INSP | 11_ui_flow.md | — |
| REQ-VIS-01 | P0 | D10 | INSP + USER（質問 1） | 05_contract.md | — |
| REQ-VIS-02 | P0 | D12 | USER（質問 1 で 5/8 以上） | — | — |
| REQ-VIS-03 | P1 | D10 | AT（REQ-OUT-04）+ INSP | 07_determinism_model.md, 08_acceptance_tests.md | — |
| REQ-VIS-04 | P0 | D10 | INSP + AT（非干渉 hash） | 01_context_map.md, 02_glossary.md, 04_state_machines.md, 05_contract.md, 07_determinism_model.md, 08_acceptance_tests.md, 11_ui_flow.md | — |
| REQ-OPS-01 | P0 | D1〜 | AT | 02_glossary.md, 03_domain_model.md, 05_contract.md, 07_determinism_model.md, 08_acceptance_tests.md | — |
| REQ-OPS-02a | P0 | D6 | AT | 08_acceptance_tests.md | — |
| REQ-OPS-02b | P0 | D7 | AT | 08_acceptance_tests.md | — |
| REQ-OPS-03 | P0 | D7 | INSP（較正ログ） | — | — |
| REQ-OPS-04 | P0 | — | INSP（カンバン） | — | — |
| REQ-OPS-05 | P0 | 配布 | INSP + AT（権限リスト） | 0004-rust-flutter-android.md, 01_context_map.md, 08_acceptance_tests.md | — |
| REQ-OPS-06 | P0 | 配布 | INSP | — | — |
| REQ-OPS-07 | P1 | 配布 | INSP | — | — |
| REQ-NFR-01 | P1 | D2/D4 | MEAS（criterion、CI でしきい値） | 06_numeric_model.md, README.md | — |
| REQ-NFR-02 | P1 | D12 | MEAS（実機） | 03_domain_model.md, 05_contract.md | — |
| REQ-NFR-03 | P2 | D7 | MEAS | — | — |
| REQ-NFR-04 | P0 | D12 | AT（manifest 権限） | 01_context_map.md, 08_acceptance_tests.md | — |
| REQ-NFR-05 | P0 | D12 | INSP | — | — |
| REQ-NFR-06 | P0 | D8〜 | AT（旧 save 読込） | 02_glossary.md, 03_domain_model.md, 05_contract.md, 08_acceptance_tests.md | — |
| REQ-NFR-07 | P0 | 全段階 | AT（REQ-SIM-05/06, GEN-04）+ INSP（文言） | 08_acceptance_tests.md | — |
| REQ-NFR-08 | P0 | 配布 | INSP（禁止語リント） | — | — |
| REQ-ACC-01 | P0 | 配布 | USER | — | — |
| REQ-ACC-02 | P0 | 配布 | USER | — | — |
| REQ-ACC-03 | P0 | 配布 | USER | — | — |
| REQ-ACC-04 | P0 | D7 | AT | 08_acceptance_tests.md | — |
| REQ-ACC-05 | P0 | D11 | AT + INSP | 02_glossary.md, 08_acceptance_tests.md | — |

## 未着手（P0 かつ AT 指定なのに設計参照なし）: 0

