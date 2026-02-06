# Voice Test Tool

音声対話システム（音声チャットボット）のテストを自動化するデスクトップアプリケーション．
事前に用意した音声ファイルを仮想マイクデバイス経由で対象システムに入力し，テストの再現性と効率性を向上させる．

**Framework:** Tauri 2.x (Rust backend + React/TypeScript frontend)

## 対応 OS

| OS | 仮想デバイス |
|----|-------------|
| macOS | BlackHole / Soundflower（要事前インストール） |
| Windows | VB-Audio Virtual Cable / VoiceMeeter（要事前インストール） |
| Linux | PulseAudio null-sink（自動作成） |

## クイックスタート

```bash
# 前提: Rust 1.75+, Node.js 20+

cd voice-test-tool
npm install
npm run tauri dev
```

## ビルド

```bash
cd voice-test-tool
npm run tauri build
```

プラットフォーム固有のインストーラーが `src-tauri/target/release/bundle/` に生成される．

## CI/CD

GitHub Actions で3プラットフォーム（macOS / Windows / Linux）の自動ビルドを実行：

- **`build.yml`** — `main` へのプッシュ/PR でビルドチェック
- **`release.yml`** — `v*` タグプッシュでリリースビルド＆GitHub Release にアップロード

```bash
# リリース作成
git tag v0.1.0
git push origin v0.1.0
```

## プロジェクト構成

```
rs-voice-test-tools/
├── .github/workflows/     # CI/CD ワークフロー
└── voice-test-tool/       # Tauri アプリケーション
    ├── src/               # React フロントエンド
    ├── src-tauri/         # Rust バックエンド
    └── README.md          # 詳細ドキュメント
```

詳細は [`voice-test-tool/README.md`](voice-test-tool/README.md) を参照．

## ライセンス

MIT
