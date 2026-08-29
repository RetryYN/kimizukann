# ADR-0002 PRNG は SplitMix64 → xoshiro256** × 4 ストリーム

- 状態: 採用（2026-08-30、D0/D1 で実装済み）
- 参照: REQ-DET-04a/b, REQ-DET-05

## 文脈
用途別に独立したストリームが必要（移動・繁殖・ばらつき・予備）。状態を save に含めて三経路一致を保証する必要がある。

## 選択肢
1. ChaCha8 → 暗号強度は不要、状態が大きい（save 肥大）
2. PCG64 → 128 bit 乗算で遅い環境がある
3. **xoshiro256**、seed から SplitMix64 で 4 ストリームの初期状態を派生** → 採用。状態 32 B/ストリーム、実装が短く検証しやすい

## 結果
- `model_version` に `prng=xoshiro256ss-v1` を含める。アルゴリズム変更は model_version bump
- 各 phase のストリーム割当と消費回数は BD-07 で固定し UT でカウントする
- 既知ベクトル（seed=0,1,2^63 の先頭 4 出力）を golden にする
