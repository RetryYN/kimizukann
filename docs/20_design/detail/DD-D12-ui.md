# DD-D12 詳細設計: UI 実装（Flutter 画面・スケジューラ）

- 版: 0.1（起草 cursor-kimi、2026-08-30）
- 上位正本: BD-11（画面遷移・ADV 敵対シーケンス・スケジューラ方針の正本）、BD-05 §12（FFI）、`docs/10_requirements/要件定義書_検証版_v0.2.md`（REQ-UI-01..07、REQ-NFR-02/03、REQ-CON-04/06/07）、v0.4 §1.6（操作は 3 つだけ）・§1.8（チュートリアル）、BD-08 §9（AT-D12-01/02、AT-D12-FFI-05）
- スコープ: BD-11 を実装可能な詳細に落とすもの。スケジューラの時計・間引き閾値、画面状態機械の Dart 実装構造、保存/復帰の UI 側手順
- 非スコープ: 画面遷移の仕様自体（BD-11 が確定済み。変更しない）、描画トークン（DD-D10）、FFI シグネチャ（DD-D11）
- 配置: `app/`（Flutter）

## 1. スケジューラ（確定。REQ-CON-04、BD-11 §2）

- 時計は `Stopwatch`（単調増加）。`DateTime.now` は tick 駆動・間引き判定・オートセーブ間隔のいずれにも使わない（REQ-CON-04）
- 速度: 1x = 10 tick/s、2x = 20、4x = 40（BD-11 §2 #2 の確定値）
- 1 フレームの tick 実行上限: 1x=1、2x=2、4x=4 tick/フレーム（60fps 前提の上限。これを超える分は次フレームへ繰越し、tick を破棄しない＝間引きは描画のみ。BD-11 §2 #5）
- 自動減速: 転換点スタンプ検出時に 1x へ落とす（REQ-UI-05）。減速はスケジューラの速度変数を書き換えるだけで sim 状態に触れない
- 一時停止: tick ループを止める。停止中も snapshot・explain・save は可（BD-11 §2 #3）

## 2. 画面状態機械（確定。BD-11 §1 の実装構造）

- BD-11 §1 の画面列挙を Dart の sealed class `AppScreen` に写す。遷移は BD-11 §1 の表どおりのみ許可し、表に無い遷移は静的に到達不能にする
- 各画面の FFI 呼出は BD-11 §4 の順序表どおり。ADV-01..（敵対シーケンス）は UI テストで全件実施する
- エラー表示: `KzError` を BD-11 §4 のメッセージ表に写像。`KZ_ERR_BUSY` はリトライ UI を出さず操作を無視（BD-11 §4）

## 3. 保存・中断再開の UI 手順（確定。REQ-UI-06、REQ-ACC-02、DD-D8）

- オートセーブ: 60 秒間隔（Stopwatch ベース）+ 終了時。原子的書換えは DD-D8 §4 の手順を core 側が行い、UI は `kz_save` を呼ぶだけ
- 復帰: 起動時に `kz_load`。`KZ_ERR_SCHEMA` / `KZ_ERR_VERSION` は DD-D8 §3 のメッセージを表示し新規開始へ誘導（BD-11 §3）
- 終了ラベル到達時は結果画面へ遷移し、説明 4 段（DD-D9 §6）を表示（REQ-UI-07、REQ-EXP-01）

## 4. チュートリアル（確定。REQ-UI-04a、v0.4 §1.8）

- 初回起動時のみ。3 操作（観る・間引く・種を入れる）を 1 巡で提示
- チュートリアル seed は reference_scenarios（DD-D11 §2）の `tutorial` パターン ID を使う（REQ-DET-09）

## 5. UT / AT 対応

| 検証 | 内容 |
|---|---|
| AT-D12-01 | BD-11 §5 の ADV 全件を UI テストで実施し、クラッシュ・UB・状態破壊なし（REQ-CON-06/07） |
| AT-D12-02 | 60 分連続運転（4x・間引き発生させる）でメモリリーク・tick 欠落なし（REQ-NFR-02/03） |
| AT-D12-FFI-05 | 中断→再起動→復帰で state hash が中断前と一致（REQ-ACC-02、DD-D8 と連携） |
| UT-D12-01 | スケジューラ: Stopwatch モックで 1x/2x/4x の tick 間隔とフレーム上限を検証 |
| UT-D12-02 | 間引き: 描画をスキップしても tick が破棄されない（DD-D10 §4 との結合） |
| UT-D12-03 | 画面状態機械: BD-11 §1 表外の遷移が到達不能（型レベル検査 + 遷移表の網羅テスト） |
| UT-D12-04 | エラー写像: 全 `KzError` 値がメッセージ表の行に対応 |

## 6. ファイル分割（実装 PR の予定。writer = cursor-grok）

| ファイル | 内容 |
|---|---|
| `app/lib/scheduler/tick_scheduler.dart` | §1 |
| `app/lib/nav/app_screen.dart` | §2 の sealed class と遷移表 |
| `app/lib/save/autosave.dart` | §3 |
| `app/lib/tutorial/tutorial_flow.dart` | §4 |
| `app/test/d12_ui_test.dart` | §5 UT（ADV は別ファイル `d12_adv_test.dart`） |

## 7. 未決事項（claude 裁定依頼）

- **D12-Q1**: オートセーブ間隔 60 秒は BD-11 に明示がなく本 DD の提案値。REQ-ACC-02（中断再開）を満たす範囲で問題ないか。代替: 30 秒 / 120 秒
