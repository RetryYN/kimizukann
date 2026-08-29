# [BD-01][review] reviewer=cursor-grok（敵対: 依存方向・FFI・決定性の抜け穴）

- 対象: `docs/design/basic/01_context_map.md`、`docs/design/adr/0001`…`0006`
- 観点: UI/再生側から見て、状態が静かに割れうる経路。数値の妥当性には踏み込まない
- 判定: **changes_requested**（読み取り専用と「hash に速度が乗らない」が方針だけで、型境界と save 範囲が穴）

例として挙げられた 3 点はいずれも成立する。詳細は F-03 / F-06 / F-07。

---

## 抜け穴

### F-01 sim-explain が core→ffi の直線上にある（依存方向）

図は `sim-core ◄── sim-explain ◄── sim-ffi`。この形だと create/step/save も explain crate を経由する。読み取り専用は方針（§2）であり、`&mut World` を型で遮断していない。explain が「検出用に」台帳へ追記・既読フラグ・スコアキャッシュを書けば、Presentation の `explain` 呼び出しが状態変更経路になる。

期待: `sim-explain` は `sim-types`（またはコピーアウトした Snapshot/Ledger DTO）だけに依存し、`sim-ffi` が `sim-core` と `sim-explain` を並列に呼ぶ。ADR-0007 を「内部モジュール可」のまま残すとこの穴は閉じない。

参照: REQ-EVT-05, REQ-CON-01

### F-02 sim-cli の菱形依存（依存方向）

図は cli が `sim-core` と `sim-ffi` の両方に依存する。較正は core 直呼び、UI は ffi、という二経路が固定される。`step(n)` の分割差は hash 一致で検知できるが、save バイト列・エラーコード・バッファ規約の差は検知しない。較正が「通った ABI」とアプリが「呼ぶ ABI」が分かれうる。

現状 `crates/sim-cli` は core+types のみ（ffi 未新設）。図をこのまま実装すると穴が実体化する。

参照: REQ-CON-01, REQ-OPS-01

### F-03 Explain が状態を変えうる（決定性・例示 1）

§2 の AT は「検出 on/off で state hash 一致」（REQ-EVT-05）だけ。hash 正規化は転換点検出を含めない（ADR-0003、BD-03 INV-12）。よって:

- explain が台帳・イベント列・z 窓だけを書き換えても AT は通る
- `explain(event_id)` がイベントを消費（pop/既読）する API でも hash は動かない
- 境界表（§3）に `event_id` が無く、副作用の有無を検査できない

REQ-DET-07 は中断復帰と 3 速度で **イベント列一致** を要求する。hash 一致だけでは足りない。explain は純関数: `(snapshot, ledger_view, event_id) -> bytes`。同じ入力で同じ出力、入力を mutate しない、を AT にする。

参照: REQ-EVT-05, REQ-DET-07, REQ-EXP-03

### F-04 スケジューラ／snapshot が PRNG を消費しうる（決定性・例示 2）

§2「入力は config と step(n) だけ」は load/explain/save を文面から排除し、かつ snapshot を列挙しない。契約 §6 は「表示用サンプリングはコア乱数を消費しない」だが、BD-01 の決定性表（§4）に無い。

抜け穴:

- `snapshot` がトークン最大 8 を core 内でサンプルし、4 ストリームのどれかを進める。速度・間引きで呼出回数が変わり DET-07 が死ぬ
- ADR-0002 の第 4 ストリーム用途は BD-07 待ち。`interaction` / 「予備」を描画や explain が借りられる
- Flutter が `step(n)` の `n` 以外で core を呼ぶ経路（毎フレーム snapshot）が「入力は step だけ」と矛盾し、レビューで見逃す

期待: §4 に「`snapshot` / `explain` / 速度変更 / 間引きは PRNG 消費 0」。ストリーム用途を BD-01 から BD-07 へ「未割当を UI 禁止」と明示。

参照: REQ-CON-05, REQ-DET-07, REQ-OUT-04, REQ-VIS-04, ADR-0002

### F-05 「固定レイアウトのバッファのみ」が可変長契約と衝突（FFI）

§2 腐敗防止層は「7 操作と固定レイアウトのバッファのみ」。REQ-CON-08 と §2 自身の「容量不足は required_len」は可変長（save / explain JSON）を前提にする。snapshot（REQ-VIS-04）と save/explain を同一文で縛ると、実装が「全部固定」か「全部可変」かに倒れ、容量不足の再呼出が片方で欠ける。

期待: snapshot = 固定レイアウト。save/explain = 呼出側バッファ + required_len。両方を表で分ける。

参照: REQ-CON-08, REQ-VIS-04

### F-06 境界表が FFI 入力を削っている（FFI）

§3 Presentation → sim-ffi は config / seed / step 数 / save blob だけ。欠け: handle、`event_id`、out バッファ、`required_len`、destroy、エラーコード。sim-ffi → Presentation に handle 生成も無い。

これだと「許可された入力」の検査リストが書けず、未知フィールドや第 8 操作が腐敗防止層を通過する。

参照: REQ-CON-01, REQ-CON-08

### F-07 save に含まれない状態（決定性・例示 3）

ADR-0003 の正規化と REQ-DET-06 の SaveEnvelope は、セル・PRNG・model_version・config_hash まで。BD-01 は「何が save に必須で、何が再計算可能か」を境界として書いていない。欠けて割れうるもの:

| 状態 | 無いと起きること | REQ |
|---|---|---|
| Fixed の 200 tick 継続カウンタ | 中断直後にストリークが 0 に戻り、終了ラベルが遅れる／来ない。load 直後の hash は一致しうる | REQ-END-03, REQ-DET-02 |
| z 窓（10 tick 平均 × 20 サンプル） | 復帰後の転換点スコア・イベント列がずれる。hash は一致しうる | REQ-EVT-01, REQ-DET-07 |
| フロー台帳 / スタンプ列 | BD-03 ADR 候補 2 は台帳を保存しない。再計算規則が BD-01 に無い | REQ-EVT-04, REQ-DET-07 |
| InflowEvent の消化位置 | 流入が二重適用または欠落 | 契約 §3, REQ-DET-02 |
| Presentation の速度・due・仮説 3 値 | 世界 hash には不要。無いと再開 UX と「説明表示中は進めない」が壊れる。SessionSave が地図に無い | REQ-UI-03, REQ-UI-05 |

REQ-EVT-04 はセル全履歴を禁じるだけで、窓・スタンプ・継続カウンタの省略を許可していない。BD-01 が「SimCore の save/load」と「カード／指標のローカル保存」を同一「持たない I/O」に混ぜている（F-08）ため、誰が何を原子書込するかも無い。

期待: WorldSave（hash 対象 + 再計算不能なコア状態）と SessionSave（速度・画面・仮説）を §3 の行として分ける。再計算するなら関数と入力を書く。

参照: REQ-DET-06, REQ-DET-07, REQ-UI-05, ADR-0003

### F-08 SimCore が save/load を持ち、同時に I/O を持たない（境界）

§1 SimCore の責務に save/load、持たないものに I/O。ファイルの原子書込・rename・破損セーブは Presentation か Distribution か不明。中断復帰（REQ-UI-05）の「save 完了を確認してから kill してよい」がどのコンテキストの試験か書けない。

参照: REQ-CON-08, REQ-UI-05, REQ-DET-06

### F-09 完成条件の検査装置が無い（依存方向）

§2 は `cargo-deny` と `scripts/check_deps.py` で逆依存を CI 検査すると書く。リポジトリに `deny.toml` も `scripts/check_deps.py` も無い。README の BD-01 完成条件（「CI で検査可能」）を満たさない。図だけの禁止は抜け穴になる。

参照: BD-01 完成条件, REQ-CON-01

### F-10 z-score（f64）の逆流が禁止されていない（決定性）

ADR-0001 は sim-core で f32/f64 を disallow し、解析はコアへ逆流禁止。BD-01 の Explain は z を理由コードに落とす。そのコードが `step` の閾値・終了・PRNG 分岐に使われたらクロス OS で hash が割れる。§2 の逆流禁止は「Presentation → SimCore」だけで Explain → SimCore が無い。

参照: ADR-0001, REQ-DET-03, REQ-EXP-03

### F-11 FFI 再入・並列が未禁止（FFI / 決定性）

handle 同時数、UI isolate と ticker の並列 `step`/`snapshot`/`save`、explain 中の step を BD-01 が禁じていない。データ競合は PRNG と台帳を非決定にする。Presentation のスケジューラ凍結（説明中は進めない）は UI 方針であり、ABI の排他ではない。

期待: handle 最大 1、全操作は同一スレッド、操作中の再入はエラー。BD-05 に先送りするなら §5 に明示。

参照: REQ-CON-01, REQ-DET-07

### F-12 「入力は config と step(n) だけ」が load を否定する（FFI）

§2 最終箇条。中断復帰の正規経路は `load(save_bytes)`。この文を実装チェックに使うと load が違法になるか、逆に「例外」が文書化されず第 8 入力が混入する。許可入力を 7 操作の引数一覧に置き換える。

参照: REQ-CON-01, REQ-CON-05, REQ-UI-05

### F-13 トークン生成箇所が地図に無い（決定性）

REQ-OUT-04 / REQ-VIS-04 はトークン非干渉と「間引きはリプレイに載せない」。生成が `snapshot`（core）か Presentation かが BD-01 に無い。core 側生成 + 何らかのカウンタが hash 外なら、速度でトークン列だけがずれ、説明の evidence が割れても hash AT は通る。

参照: REQ-OUT-04, REQ-VIS-04, REQ-EVT-05

### F-14 ADR-0003 と §4 の「同一 hash」が範囲不足（決定性）

§4 SimCore は「同一 config/seed → 同一 hash（三経路・クロス OS）」。終了ラベルは契約 §10 では三経路の出力だが、BD-01 の表に無い。継続カウンタが hash 外なら「hash 一致・ラベル不一致」をこの表は検出対象にしない。Calibration の「同一 manifest」も、ffi 経路と core 経路（F-02）を区別しない。

参照: REQ-DET-02, REQ-END-03, ADR-0003

---

## 軽微（数えに入れない）

- Distribution に矢印が無く、APK へ載せる `model_version` / SHA-256 の出所が地図に無い
- ADR-0004「Flutter 側にロジックを置かない」と Presentation のスケジューラ所有が衝突して読める（sim 計算の意なら明記）
- ADR-0006 の余り規則はコンテキスト境界と無関係（本章の対象外）

---

## 集計

| 重大 | ID |
|---|---|
| 要修正 | F-01, F-03, F-04, F-05, F-07, F-09, F-12 |
| 指摘 | F-02, F-06, F-08, F-10, F-11, F-13, F-14 |

findings = **14**
