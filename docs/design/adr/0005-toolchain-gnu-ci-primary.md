# ADR-0005 Windows は stable-gnu toolchain、合否の正本は CI

- 状態: 採用（2026-08-30）
- 参照: REQ-CON-04, REQ-DET-03

## 文脈
開発機に MSVC link.exe が無く（VS Build Tools 未導入）、Smart App Control が未署名 exe を断続的にブロックする（os error 4551）。ローカル実行結果は再現性がない。

## 選択肢
1. VS Build Tools 導入（数 GB・管理者権限）→ 保留
2. **stable-x86_64-pc-windows-gnu（リンカ同梱）** → 採用
3. Smart App Control オフ → オーナー判断待ち（オフにすると再オン不可）

## 結果
- `rust-toolchain.toml` で channel=stable を固定。Windows の既定 host は gnu
- CI（ubuntu + windows-gnu）を合否の正本にし、ローカルは参考値。result メッセージには CI URL を必須化
- クロス OS hash 一致（ubuntu vs windows）を CI job にする。Android arm64 は D12 で実機
