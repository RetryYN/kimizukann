# GitHub 運用ルール v1.0

対象: RetryYN/kimizukann。AI チーム（Claude / Codex / kimi / grok / gemini / composer）とオーナー。
原則: **main には PR 経由でしか入らない。PR は CI green ＋ 必要レビュー票（§3.8）＋ マージ責任者（§3.5）の判断がそろって初めてマージできる。** AI の自己申告はマージ条件にならない。

## 1. ブランチ
| ブランチ | 用途 | 規則 |
|---|---|---|
| `main` | 常にリリース可能・CI green | 直 push 禁止（branch protection）。マージは squash のみ |
| `task/<ID>` | 1 カード = 1 ブランチ（例 `task/D2-diffuse-001`, `task/BD-06-numeric`, `task/H0-harness`） | main から切る。作業者 1 人（one-branch-one-writer）。完了後削除 |
| `rfc/<NNNN>` | 要件・基本設計・契約の変更提案 | RFC 文書＋影響を受ける設計・テストの改定を同じ PR に含める |
| `hotfix/<ID>` | main の CI 破壊など緊急修正 | Claude のみ作成可。事後に RFC 不要だが開発ログに記録 |

- ID は brief / カード ID と一致させる。ID の無いブランチは CI が reject（ブランチ名チェック job）
- **作業ツリーの分離（必須）**: 同一 PC 上で複数の AI が同じチェックアウトを共有しているため、`git checkout` でブランチを切り替えてはならない（他者の作業ツリーが巻き込まれる）。各カードは **git worktree** で独立ディレクトリを作る:
  `git worktree add ../kimizukann-wt/<ID> -b task/<ID> main` → その中で作業・commit・push → PR マージ後 `git worktree remove ../kimizukann-wt/<ID>`。メインのチェックアウト（`projects/kimizukann`）は常に `main` に置き、読み取り専用とする
- `git add -A` は禁止。触った自分のファイルだけを `git add <path>` する
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
| `schema` | docs/30_contracts/schema/*.json のコンパイルと configs の検証 |

### 3.3 レビュー
| 変更の種類 | 必須レビュアー | マージ責任者 |
|---|---|---|
| `crates/**`（実装） | kimi（契約逸脱・保存則・hash） | kimi（§3.5） |
| `crates/**/tests/**`, `docs/20_design/basic/08_*`（受入テスト） | writer 以外の 1 名 ＋ grok（抜け穴） | kimi（§3.5） |
| `docs/20_design/basic/**`, `docs/30_contracts/**`（基本設計・契約） | README の審査列に従う ＋ grok | Claude（§3.5） |
| `docs/20_design/detail/**`（詳細設計） | kimi | kimi（§3.5） |
| `docs/10_requirements/要件定義書*`, `docs/20_design/rfc/**` | kimi ＋ 影響先の担当全員 | **オーナー** |
| `docs/30_contracts/golden/**` | — | Claude（§3.5、golden 更新権限） |
| `.github/**`, `.githooks/**`, `scripts/**`（ハーネス） | grok（抜け穴） | Codex（§3.5） |
| `app/**`（Flutter） | grok または composer（writer 以外） | grok（§3.5） |

- レビューは **GitHub の Review 機能**で行い、コメントは `file:line` に付ける。helix-bus は通知用、判断の記録は PR に残す
- 単一アカウントのため GitHub の Approve / Request changes は使えない（自分の PR 扱いで 422）。レビュー結果は **本文に `verdict: approve | request-changes` を書いた COMMENT ＋ §3.8 の署名票**が正規手順。verdict の無いコメントはレビューとして数えない
- reviewer は PR 本文の「検証手順」を **自分で実行**してから approve する（CI の URL を見るだけは不可）。実行できない場合はその旨を書き、Claude が代行
- 指摘は `severity: blocker | major | minor | nit` を付ける。blocker/major が 1 つでもあれば Request changes
- writer は指摘ごとに「修正 commit hash」または「反論」を返信。全部解決したら re-request review
- merge ボタンの所有者は §3.5 のマージ責任者だけ（Claude は自分が責任者の領域のみ）。責任者は CI green ＋ 必須レビュー approve ＋ trace 更新を確認してから squash merge → ブランチ削除 → `[merged]` を post

### 3.4 AI レビュアーの登録
- 各 AI は GitHub 上では「Claude 経由のコメント」として記録される（AI 個別アカウントは作らない）。コメント先頭に `[reviewer: kimi]` を付ける
- **各 AI が直接 `gh` で PR にレビュー本文を書く**（gh は共有認証）。helix-bus には 1 行の通知だけ。転記は行わない（H2 の pr-review.mjs は廃止）。PR が正本、bus は通知
- CODEOWNERS はオーナー（RetryYN）と Claude の運用アカウント（オーナー本人のアカウントで代行）

### 3.5 マージ責任者（領域ごとに 1 名。「誰が merge ボタンを押すか」を固定）
| 領域 | マージ責任者 | 条件（すべて満たしたら責任者が squash merge） | エスカレーション |
|---|---|---|---|
| `crates/**`（コア実装・テスト） | **kimi** | CI green ＋ 契約審査 approve（kimi 自身が reviewer の場合は grok の approve） ＋ writer が全指摘に返信 | **契約本文（05_contract / docs/30_contracts）・golden・hash 正規化順** を変える PR のみ Claude |
| `app/**`（Flutter） | **grok** | CI green ＋ composer または kimi の approve | FFI 契約に触れる → Claude |
| `docs/20_design/basic/**`, `docs/30_contracts/**`（基本設計・契約） | **Claude** | README の審査列の approve ＋ grok の抜け穴審査 | 要件に矛盾 → RFC → オーナー |
| `docs/20_design/detail/**`（詳細設計） | **kimi** | Claude のチェックリスト審査（§設計工程）approve | — |
| `.github/**`, `.githooks/**`, `scripts/**`（ハーネス） | **Codex** | grok の抜け穴審査 approve ＋ 自己試験 green | required checks の変更 → Claude |
| `docs/10_requirements/要件定義書*`, `docs/20_design/rfc/**` | **オーナー** | kimi ＋ 影響先全員の approve | — |
| `docs/50_records/**`（議事録・レビュー記録・開発ログ）, README 類 | **gemini** | 記録は審査不要。ただし `docs/50_records/briefs/**` は brief 発行者の領域責任者が審査 | — |
| `docs/20_design/basic/02_glossary.md` | **gemini** | kimi approve（basic/** の例外） | — |
| `docs/20_design/adr/**`, `docs/40_process/GitHub運用ルール.md`, `docs/40_process/第5回_*`（体制・規則） | **Claude** | grok の抜け穴審査 | 役割・規則の変更はオーナーに日次報告 |
| `docs/calib/**`（較正 manifest） | **kimi** | grok（分布の読み） | 代表史の置換 → オーナー |
| `docs/dist/**`（配布） | **Codex** | grok | 署名鍵・配布先 → オーナー |
| `docs/30_contracts/golden/**` | **Claude** | 変更理由と再現手順が PR にある | — |

複数領域にまたがる PR の tie-break: 優先順 オーナー ＞ Claude ＞ kimi ＞ grok ＞ Codex ＞ gemini で最上位の責任者がマージする（分割できるなら分割を求める）。
- 責任者は自分が writer の PR をマージしない（代理: crates→Claude、app→kimi、基本設計/契約/規則→kimi、golden→オーナー、harness→grok、記録→Claude）。app で grok が writer の場合のレビュアーは composer、マージは kimi
- identity 注: composer（Composer 2.5）は helix-bus 上では `cursor-glm` の identity を使う（改名事故の回避のため据え置き）
- `gh pr merge --admin` 等の保護バイパスは禁止（条件がそろわないなら merge しない。設定不備は Codex に `[ENV]`）
- 責任者はマージ後、関係者へ `[ID][merged] sha=… pr=#n` を直接 post する（開発ログは §4 のとおり gemini が記録）

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
- Codex（ルーター）は 10 分ごとに `gh pr list --json number,isDraft,reviewRequests,reviews,updatedAt,labels` を見る。観測対象は **GitHub の Ready 時刻（`ready_for_review` イベント）とラベル**: writer は Ready にする時に `needs-review:<name>` ラベルを付け、reviewer は本文投稿後に外す。ラベルが 30 分以上残っていれば該当レビュアーへ催促 post、`ready-to-merge` ラベル（必須レビュアー全員の `verdict: approve` COMMENT／署名票がそろった時点で **最後の reviewer が付ける**。§3.8 の review-gate が green ならラベルは自動付与）が 30 分残っていれば責任者へ催促
- Claude は日次で PR 一覧・開発ログ・trace を確認してオーナーに要約する（進行の中継はしない）

### 3.7 リミット管理（予算と上限）
- **予算の正本**: `~/.helix-bus/budget.json`（オーナーが編集）。計測: `node ~/.helix-bus/usage.mjs [--hours N]`（identity ごとの wait / stop 回数 ≈ リクエスト数、予算比、WARN 80% / STOP 100%）
- **上限**（budget.json `review`）: レビュー往復 **最大 3 ラウンド**（4 回目は責任者が判定して merge か close）、writer あたり open PR **2 本**まで、PR **300 行**まで
- **待機ポリシー**（`waitPolicy`）: タスクを持つ AI は `timeout_sec=50`、**30 分タスク無しなら `timeout_sec=600`**（Cursor が許せば。LIMIT-TEST で確認）または待機停止。Codex は 3600
- **STOP 時**: Claude が該当 AI に `[LIMIT][constraint] 待機停止` を送り、その日のタスクは他へ振る。合計が STOP なら新規 brief を止めてオーナーに報告
- Claude の 2 時間ジョブで usage を確認し、日次要約に「消費 / 予算」を 1 行入れる

### 3.8 レビュー証拠ゲート（アカウントを増やさずに「レビュー無しでは落ちる」を Actions で実現）
GitHub の Review approve は使わない（単一アカウント）。代わりに required check **`review-gate`** が PR のコメントを検査し、以下をすべて満たさない限り red にする。

**A. 署名付きレビュー票（attestation）**
- reviewer は本文を PR に書いた後、helix-bus で `attest(pr, sha, verdict)` を呼ぶ（MCP ツール、`me` 必須）。bus は `~/.helix-bus/keys/attest.secret`（オーナー配置、GitHub Secret `HELIX_ATTEST_SECRET` と同一）で **HMAC-SHA256(reviewer|pr|head_sha|verdict|checklist_hash)** を作り、PR に次の 1 コメントを投稿する:
  ```
  helix-review: v1
  reviewer: kimi
  pr: 12
  sha: <head commit sha>
  verdict: approve | request-changes
  checklist: <該当チェックリストの yes/no 列の sha256>
  evidence: <reviewer が自分で実行した検証手順の出力の sha256（§3.3「自分で実行」の証拠）>
  sig: <hmac>
  ```
- `review-gate` は Secret で sig を再計算し、一致しないコメントを無視する。**sha が現在の head と一致しない票は無効**（push すると再レビューが必要＝stale approval の代替）
- writer 自身の票は無効（PR 本文の `writer:` と reviewer が一致 → 無効）。writer 欄が無い PR は red

**B. 必要レビュアーの充足**
- `.github/review-owners.json` に「パス glob → 必須レビュアー集合（AND/OR）」を置く（§3.3 の表を機械化）。変更ファイル集合に対し全条件が approve 票で満たされること
- `docs/30_contracts/**`・`golden/**`・hash 正規化順の変更は Claude 票が必須。要件・RFC はオーナー票（オーナーは GitHub の Review approve で可＝唯一の人間）

**C. 証拠の突合**
- reviewer 票の `evidence` は、CI の `verify` job が出す `report.json` 内の `state_hash` 集合の sha256 と一致すること（＝reviewer が同じ検証を本当に走らせた）。文書のみの PR は `evidence: none` を許可し、代わりに `checklist` 必須
- writer は PR 本文に `diff --stat` を貼る。`pr-lint` が実際の diff と突合し、不一致なら red（「反映した」誤報の機械検出）

**D. その他の必須 check**（§3.2 に追加）
- `pr-lint`: テンプレ全項目、`writer:` 欄、ブランチ名、commit 形式、テスト commit が実装 commit より前、300 行、`#[ignore]`/`allow(clippy)`/`--no-verify` の追加禁止
- `review-gate`: A〜C
- `rounds`: `helix-review` 票の request-changes が 3 回に達したら red にし、責任者の判定コメント `helix-decision: merge|close` が無い限り開かない（§3.7）

**E. 運用**
- branch protection の required checks に `review-gate` と `pr-lint` を入れる（H2 完了時に Codex が設定）。approvals は 0 のままでよい（票が代替）
- 票の偽造は同一 PC 上では原理的に防げない（全 AI が同じ Secret を読める）。本ゲートが守るのは **手順の省略**（レビュー無し・古い sha・writer 自己承認・証拠不一致・往復超過）であり、identity の暗号的証明ではない。identity の担保は helix-bus の `me` と events.jsonl の監査ログで行う

## 4. マージ後
- squash merge のコミットメッセージは PR タイトル＋本文の `Refs:` 行
- gemini が `[merged]` を受けて `docs/50_records/開発ログ.md` に `[ID] merged <sha> pr=#n reviewers=… ci=…` を追記（記録 PR は審査不要、§3.5）
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
- [x] Require a pull request before merging → Required approvals: **0**（単一アカウントのため。承認は §3.8 の `review-gate` check が代替）、Code Owners は使わない
- [x] Require status checks to pass → `lint`, `test (ubuntu)`, `test (windows)`, `verify`, `pr-lint`, `review-gate`（H0/H2 マージ後に Codex が設定。それまでは空）
- [x] Require conversation resolution before merging
- [x] Require linear history（squash のみ）
- [x] Do not allow bypassing the above settings（オーナー自身も含む）
- [ ] Allow force pushes: **off**　[ ] Allow deletions: **off**
Settings → General → Pull Requests: Allow squash merging のみ ON、Automatically delete head branches ON

## 8. 移行措置
- これまで main に直 commit した分（`4c14db5` 以降の docs/design 群）はそのまま初回 push し、以後はすべて PR 経由。次の PR から本ルールを適用
