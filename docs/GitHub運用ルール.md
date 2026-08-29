# GitHub 運用ルール v1.0

対象: RetryYN/kimizukann。AI チーム（Claude / Codex / kimi / grok / gemini / composer）とオーナー。
原則: **main には PR 経由でしか入らない。PR は CI green ＋ 必要レビュー ＋ ゲート承認がそろって初めてマージできる。** AI の自己申告はマージ条件にならない。

## 1. ブランチ
| ブランチ | 用途 | 規則 |
|---|---|---|
| `main` | 常にリリース可能・CI green | 直 push 禁止（branch protection）。マージは squash のみ |
| `task/<ID>` | 1 カード = 1 ブランチ（例 `task/D2-diffuse-001`, `task/BD-06-numeric`, `task/H0-harness`） | main から切る。作業者 1 人（one-branch-one-writer）。完了後削除 |
| `rfc/<NNNN>` | 要件・基本設計・契約の変更提案 | RFC 文書＋影響を受ける設計・テストの改定を同じ PR に含める |
| `hotfix/<ID>` | main の CI 破壊など緊急修正 | Claude のみ作成可。事後に RFC 不要だが開発ログに記録 |

- ID は brief / カード ID と一致させる。ID の無いブランチは CI が reject（ブランチ名チェック job）
- rebase は禁止（履歴の再現性のため）。main の取り込みは `git merge main`

## 2. コミット
- 形式: `<type>(<scope>): <要約>`。type = `feat | fix | design | docs | test | ci | chore | rfc`。scope = crate 名または BD/DD の ID
- 1 コミット 1 目的。テスト追加コミットは実装コミットより **前**（TDD 順を履歴で示す。CI が確認）
- 本文に `Refs: REQ-…, BD-…, DD-…` を書く（trace の材料）
- `--no-verify`、`--amend`（push 済みのもの）、force push は禁止

## 3. PR
### 3.1 作成条件（Draft → Ready）
- ブランチが `task/` `rfc/` `hotfix/` のいずれか
- テンプレート（`.github/PULL_REQUEST_TEMPLATE.md`）の全項目が埋まっている。空欄があれば bot が Draft に戻す
- 変更行数の目安: コード ≤ 300 行（テスト込み）。超える場合は分割理由を書く
- 自分の環境での実行結果は「参考」欄。**合否は CI の結果だけ**

### 3.2 必須チェック（branch protection の required checks）
| check | 内容 |
|---|---|
| `lint` | fmt --check, clippy -D warnings, 禁止型/浮動小数点 lint |
| `test (ubuntu)` / `test (windows)` | cargo test --workspace |
| `verify` | sim-cli verify --suite all → report.json |
| `determinism-xos` | ubuntu と windows の state_hash 一致 |
| `gate-selftest` | mutation 4 種を verify が落とすこと |
| `trace` | `scripts/gen_trace.py --strict`（P0 で設計参照なし = 0） |
| `pr-lint` | テンプレート項目・ブランチ名・commit 形式・テスト先行順 |
| `schema` | docs/contracts/schema/*.json のコンパイルと configs の検証 |

### 3.3 レビュー
| 変更の種類 | 必須レビュアー | ゲート（最終承認） |
|---|---|---|
| `crates/**`（実装） | kimi（契約逸脱・保存則・hash） | Claude |
| `crates/**/tests/**`, `docs/design/basic/08_*`（受入テスト） | writer 以外の 1 名 ＋ grok（抜け穴） | Claude |
| `docs/design/basic/**`, `docs/contracts/**`（基本設計・契約） | README の審査列に従う ＋ grok | Claude |
| `docs/design/detail/**`（詳細設計） | kimi | Claude |
| `docs/要件定義書*`, `docs/design/rfc/**` | kimi ＋ 影響先の担当全員 | **オーナー** |
| `docs/contracts/golden/**` | — | **Claude のみ**（golden 更新権限） |
| `.github/**`, `.githooks/**`, `scripts/**`（ハーネス） | grok（抜け穴） | Claude |
| `app/**`（Flutter） | grok または composer（writer 以外） | Claude |

- レビューは **GitHub の Review 機能**で行い、コメントは `file:line` に付ける。helix-bus は通知用、判断の記録は PR に残す
- レビュー結果は `Approve / Request changes` のみ。Comment だけで放置しない
- reviewer は PR 本文の「検証手順」を **自分で実行**してから approve する（CI の URL を見るだけは不可）。実行できない場合はその旨を書き、Claude が代行
- 指摘は `severity: blocker | major | minor | nit` を付ける。blocker/major が 1 つでもあれば Request changes
- writer は指摘ごとに「修正 commit hash」または「反論」を返信。全部解決したら re-request review
- **Claude のゲート承認は最後**。CI green ＋ 必須レビュー approve ＋ trace 更新を確認してから approve → squash merge → ブランチ削除 → 開発ログに 1 行

### 3.4 AI レビュアーの登録
- 各 AI は GitHub 上では「Claude 経由のコメント」として記録される（AI 個別アカウントは作らない）。コメント先頭に `[reviewer: kimi]` を付ける
- レビューの本文は helix-bus で受け取り、Claude が `gh pr review` で転記する（`scripts/pr-review.mjs` — H2 で実装）。転記時に内容を変えない
- CODEOWNERS はオーナー（RetryYN）と Claude の運用アカウント（オーナー本人のアカウントで代行）

### 3.5 マージ責任者（領域ごとに 1 名。「誰が merge ボタンを押すか」を固定）
| 領域 | マージ責任者 | 条件（すべて満たしたら責任者が squash merge） | エスカレーション |
|---|---|---|---|
| `crates/**`（コア実装・テスト） | **kimi** | CI green ＋ 契約審査 approve（kimi 自身が reviewer の場合は grok の approve） ＋ writer が全指摘に返信 | 保存則・hash・契約に触れる変更、golden 変更 → Claude |
| `app/**`（Flutter） | **grok** | CI green ＋ composer または kimi の approve | FFI 契約に触れる → Claude |
| `docs/design/basic/**`, `docs/contracts/**`（基本設計・契約） | **Claude** | README の審査列の approve ＋ grok の抜け穴審査 | 要件に矛盾 → RFC → オーナー |
| `docs/design/detail/**`（詳細設計） | **kimi** | Claude のチェックリスト審査（§設計工程）approve | — |
| `.github/**`, `.githooks/**`, `scripts/**`（ハーネス） | **Codex** | grok の抜け穴審査 approve ＋ 自己試験 green | required checks の変更 → Claude |
| `docs/要件定義書*`, `docs/design/rfc/**` | **オーナー** | kimi ＋ 影響先全員の approve | — |
| `docs/相談会/**`, `docs/design/basic/02_glossary.md`, README 類（記録・文書） | **gemini** | 記録は審査不要。glossary は kimi approve | — |
| `docs/contracts/golden/**` | **Claude** | 変更理由と再現手順が PR にある | — |
- 責任者は自分が writer の PR をマージしない（その領域の代理: crates→Claude、app→kimi、harness→grok、記録→Claude）
- 責任者はマージ後 5 分以内に開発ログへ 1 行（`bus-ctl log` で自動化予定）と、関係者へ `[ID][merged] sha=… pr=#n` を直接 post する

### 3.6 AI 間の直接ルーティング（Claude を経由しない）
原則: **PR が状態の正本、helix-bus は通知**。誰かに何かをしてほしい時は、その相手に直接 post する。Claude には「エスカレーション」と「マージ責任者が Claude の PR」だけを送る。

| 事象 | 誰が | 誰に | 何を |
|---|---|---|---|
| PR を Ready にした | writer | その領域の必須レビュアー（§3.3）＋ マージ責任者 | `[ID][review-request] pr=#n` |
| レビュー完了 | reviewer | writer ＋ マージ責任者 | `[ID][review] verdict=… pr=#n`（本文は GitHub、bus は 1 行） |
| 指摘に全部返信した | writer | reviewer | `[ID][re-request] pr=#n` |
| 条件がそろった | マージ責任者 | （自分で merge）→ writer ＋ reviewer ＋ gemini（記録） | `[ID][merged] sha=… pr=#n` |
| 判断に迷う・契約や要件に触れる | 誰でも | Claude | `[ID][escalate] pr=#n 論点=…` |
| 設計章が完成した | 起草者 | README の審査者 ＋ Claude（BD の責任者） | `[BD-xx][review-request]` |
| brief を出す | 領域のマージ責任者（コア: kimi、UI: grok、ハーネス: Codex） | writer | `[ID][brief]`（Claude は BD/契約の brief のみ） |
| 環境・CI が壊れた | 気づいた人 | Codex | `[ENV][request]` |
| 用語が無い・文言が要る | 誰でも | gemini | `[TERM][request]` |
- Codex（ルーター）は 10 分ごとに `gh pr list` を見て、review-request が 30 分以上放置されていれば該当レビュアーに催促 post、条件がそろっているのに未マージなら責任者に催促 post する
- Claude は日次で PR 一覧・開発ログ・trace を確認してオーナーに要約する（進行の中継はしない）

## 4. マージ後
- squash merge のコミットメッセージは PR タイトル＋本文の `Refs:` 行
- Claude が `docs/相談会/開発ログ.md` に `[ID] merged <sha> pr=#n reviewers=… ci=…` を追記（bus-ctl で自動化予定）
- golden や model_version が変わった PR は CHANGELOG.md に 1 行

## 5. リリース（D12 以降）
- タグ `v0.<minor>.<patch>`（検証版は 0.x）。release ブランチは作らない（main からタグ）
- APK は CI の release job が署名して artifact 化。配布メモ（SHA-256・ビルド日時・model_version）は自動生成
- hotfix はタグから `hotfix/` を切り、main と両方にマージ

## 6. 禁止事項（違反は PR reject）
1. main への直 push、force push、rebase
2. CI を通さないマージ、required check の無効化
3. `#[ignore]`、`allow(clippy::…)`、`--no-verify` の追加（brief に明記された例外を除く）
4. golden / model_version の変更を Claude 承認なしに含める
5. 自分の PR を自分でレビュー・承認する
6. 秘密情報（鍵・トークン・署名ファイル）のコミット

## 7. オーナーが GitHub で設定するもの（1 回だけ）
Settings → Branches → Add rule `main`:
- [x] Require a pull request before merging → Required approvals: **1**、Dismiss stale approvals、Require review from Code Owners
- [x] Require status checks to pass → 上記 3.2 の check を全部（最初は `lint`, `test`, `verify` だけで可。H1/H2 完了後に追加）
- [x] Require conversation resolution before merging
- [x] Require linear history（squash のみ）
- [x] Do not allow bypassing the above settings（オーナー自身も含む）
- [ ] Allow force pushes: **off**　[ ] Allow deletions: **off**
Settings → General → Pull Requests: Allow squash merging のみ ON、Automatically delete head branches ON

## 8. 移行措置
- これまで main に直 commit した分（`4c14db5` 以降の docs/design 群）はそのまま初回 push し、以後はすべて PR 経由。次の PR から本ルールを適用
