# BD-08 受入テスト設計（AT）

- 版: 0.2（起草 cursor-kimi、2026-08-30。追記 cursor-grok: mutation 検出力・AT-D12-ADV・SCH 許容）
- 入力: `docs/10_requirements/要件定義書_検証版_v0.2.md`（sign-off 済）。振る舞いの根拠は BD-03〜BD-07・BD-11 を参照
- 完成条件: **全 P0 REQ に ≥1 AT**（`scripts/gen_trace.py --strict` で `missing_p0_at=0`）。各 AT に REQ と章参照。CI で機械判定できる（USER/INSP 併記の REQ も AT 部分は本表で担保）
- AT-ID: `AT-D<段階>-<連番>`。段階は REQ の「段階」列の最初の実行時点。同一 AT が複数 REQ を兼ねる場合は REQ を併記する。D12 の実行ケースは `AT-D12-UI|SCH|RES|FFI|ADV-*`（BD-11）。`AT-D12-01`〜`11` は REQ カバー用の傘行
- 数値は「確定 / 初期仮説（Dn で確定）」を明記する

## 1. 実行基盤と判定方法

- 実行: `sim-cli verify --suite <Dn>` が本表の AT を段階ごとに実行し、JSON レポート（AT-ID ごとの合否・実測値・evidence）を出力、1 件でも不合格なら非 0 終了（REQ-OPS-01）。CI の `verify` job が全 suite を実行する。参照: REQ-CON-04
- 判定方法の種類（表の「判定」列はこのいずれか、または併記）:
  - **hash 一致**: state hash（BD-05 §10、sha256-v2）のビット一致。同一入力の反復一致と、**committed golden** との一致は別物（§2 AT-D1-06）
  - **schema 検査**: config / save / result schema（BD-05 §13）への適合と `additionalProperties=false` の拒否を判定
  - **保存則**: 総質量の厳密一致（BD-06 §3.1-1、固定小数点で誤差 0）を判定
  - **分布判定**: seed 群の終了ラベル・順位分布を JSON 集計し帯判定（D7 ゲート）
  - **権限検査**: AndroidManifest / 依存リストの機械検査
  - **終了コード・JSON**: verify のプロセス終了とレポート schema
  - **public-api diff**: 公開面のスナップショット一致
  - **画面遷移**: Flutter integration test（AT-D12 系のみ）
  - **台帳集計 / 終了ラベル / ログ**: 上記以外の機械集計
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
| AT-D1-06 | REQ-DET-05, REQ-SIM-04 | 代表 config・seed の 2,000 tick | state hash が committed golden（sha256-v2）とビット一致。ε の Fixed 表現は 100（BD-06 P12） | hash 一致（golden） | BD-05 §10, BD-06 P12 |
| AT-D1-07 | REQ-SIM-04 | §11 の各 MUT-* を 1 件ずつ仕込んだビルドで `verify --suite D1`（D2 検出の MUT は `--suite D2`） | 変異体は必ず不合格。1 件でも合格なら CI 全体 fail | 終了コード・JSON | 本章 §11 |

## 3. D2（64×64・拡散・クロス OS）

| AT-ID | REQ | 入力 | 期待 | 判定 | 参照 |
|---|---|---|---|---|---|
| AT-D2-01 | REQ-SIM-06 | 64×64 閉鎖系、2,000 tick | 総質量が厳密一致 | 保存則 | BD-06 §3.1-1 |
| AT-D2-02 | REQ-SIM-10 | 左右反転 config（拡散係数 0.05/近傍/tick、初期仮説） | 状態が鏡像一致 | hash 一致（鏡像変換後） | BD-05 §2, §4 |
| AT-D2-03 | REQ-DET-03 | 同一入力を CI ubuntu / windows で実行 | state hash が OS 間で一致 | hash 一致 | BD-07 §5 |

D3 は AT なし（REQ-SIM-02/08/11/12 は UT・property で担保）。REQ-DET-03 の Android arm64 は MEAS（実機）。CI 行は ubuntu/windows のみ（AT-D2-03）。

## 4. D4（系統・終了判定）

| AT-ID | REQ | 入力 | 期待 | 判定 | 参照 |
|---|---|---|---|---|---|
| AT-D4-01 | REQ-SCOPE-02 | 4 系統プリセット | アオシキ／シロナミ／アカバエ／クロシデの 4 系統が特徴・代償付きで定義済み | schema 検査 + INSP 補助 | BD-03 §1 |
| AT-D4-02 | REQ-SCOPE-04 | ReasonCode 網羅 fixture 群（各 ReasonCode に到達する最小 config の集合。Diffusion は D2 格子 fixture と共有） | fixture 群の和集合で全 7 種が台帳に 1 回以上出現。単一 seed での全種出現は D4 で代表 seed 選定時に検証し、到達不能なら REQ-SCOPE-04 を RFC 改訂する | 台帳集計 | BD-03 §1.1, BD-05 §3 |
| AT-D4-03 | REQ-SCOPE-05 | 4 系統プリセット | `use_carcass` 系統が 1 以上存在 | schema 検査 | BD-03 §1 |
| AT-D4-04 | REQ-SIM-03b | 空き家発生が確認済みの代表 seed | 空き家条件（occupancy_peak > 0.3 ∧ biomass_sum < ε ∧ nutrient > θ）を満たすセルが 1 回以上出現し、判定が状態を変えない | 観測ログ + hash 一致 | BD-03, BD-05 §2 |
| AT-D4-05 | REQ-END-01 | 終了ラベル enum の schema | Extinct/Fixed/Coexist/Reversal/TimeLimit の 5 種のみ | schema 検査 | BD-04 §1, BD-05 §13 |
| AT-D4-06 | REQ-END-02 | 全系統 < ε（ε = Fixed 100 = 1e-4 × 初期総生体量）に収束する config | Extinct で即終了 | 終了ラベル | BD-04 §1, BD-06 P12 |
| AT-D4-07 | REQ-END-03 | 1 系統が 70% 以上を 200 tick 継続する config（到達 config は D4 の境界値 UT で構成し本 AT が引用。D4 持ち越し） | Fixed で即終了（199 tick では非終了） | 終了ラベル + 境界 | BD-04 §1 |
| AT-D4-08 | REQ-END-04a | 上限 tick 時に 2 系統が各 15% 以上の config（同上: D4 の境界値 UT で構成。D4 持ち越し） | Coexist（上限前は非終了） | 終了ラベル | BD-04 §1 |

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
| AT-D7-06 | REQ-ACC-04, REQ-GOAL-03 | D7 の 100 seed 結果（台帳・終了ラベル・10 tick 平均の時系列） | REQ-GEN-05/06 合格・REQ-END-05 帯内・出来事 3 種が各 1 本以上。機械定義: (a) 優勢枯渇衰退 = シェア ≥50% の系統がピークから 50% 以上減少し、減少区間の主 ReasonCode が Starvation/Maintenance かつ占有 region の nutrient < θ、(b) 死骸・空きニッチ逆転 = Reversal ラベルかつ当該系統の台帳に carcass 由来 Intake または空き家条件セルでの増殖、(c) seed 分岐 = 同条件別 seed 群で Extinct/Fixed/Coexist が各 1 本以上 | 分布判定（ゲート総合） | BD-03, BD-12 |

## 8. D8（保存・転換点）

| AT-ID | REQ | 入力 | 期待 | 判定 | 参照 |
|---|---|---|---|---|---|
| AT-D8-01 | REQ-DET-02 | ランダム tick 数点（CI が seed から決定的に選択）で save→load→残り | 一気実行と最終 hash がビット一致 | hash 一致 | BD-05 §10 |
| AT-D8-02 | REQ-DET-06 | SaveEnvelope の正規・破損・版不一致の各 save | 必須 12 フィールド保持（BD-10 §2）、checksum / schema_version / model_version / state_hash 不一致の 4 経路をそれぞれ専用エラーで拒否 | schema 検査 + エラー検査 | BD-05 §12, §13, BD-10 §2 |
| AT-D8-03 | REQ-EVT-02 | 100 seed の転換点検出 | 検出 ≥1 件の seed が 90 本以上（空転率 ≤10%）かつ保存 32 件到達 seed 0 本（満杯率 0%）（初期仮説。OPEN-04 の D8 較正で確定） | 分布判定 | BD-12 |
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

REQ カバー用の傘行（実装ケースは後表と BD-11）:

| AT-ID | REQ | 入力 | 期待 | 判定 | 参照 |
|---|---|---|---|---|---|
| AT-D12-01 | REQ-SCOPE-03 | 4 レバー（初期個体数・配置・札・ばらつき）の config | 4 レバーが config schema で表現され、範囲外は拒否 | schema 検査 | BD-05 §13 |
| AT-D12-02 | REQ-SCOPE-08, REQ-UI-06 | ひとこと仮説 3 問の回答→終了 | 終了時に 起きた/起きなかった/まだわからない を表示。正解数・ランク・誘導的順序なし | 判定ロジック + INSP 補助 | BD-11 |
| AT-D12-03 | REQ-OUT-02, REQ-OUT-05, REQ-NFR-04, REQ-OPS-05 | APK の AndroidManifest・依存リスト | ネットワーク権限・不要権限なし、生成 AI・外部 solver 依存なし | 権限検査 | BD-01 §4 |
| AT-D12-04 | REQ-CON-05 | コアのシンボル・lint 結果 | wall clock 非使用（lint）、速度変更は UI スケジューラのみ | 権限検査（lint） | BD-07 §4 |
| AT-D12-05 | REQ-DET-07 | 低速 1／標準 4／高速 16 tick/s + 一時停止・中断復帰 | PRNG 消費なし、最終 hash・終了ラベル・イベント列が 3 速度で一致 | hash 一致 | BD-07 §3, BD-11 |
| AT-D12-06 | REQ-DET-08 | 「同 seed 再現」「別 seed 再試行」の 2 経路 | 同 seed は hash 一致、別 seed は区別され UI から両方選べる | hash 一致 + 画面遷移 | BD-11 |
| AT-D12-07 | REQ-UI-01 | 一巡: 仕込む→仮説 3 問→放つ→スタンプ→今どうなってる→生命史カード→もしもレバー→再実験 | この順でオフライン完結（画面遷移テスト） | 画面遷移 | BD-11 §3 |
| AT-D12-08 | REQ-UI-03 | 模擬 30〜120 Hz フレーム列 | 7 要素を独立ケース AT-D12-SCH-01〜07 で検証（本行は索引） | 画面遷移 + ログ | BD-11 §2 |
| AT-D12-09 | REQ-UI-05 | OS 中断・アプリ終了からの復帰（3 速度） | 復帰後の hash が連続（詳細は AT-D12-RES-01〜04） | hash 一致 | BD-11, BD-05 §10 |
| AT-D12-10 | REQ-UI-07 | 生命史カードの保存・一覧 | 必須項目（4 系統開始/終了・終了ラベル・主要転換点・有力原因・初期条件と seed・次に試す変更）を持ちローカル保存 | schema 検査 | BD-11, BD-12 |
| AT-D12-11 | REQ-UI-08 | 前回条件の複製 + 1 項目変更で再開 | 複製 config が一致し変更 1 項目のみ差分、変更項目数を記録 | schema 検査 + ログ | BD-11 |

ID 対応（composer は右列を実装する）:

| 傘 / 索引 | 実行ケース（BD-11） |
|---|---|
| AT-D12-01 | AT-D12-UI-02 |
| AT-D12-02 | AT-D12-UI-03, AT-D12-UI-08 |
| AT-D12-05 | AT-D12-SCH-01 |
| AT-D12-06 | AT-D12-UI-11 |
| AT-D12-07 | AT-D12-UI-01…12 |
| AT-D12-08 | AT-D12-SCH-01…09 |
| AT-D12-09 | AT-D12-RES-01…04 |
| AT-D12-04 の lint | AT-D12-ADV-14 |
| （FFI 境界） | AT-D12-FFI-01…04 |

### 10.1 SCH 許容（BD-11 から委譲。確定）

| AT-ID | 入力 | 期待 | 判定 |
|---|---|---|---|
| AT-D12-SCH-01 | 同一 config・seed、速度 1/4/16 + 途中停止、2,000 tick | 最終 hash・終了ラベル・イベント列がビット一致。PRNG 非消費 | hash 一致 |
| AT-D12-SCH-02 | 模擬 60 Hz 完全周期、標準 4 tick/s、4 秒 | 実行 tick = 16 ± 0（確定） | ログ |
| AT-D12-SCH-03 | 1 Hz フレーム × 高速 16 | 1 フレームの `step` 合計 ≤ 2。`step(16)` 一括は不合格 | ログ |
| AT-D12-SCH-04 | 遅れ 5 tick のあと通常フレーム | 累積実行 tick が目標に追いつく | ログ |
| AT-D12-SCH-05 | 間引きあり/なし | state hash 一致。同一 stamp id の同時二重描画なし | hash 一致 |
| AT-D12-SCH-06 | 人工 200 ms/フレーム × 3 | 3 フレーム後に一段下げ。手動加速で累積遅れを 0 に戻す（確定） | ログ |
| AT-D12-SCH-07 | S-NOW/S-STAMP を 2 秒 | その間 `step` 0。閉じた直後に `step` バーストなし | ログ + hash 一致 |
| AT-D12-SCH-08 | 模擬 30 Hz と 120 Hz、標準 4 tick/s、60 秒 | 実行 tick が 240 に対し ± 2（確定） | ログ |
| AT-D12-SCH-09 | 高速 16、スタンプ多発 seed | 同一 `event_id` の皿上ピンが同時に 2 個以上に増えない | 画面遷移 |

### 10.2 敵対ケース（BD-11 §5。composer 向け粒度）

| AT-ID | REQ | 入力 | 期待 | 判定 | 参照 |
|---|---|---|---|---|---|
| AT-D12-ADV-01 | REQ-UI-01 | S-RELEASE で「放つ」を 10 Hz 連打 | `create` 1 回。第 2 打以降無視。handle 数 = 1 | 画面遷移 + ログ | BD-11 §5 |
| AT-D12-ADV-02 | REQ-DET-07 | S-DISH で速度 16↔1 を 10 Hz 切替、2,000 tick | hash・PRNG 経路が連続運転と一致。各フレームの `step` 合計は要素 3 の上限 | hash 一致 + ログ | BD-11 §2 #3 |
| AT-D12-ADV-03 | REQ-UI-05 | 速度切替と同じフレームで OS `paused` | `save` する。`due` の切替途中値を persist しない。再開は保存 tick から。速度は最後に確定した値 | hash 一致 + ログ | BD-11 §3 |
| AT-D12-ADV-04 | REQ-UI-03 | S-NOW 表示中に ticker 発火 | `step` 0。閉じた直後に `due` バーストなし（基準引き直し） | ログ | BD-11 §2 #7 |
| AT-D12-ADV-05 | REQ-UI-03 | S-STAMP 表示中に別スタンプ連打 | 表示中は `step` しない。`explain` は最新タップ 1 件。古いバッファを描画に使わない | ログ + バッファ検査 | BD-11 §4 |
| AT-D12-ADV-06 | REQ-DET-07 | 高速中にスタンプと今どうなってるを交互 | 重ねが開いた瞬間から凍結。対照ランより開いていた時間分だけ tick が少ない | hash 一致 | BD-11 §2 #7 |
| AT-D12-ADV-07 | REQ-DET-02 | `save` 中にプロセス kill | `load` 成功なら H 連続。失敗なら S-ERR。中途 handle なし | hash 一致 + 画面遷移 | BD-11 §3.2 |
| AT-D12-ADV-08 | REQ-CON-01 | `save` 中に「続きから」を二重起動 | 第 2 起動は第 1 の `load` 完了まで待つ。handle 2 つ禁止 | ログ | BD-11 §5 |
| AT-D12-ADV-09 | REQ-UI-07 | 終了直後・S-CARD 描画前にアプリ終了 | 再起動後、カードまたは再開セーブの一方が完全。両方欠け・両方中途は不合格 | schema 検査 + 画面遷移 | BD-11 §5 |
| AT-D12-ADV-10 | REQ-UI-08 | S-WHATIF で 2 レバーを変えようとする | 第 2 はロック。記録される変更項目数は 0 または 1 | ログ | BD-11 §1 |
| AT-D12-ADV-11 | REQ-DET-08 | S-RETRY で (A) と (B) を連続タップ | `create` は 1 経路だけ。(A) は seed 不変。(B) は seed だけ変更 | hash 一致 + ログ | BD-11 §1 |
| AT-D12-ADV-12 | REQ-UI-01 | 仮説未選択で戻る／OS kill | 再起動は S-BOOT。未 `create` ならセーブ無し | 画面遷移 | BD-11 §1 |
| AT-D12-ADV-13 | REQ-UI-03 | 自動一段下げと手動加速が同フレーム | 手動を採用し累積遅れを 0。そのフレームの `k` は新速度の上限 | ログ | 本章 §10.1 SCH-06 |
| AT-D12-ADV-14 | REQ-CON-05 | 端末時計を +1 h | 単調時計なら `due` 急増なし。wall clock 実装は本 AT で不合格 | ログ | BD-07 §4 |
| AT-D12-ADV-15 | REQ-VIS-04 | `snapshot` バッファを `step` と共有して再利用 | 禁止。コピーアウト後にだけ描画。間引きフレームで古いバッファを新 tick として出さない | バッファ検査 + hash 一致 | BD-11 §4 |
| AT-D12-ADV-16 | REQ-UI-05 | 説明表示中に中断→復帰 | 復帰後も重ねを再開してよい。最初のフレームは凍結。H は中断 tick と一致 | hash 一致 | BD-11 §3 |
| AT-D12-ADV-17 | REQ-UI-07 | S-CARDS から実行中ラン相当を開く | 実行中は一覧に「未完了」を出さない。完了カードのみ | 画面遷移 | BD-11 §1 |
| AT-D12-ADV-18 | REQ-OUT-05 | オフライン・飛行機モードで一巡 | 完了する。送信 API・ネットワーク権限なし | 画面遷移 + 権限検査 | BD-11 §5 |
| AT-D12-ADV-19 | REQ-UI-01 | `explain` 失敗後に再生再開 | handle 維持。失敗では tick が進まない。再開は閉じたあと | ログ + hash 一致 | BD-11 §4 |
| AT-D12-ADV-20 | REQ-UI-05 | 低速で 2,000 tick 中に 50 回中断復帰 | 各復帰で H 連続。最終三要素が連続運転と一致 | hash 一致 | BD-11 §3 |

敵対列 20。UI-01…12 / RES-01…04 / FFI-01…04 の入力・期待は BD-11 §1・§3・§4 を正本とし、本ファイルは索引する。

## 11. gate 自己試験用 mutation 一覧

verify ハーネス自身の検出力を保証する（REQ-SIM-04）。各 MUT を 1 件ずつ仕込んだビルドで、検出 AT が **必ず不合格** になること。同一実行の反復 hash（AT-D1-03）だけでは決定的な変異を落とせない。保存則も位相入替・丸め反転を通す。検出の主軸は **golden hash（AT-D1-06）** と質量漏れ（AT-D1-01）。

| mutation-ID | 変異 | 検出すべき AT | 備考 |
|---|---|---|---|
| MUT-01 | 7 phase の順序入替（intake↔maintenance） | AT-D1-06 | 質量は保存されうる。AT-D1-02 では不足 |
| MUT-02 | 丸め方向の反転（ゼロ方向→負方向） | AT-D1-06 | 決定的なので AT-D1-03 は通る |
| MUT-03 | 走査順の反転（row-major 逆順） | AT-D1-06, AT-D2-02 | D1 1 セルでは無効。D2 と golden で落とす |
| MUT-04 | 拡散の余りを送り元に残さず捨てる | AT-D1-01 | 保存則で検出 |
| MUT-05 | SplitMix64 の 4 ストリーム充填順を movement↔reproduction で入替 | AT-D1-06 | 用途入替だけでは D1 の tick 消費 0 のため hash が変わらない |
| MUT-06 | occupancy phase をスキップ | AT-D1-06 | 質量は保存されうる |
| MUT-07 | i64 加算を wrap（飽和・NumericError にしない） | AT-D1-06 | 代表 config では値が小さいと沈黙しうる。上限近傍 fixture を併用 |
| MUT-08 | 状態更新経路に `f64` を 1 箇所入れる | AT-D1-06, AT-D2-03 | lint だけでは実行時差を保証しない |
| MUT-09 | 余りを主出力ではなく副出力へ返す | AT-D1-01, AT-D1-06 | |
| MUT-10 | hash 正規化から PRNG 4 ストリーム状態を落とす | AT-D1-06 | REQ-DET-05 |
| MUT-11 | `step(n)` 途中終了でも n 回分 PRNG を進める | AT-D1-04, AT-D1-06 | BD-07 §3 |
| MUT-12 | SaveEnvelope の checksum を常に 0 | AT-D8-02 | D1 suite では検出しない |

## 12. trace との整合

- 本ファイルの REQ 参照と AT-ID は `scripts/gen_trace.py` が走査し、`docs/20_design/trace.md` に反映される。CI は「AT の無い P0 要求」「REQ 参照の無い AT」を fail にする（要件定義書 §8）
- `gen_trace.py` の AT 正規表現は `AT-D<digits>-<digits>` のみ。`AT-D12-ADV-01` は現状カウントされない（ハーネス側の持ち越し）
- AT の追加・変更は本ファイルの RFC 改訂を伴う。確定していない期待値は「初期仮説」と明記し、該当 Dn の較正で確定する
