# ADR-0006 変換の余りは主出力プールへ、拡散の余りは送り元セルへ

- 状態: 採用（2026-08-30、D0 統合判定 B4、D1 r2 で実装修正）
- 参照: REQ-SIM-05, REQ-SIM-06

## 文脈
固定小数点で `floor(0.70×in) + floor(0.30×in) < in` となる余り（最大 1u×出力数）の行き先が未定義だと、保存則は満たしても分配に系統的偏りが出る（waste 側に載せると毒性判定に効く）。D1 初版は waste 側に載せていた。

## 選択肢
1. 余りは最後の出力へ（実装が楽だが、表の並び順で結果が変わる）
2. 按分（丸め規則が増える）
3. **主出力（表の 1 行目）へ。拡散は送り元セルに残す** → 採用。`ConversionRule.remainder_to` で明示

## 結果
- 全変換は `ConversionRule{from, to, coefficient, remainder_to}` 経由。`remainder_to` は出力プールのいずれかに限定（kimi N1: 型で値域を限定するか match 網羅、D2 で対応）
- UT: `split_output(3, 0.5) == (2, 1)` を golden に
