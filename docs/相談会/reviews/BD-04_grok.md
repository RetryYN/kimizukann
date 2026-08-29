# [BD-04][review] reviewer=cursor-grok（未定義遷移・ガード漏れ・tick 跨ぎ）

- 対象: `docs/design/basic/04_state_machines.md` v0.1（kimi）
- 観点: 表の穴、ガードの抜け、`step(n)` と phase をまたぐ曖昧さ。数値の妥当性には踏み込まない
- 判定: **changes_requested**

完成条件「全状態×全イベントが埋まり、生成テストが書ける」は、記法 `—` と生成テストの Err 期待が矛盾しており、未達。

---

## 未定義遷移

### U-01 `—` と生成テストの Err が食い違う

§記法: `—` = 到達不能、`Err` = 拒否して状態不変。§1.3 は「`—` / `Err` セルは必ず Err で状態不変」。fuzzer が Destroyed に `create` / 二重 `destroy` / 既存 Run に `load` を送ると、表は到達不能、テストは Err。実装は無視・panic・新規 handle のどれでも仕様を満たせる。

期待: 到達不能でも ABI に届く入力は全部 `Err`（状態不変）。`—` は使わないか、「テスト対象外（コンパイラが排除）」と切り分ける。

参照: REQ-CON-01

### U-02 既存 Run への `load` が `—`

UI は S-BOOT で `load` しうるし、実行中に二重起動しうる（BD-11 ADV-08）。「常に新規 Run」（ADR 候補 1）なら、既存 handle 側の遷移が無い。旧 handle は生きたままか、暗黙 destroy か、Err か。表は到達不能として消している。

期待: 既存 Prepared/Running/Terminated への `load` は `Err`（旧状態不変）または `destroy` 相当の明示遷移。新規 Run の初期状態は別行「Ø → load → Prepared|Running」を表に書く。

参照: REQ-CON-01, REQ-DET-06, REQ-UI-05

### U-03 `Destroyed` のあと `create` が `—`

同一表のままでは Destroyed は吸収状態で、destroy→create の一巡（BD-11）が表現できない。「インスタンスを捨てて Ø から create」と書かないと、use-after-destroy と新規 create が同じ `—` になる。

参照: REQ-CON-01

### U-04 `load` の初期状態が表の外

脚注は tick=0 なら Prepared、tick≥1 なら Running。終了済みセーブ（Terminated）を load した行が無い。終了カード再表示・再開拒否が書けない。`termination_detected` を load 直後に走らせるかも未定義。

参照: REQ-DET-06, REQ-END-01, REQ-UI-07

### U-05 `step(0)` と負の `n`

列は `step(n)` だけ。`n=0` は Prepared のままか、T1 だけ走らせて tick 0 Extinct か。`n` が 0 または過大（残り tick を超える）のガードが無い。

参照: REQ-CON-05, REQ-END-04a

---

## ガード漏れ

### G-01 `termination_detected` と T1 の二重経路

`step` 内 T1 が Terminated へ直遷する一方、独立イベント `termination_detected` が Prepared/Running からも Terminated へ行ける。誰が step 外でこのイベントを発行するかが無い。二重発行でラベルが書き換わるか、Prepared で step 前に Extinct するかがテスト不能。

期待: 終了判定は T1 のみ。内部イベント列は削除するか、「step の 7 phase 直後、同一呼出内」とガードする。

参照: REQ-END-02, REQ-END-04c

### G-02 Occupied 行の「θ_occ > ε だから Vacant 不能」

Vacant 表は Occupied ∧ biomass < ε を「θ_occ > ε なので偽」と切っている。θ_occ は初期仮説（D2）。θ_occ ≤ ε なら Occupied かつ Vacant が可能で、表示専用判定がコア状態と矛盾する。ガード `θ_occ > ε` が INV に無い。

参照: REQ-SIM-03a, REQ-SIM-03b

### G-03 毒性倍率の適用タイミング

Lineage 表は emission で「次 tick の maintenance に ×1.4」。同一 tick で emission のあと occupancy しか残らないので当 tick の maintenance には効かない（意図）。`step(n)` の tick 境界で倍率が付くのは「次の tick の maintenance」であり、`n` バッチの 2 tick 目からは効く。表に「tick カウンタをまたいだフラグ」が状態として無い。waste はセル共有なので、系統 L1 の emission が L2 の次 maintenance を変える。系統機械がセル waste を状態に持っていない。

参照: REQ-SIM-02, REQ-SIM-04

### G-04 Pause を Run から外したあとの外部ガード

Pause を持たないのは REQ-CON-05 どおり。説明中・OS 中断中に `step` が来たら Running は受理する。凍結は Presentation の機械が本章に無いので、コアは進められる。BD-04 完成条件が「生成テストが書ける」なら、UI 側の「凍結中は step を送らない」機械か、core の `step` 許可フラグが要る。無いと AT-D12-SCH-07 の期待をこの表は保証しない。

参照: REQ-CON-05, REQ-UI-03

### G-05 Starving + intake 後の不足分

「intake で energy が回復しても当 tick は Starving」と「starvation_and_death は cost − energy を carcass 化」が同居する。intake 後 energy ≥ cost なら不足分 ≤ 0 で無ダメージ→Alive。回復は starvation 結果にのみ依存、という補足と、energy を参照する式が衝突する。ガードは「不足分 = max(0, 記録済み不足)」のように、maintenance 時点の値を使うのか、starvation 時点の energy なのかを固定する。

参照: REQ-SIM-08, REQ-SIM-04

---

## tick 跨ぎの曖昧さ

### X-01 `step(n)` の途中終了

Prepared/Running の `step(n)` は「7 phase を n 回、tick += n。途中で終了なら Terminated」。終了した tick のあと残りの n を実行するか、そこで切るかが無い。`step(16)` で tick 3 に Extinct した場合、PRNG 消費・hash・イベント列が 3 経路試験とずれる。

期待: 終了 tick の phase 完了で打ち切り。実行した tick 数を返す。残りは消費しない。

参照: REQ-DET-02, REQ-END-02, REQ-CON-05

### X-02 Dying は「跨がない」が状態として残る

Dying は starvation_and_death 内で即 Absent。他イベントは `—`。`step(n)` が phase の途中で戻りうる（NumericError、スレッド割り込み）と Dying が tick を跨ぐ。次 tick の intake は表上到達不能。実装が Dying をフィールドに残すと U-01 と同じ穴。

期待: Dying は観測可能状態にしない。遷移は `Starving|Alive → Absent` の原子アクション。または INV「tick 境界に Dying は存在しない」。

参照: REQ-SIM-09, REQ-SIM-04

### X-03 occupancy は phase 末、Vacant は随時導出

`biomass_changed` は即遷移しない（正しい）。`snapshot` は 7 phase の途中では呼べないのか、呼ぶと Vacant が occupancy 更新前の biomass で点灯するのか。Run 表はどの状態でも snapshot 可。tick 内の観測点が無い。

期待: snapshot/explain/save は **tick 境界のみ**（1 回の `step` の入出力）。phase 途中のコピーアウトは Err。

参照: REQ-VIS-04, REQ-DET-02

### X-04 Terminated セーブの tick とラベル

終了した tick で save し load する。初期状態規則（tick≥1 → Running）だと Terminated が Running に戻る。再 `step` は ADR 候補 2 で Err になるはずが、load が Running を作ると再実行できる。終了ラベルのヒステリシス（Fixed 200 tick）は BD-04 の状態に無く、BD-01 F-07 と同じ穴。

参照: REQ-END-03, REQ-DET-06, REQ-DET-02

### X-05 生成テストが 1 tick 部分列だけ

§3.3 は「1 tick 分の phase 順部分列」。`step(n)` の n>1、Dying 非残留、毒性の次 tick、Terminated 打ち切りを生成しない。完成条件の「生成テストが書ける」は跨ぎケースで満たしていない。

参照: REQ-DET-02, REQ-SIM-02

---

## 軽微

- T1 本文は Extinct/Fixed を毎 tick、Coexist/Reversal を上限時、と正しく、表の `termination_detected` 列より精密。列を T1 に合わせて削るべき
- Cell 機械の初期状態（tick 0 の Occupied/Fading）が未定義
- Absent→Alive は D2 待ちと明記済み（許容）

---

## 集計

| 重大 | ID |
|---|---|
| 要修正 | U-01, U-02, U-04, U-05, G-01, G-05, X-01, X-02, X-04 |
| 指摘 | U-03, G-02, G-03, G-04, X-03, X-05 |

findings = **14**
