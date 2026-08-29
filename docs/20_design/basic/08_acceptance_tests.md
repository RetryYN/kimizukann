# BD-08 受入テスト設計（AT）

- 版: 0.1（起草 cursor-kimi、2026-08-30）
- 入力: `docs/10_requirements/要件定義書_検証版_v0.2.md`（sign-off 済）。振る舞いの根拠は BD-03〜BD-07・BD-11 を参照
- 完成条件: **全 P0 REQ に ≥1 AT**（`scripts/gen_trace.py --strict` で `missing_p0_at=0`）。各 AT に REQ と章参照。CI で機械判定できる（USER/INSP 併記の REQ も AT 部分は本表で担保）
- AT-ID: `AT-D<段階>-<連番>`。段階は REQ の「段階」列の最初の実行時点。同一 AT が複数 REQ を兼ねる場合は REQ を併記する
- 数値は「確定 / 初期仮説（Dn で確定）」を明記する

## 1. 実行基盤と判定方法

- 実行: `sim-cli verify --suite <Dn>` が本表の AT を段階ごとに実行し、JSON レポート（AT-ID ごとの合否・実測値・evidence）を出力、1 件でも不合格なら非 0 終了（REQ-OPS-01）。CI の `verify` job が全 suite を実行する。参照: REQ-CON-04
- 判定方法の種類:
  - **hash 一致**: state hash（BD-05 §10、sha256-v2）のビット一致で判定
  - **schema 検査**: config / save / result schema（BD-05 §13）への適合と `additionalProperties=false` の拒否を判定
  - **保存則**: 総質量の厳密一致（BD-06 §3.1-1、固定小数点で誤差 0）を判定
  - **分布判定**: seed 群の終了ラベル・順位分布を JSON 集計し帯判定（D7 ゲート）
  - **権限検査**: AndroidManifest / 依存リストの機械検査
- AT は全て headless（sim-cli / sim-core 経由）で完結させ、UI 実装の有無に依存しない。UI 経由が本質的な AT-D12 系のみ画面遷移テスト（Flutter integration test）で実施する。参照: REQ-UI-01

## 2. D0〜D1（基盤・1 セル）

| AT-ID | REQ | 入力 | 期待 | 判定 | 参照 |
|---|---|---|---|---|---|
| AT-D0-01 | REQ-GEN-01 | 4 系統プリセット config + 余分キー入り config | 正規 config は schema 通過、余分キーは拒否 | schema 検査 | BD-05 §13 |
| AT-D0-02 | REQ-CON-01 | `cargo public-api` 出力 | 公開型・FFI 7 操作が BD-05 §11/§12 と一致（CI で常時） | public-api diff | BD-05 §11, §12 |
| AT-D1-01 | REQ-SIM-05, REQ-NFR-07 | 1 セル閉鎖系、代表 config | 全変換で Σ出力 + 余り = 入力、余りは主出力へ（質量収支を恣意的に破らないことの機械検証を兼ねる） | 保存則 | BD-03 INV-01, BD-05 §4 |
| AT-D1-02 | REQ-SIM-06, REQ-NFR-07 | 1 セル閉鎖系、2,000 tick | 総質量が厳密一致 | 保存則 | BD-06 §3.1-1 |
| AT-D1-03 | REQ-DET-01 | 同一 config・seed で 100 回実行 | state hash が 100 回一致 | hash 一致 | BD-07 §1〜§3 |
| AT-D1-04 | REQ-DET-02, REQ-DET-04c | `step(2000)` / `step(1)×2000` / tick=1,000 で save→load→残り | 3 経路の最終 hash がビット一致 | hash 一致 | BD-05 §10, BD-07 §5 |
| AT-D1-05 | REQ-OPS-01 | `sim-cli verify --suite D1` を故意に壊したビルドでも実行 | JSON レポート出力 + 合格時 0 / 不合格時非 0 終了 | 終了コード・JSON | 本章 §1 |

## 3. D2（64×64・拡散・クロス OS）

| AT-ID | REQ | 入力 | 期待 | 判定 | 参照 |
|---|---|---|---|---|---|
| AT-D2-01 | REQ-SIM-06 | 64×64 閉鎖系、2,000 tick | 総質量が厳密一致 | 保存則 | BD-06 §3.1-1 |
| AT-D2-02 | REQ-SIM-10 | 左右反転 config（拡散係数 0.05/近傍/tick、初期仮説） | 状態が鏡像一致 | hash 一致（鏡像変換後） | BD-05 §2, §4 |
| AT-D2-03 | REQ-DET-03 | 同一入力を CI ubuntu / windows で実行 | state hash が OS 間で一致 | hash 一致 | BD-07 §5 |

D3 は AT なし（REQ-SIM-02/08/11/12 は UT・property で担保）。

## 4. D4（系統・終了判定）

| AT-ID | REQ | 入力 | 期待 | 判定 | 参照 |
|---|---|---|---|---|---|
| AT-D4-01 | REQ-SCOPE-02 | 4 系統プリセット | アオシキ／シロナミ／アカバエ／クロシデの 4 系統が特徴・代償付きで定義済み | schema 検査 + INSP 補助 | BD-03 §1 |
| AT-D4-02 | REQ-SCOPE-04 | 代表 seed の 2,000 tick | ReasonCode 全 7 種が台帳に 1 回以上出現 | 台帳集計 | BD-03 §1.1, BD-05 §3 |
| AT-D4-03 | REQ-SCOPE-05 | 4 系統プリセット | `use_carcass` 系統が 1 以上存在 | schema 検査 | BD-03 §1 |
| AT-D4-04 | REQ-SIM-03b | 空き家発生が確認済みの代表 seed | 空き家条件（occupancy_peak > 0.3 ∧ biomass_sum < ε ∧ nutrient > θ）を満たすセルが 1 回以上出現し、判定が状態を変えない | 観測ログ + hash 一致 | BD-03, BD-05 §2 |
| AT-D4-05 | REQ-END-01 | 終了ラベル enum の schema | Extinct/Fixed/Coexist/Reversal/TimeLimit の 5 種のみ | schema 検査 | BD-04 §1, BD-05 §13 |
| AT-D4-06 | REQ-END-02 | 全系統 < ε（ε = 1e-4 × 初期総生体量）に収束する config | Extinct で即終了 | 終了ラベル | BD-04 §1, BD-06 P12 |
| AT-D4-07 | REQ-END-03 | 1 系統が 70% 以上を 200 tick 継続する config | Fixed で即終了（199 tick では非終了） | 終了ラベル + 境界 | BD-04 §1 |
| AT-D4-08 | REQ-END-04a | 上限 tick 時に 2 系統が各 15% 以上の config | Coexist（上限前は非終了） | 終了ラベル | BD-04 §1 |

## 5. D5（4 環境・流入）

| AT-ID | REQ | 入力 | 期待 | 判定 | 参照 |
|---|---|---|---|---|---|
| AT-D5-01 | REQ-SCOPE-01 | チュートリアル config | `environment_id` が 4 環境 enum の 1 つを参照し、 dangling 参照は拒否 | schema 検査 + 参照検査 | BD-05 §13 |
| AT-D5-02 | REQ-SIM-07 | 流入あり系（InflowEvent 付き）、2,000 tick | 総質量 = 初期総量 + Σ inflow.amount に厳密一致 | 保存則 | BD-05 §2, BD-06 §3.1-1 |
| AT-D5-03 | REQ-ENV-01 | 環境レコード 4 件 | 必須 7 フィールドを持ち schema 通過 | schema 検査 | BD-05 §13 |
| AT-D5-04 | REQ-ENV-02 | center_rich / edge_sparse / local_waste / carcass_pulse の 4 config | 全件 schema 通過、load→save→load で同一（hash 一致） | schema 検査 + hash 一致 | BD-05 §10, §13 |

## 6. D6（煙試験）

| AT-ID | REQ | 入力 | 期待 | 判定 | 参照 |
|---|---|---|---|---|---|
| AT-D6-01 | REQ-OUT-01 | 全検証 seed の実行結果 | 系統数が初期値（4）を超えない | 台帳・状態集計 | BD-03 INV |
| AT-D6-02 | REQ-OPS-02a, REQ-SIM-14 | 4 環境 × 20 seed、各 2,000 tick | panic・NumericError 0 件で分布 JSON を返す | 終了コード + JSON | BD-06 §3 |

## 7. D7（較正ゲート）

| AT-ID | REQ | 入力 | 期待 | 判定 | 参照 |
|---|---|---|---|---|---|
| AT-D7-01 | REQ-SCOPE-06 | 4 環境 × 100 seed | 2,000 tick 以内に複数の終了ラベルが出現 | 分布判定 | BD-04 §1 |
| AT-D7-02 | REQ-GEN-05 | 4 環境 × 100 seed の終端生体量 | (a) 3 環境以上で中央値 1 位の系統なし、(b) 全環境 2 位以内かつ固定率 1 位なし | 分布判定 | BD-03 |
| AT-D7-03 | REQ-GEN-06 | 同上 | 各系統が 1 位になる環境が 1 つ以上 | 分布判定 | BD-03 |
| AT-D7-04 | REQ-END-05 | 100 seed × center_rich | A 型（共存）≥10%、B 型（アオシキ中央固定）5〜20%、C 型（全滅）≥5%（初期仮説） | 分布判定 | BD-04 §1 |
| AT-D7-05 | REQ-OPS-02b | D7 較正実行 | AT-D7-02/03/04 を機械判定し manifest に config hash・分布・変更理由を保存 | JSON + manifest 検査 | 本章 §1 |
| AT-D7-06 | REQ-ACC-04, REQ-GOAL-03 | D7 の 100 seed 結果 | REQ-GEN-05/06 合格・REQ-END-05 帯内・3 種の出来事（優勢枯渇衰退／死骸・空きニッチ逆転／seed 分岐）が各 1 本以上 | 分布判定（ゲート総合） | BD-03, BD-12 |

## 8. D8（保存・転換点）

| AT-ID | REQ | 入力 | 期待 | 判定 | 参照 |
|---|---|---|---|---|---|
| AT-D8-01 | REQ-DET-02 | ランダム tick 数点（CI が seed から決定的に選択）で save→load→残り | 一気実行と最終 hash がビット一致 | hash 一致 | BD-05 §10 |
| AT-D8-02 | REQ-DET-06 | SaveEnvelope の正規・破損・版不一致の各 save | 必須 7 フィールド保持、checksum/schema_version/model_version 不一致をエラー | schema 検査 + エラー検査 | BD-05 §12, §13 |
| AT-D8-03 | REQ-EVT-02 | 100 seed の転換点検出 | 検出 0 件（空転）でも保存上限超過（満杯）でもない seed 分布 | 分布判定 | BD-12 |
| AT-D8-04 | REQ-EVT-05 | 同一 seed を検出 on / off で実行 | state hash が一致（検出は表示専用） | hash 一致 | BD-07 §2, BD-12 |
| AT-D8-05 | REQ-NFR-06 | 旧 schema_version の save | 移行規則どおり読込またはエラー（migration 試験） | schema 検査 | BD-10 |

## 9. D9〜D11

D9（説明器）は AT なし（REQ-EXP-01〜06 は UT・INSP・USER で担保）。

| AT-ID | REQ | 入力 | 期待 | 判定 | 参照 |
|---|---|---|---|---|---|
| AT-D10-01 | REQ-OUT-04, REQ-VIS-03 | 表示用トークンを消した同一 seed 実行 | 終了ラベル・state hash がビット一致 | hash 一致 | BD-07 §2 |
| AT-D10-02 | REQ-VIS-04 | 描画間引きあり/なしの同一 seed 実行 | スナップショットが固定レイアウトで、間引きがリプレイ経路の hash に非干渉 | hash 一致 + バッファ検査 | BD-05 §12.4 |
| AT-D11-01 | REQ-DET-09 | model_version を bump したビルド | reference_scenarios の model_version 不一致でゲート失敗 → チュートリアル seed 再選定を強制 | JSON + 終了コード | 本章 §1 |
| AT-D11-02 | REQ-ACC-05 | チュートリアル seed の一巡 | 主要スタンプ ≥1 件・10 分以内・`evidence_refs` 非空・最小描画で転換点を視認可能 | JSON + INSP 補助 | BD-11, BD-12 |

## 10. D12（一巡 UI・配布前）

| AT-ID | REQ | 入力 | 期待 | 判定 | 参照 |
|---|---|---|---|---|---|
| AT-D12-01 | REQ-SCOPE-03 | 4 レバー（初期個体数・配置・札・ばらつき）の config | 4 レバーが config schema で表現され、範囲外は拒否 | schema 検査 | BD-05 §13 |
| AT-D12-02 | REQ-SCOPE-08, REQ-UI-06 | ひとこと仮説 3 問の回答→終了 | 終了時に 起きた/起きなかった/まだわからない を表示。正解数・ランク・誘導的順序なし | 判定ロジック + INSP 補助 | BD-11 |
| AT-D12-03 | REQ-OUT-02, REQ-OUT-05, REQ-NFR-04, REQ-OPS-05 | APK の AndroidManifest・依存リスト | ネットワーク権限・不要権限なし、生成 AI・外部 solver 依存なし | 権限検査 | BD-01 §4 |
| AT-D12-04 | REQ-CON-05 | コアのシンボル・lint 結果 | wall clock 非使用（lint）、速度変更は UI スケジューラのみ | 権限検査（lint） | BD-07 §4 |
| AT-D12-05 | REQ-DET-07 | 低速 1／標準 4／高速 16 tick/s + 一時停止・中断復帰 | PRNG 消費なし、最終 hash・終了ラベル・イベント列が 3 速度で一致 | hash 一致 | BD-07 §3, BD-11 |
| AT-D12-06 | REQ-DET-08 | 「同 seed 再現」「別 seed 再試行」の 2 経路 | 同 seed は hash 一致、別 seed は区別され UI から両方選べる | hash 一致 + 画面遷移 | BD-11 |
| AT-D12-07 | REQ-UI-01 | 一巡: 仕込む→仮説 3 問→放つ→スタンプ→今どうなってる→生命史カード→もしもレバー→再実験 | この順でオフライン完結（画面遷移テスト） | 画面遷移 | BD-11 §3 |
| AT-D12-08 | REQ-UI-03 | 模擬 30〜120 Hz フレーム列 | 7 要素（固定時間刻み／フレーム上限 1/1/2／遅れ持ち越し／描画間引き・tick 不捨棄／500 ms で自動一段下げ／中断中は進めない）を独立ケースで検証 | 画面遷移 + ログ | BD-11 §3.5 |
| AT-D12-09 | REQ-UI-05 | OS 中断・アプリ終了からの復帰（3 速度） | 復帰後の hash が連続 | hash 一致 | BD-11, BD-05 §10 |
| AT-D12-10 | REQ-UI-07 | 生命史カードの保存・一覧 | 必須項目（4 系統開始/終了・終了ラベル・主要転換点・有力原因・初期条件と seed・次に試す変更）を持ちローカル保存 | schema 検査 | BD-11, BD-12 |
| AT-D12-11 | REQ-UI-08 | 前回条件の複製 + 1 項目変更で再開 | 複製 config が一致し変更 1 項目のみ差分、変更項目数を記録 | schema 検査 + ログ | BD-11 |

D12 の敵対・境界ケース一覧（AT-D12-* 拡充分）は grok が BD-11 敵対列から追記する。

## 11. gate 自己試験用 mutation 一覧（grok 追記）

verify ハーネス自身の検出力を保証するため、既知の壊れ方を仕込んだ変異体が必ず不合格になることを CI で確認する（REQ-SIM-04 の mutation 自己試験）。初期案（各 1 行、確定は grok）:

| mutation-ID | 変異 | 検出すべき AT |
|---|---|---|
| MUT-01 | 7 phase の順序入替（intake↔maintenance） | AT-D1-02, AT-D2-01 |
| MUT-02 | 丸め方向の反転（ゼロ方向→負方向） | AT-D1-03 |
| MUT-03 | 走査順の反転（row-major 逆順） | AT-D1-03, AT-D2-02 |
| MUT-04 | 拡散の余りを送り元に残さず捨てる | AT-D1-01 |
| MUT-05 | PRNG ストリームの取り違え（movement↔reproduction） | AT-D1-03 |

## 12. trace との整合

- 本ファイルの REQ 参照と AT-ID は `scripts/gen_trace.py` が走査し、`docs/20_design/trace.md` に反映される。CI は「AT の無い P0 要求」「REQ 参照の無い AT」を fail にする（要件定義書 §8）
- AT の追加・変更は本ファイルの RFC 改訂を伴う。確定していない期待値は「初期仮説」と明記し、該当 Dn の較正で確定する
