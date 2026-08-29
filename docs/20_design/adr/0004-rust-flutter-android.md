# ADR-0004 Rust headless コア + Flutter UI、初期 OS は Android 1 本

- 状態: 採用（2026-08-30、検証版計画 v1.0 §1・7.2）
- 参照: REQ-CON-01, REQ-CON-03, REQ-OPS-05/07

## 文脈
1 人（＋AI チーム）開発、決定性が必須、Windows 開発環境、テスターは国内 8〜12 人。

## 選択肢
- コア: C++（安全性維持コスト）／Dart のみ・TS（契約が UI に侵入、決定性が弱い）／**Rust**（採用）
- UI: ネイティブ 2 本（工数二重）／**Flutter**（採用）／Unity（写実は不要、契約侵入）
- OS: iOS（macOS/Xcode・TestFlight 審査が必要）／**Android APK 直配布**（採用）

## 結果
- FFI は 7 操作・固定バッファのみ（BD-05）。Flutter 側にロジックを置かない
- iOS は状態 hash と描画の実機試験を別ゲートにし、検証レポートに「iPhone 未実施」を明記
- Rust toolchain は Windows では GNU（ADR-0005）
