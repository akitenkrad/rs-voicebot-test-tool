# Voice Test Tool

音声対話システム（音声チャットボット）のテストを自動化・効率化するためのGUIアプリケーション．
事前に用意した音声ファイルを仮想マイクデバイス経由で対象システムに入力し，テストの再現性と効率性を向上させる．

> **Framework:** Tauri 2.x (Rust + React/TypeScript)

---

## 目次

- [対応OS](#対応os)
- [前提条件](#前提条件)
- [セットアップ](#セットアップ)
- [起動方法](#起動方法)
- [ビルド（配布用バイナリ生成）](#ビルド配布用バイナリ生成)
- [使い方](#使い方)
  - [画面構成](#画面構成)
  - [音声ファイルの読込と再生](#音声ファイルの読込と再生)
  - [仮想マイクデバイス](#仮想マイクデバイス)
  - [プレイリスト](#プレイリスト)
  - [テストシナリオ](#テストシナリオ)
  - [ログとレポート](#ログとレポート)
  - [設定](#設定)
  - [キーボードショートカット](#キーボードショートカット)
- [シナリオファイル形式](#シナリオファイル形式)
- [プロジェクト構成](#プロジェクト構成)
- [トラブルシューティング](#トラブルシューティング)
- [ライセンス](#ライセンス)

---

## 対応OS

| OS | 仮想デバイス方式 |
|----|------------------|
| **macOS** | [BlackHole](https://existential.audio/blackhole/) または Soundflower を検出 |
| **Windows** | [VB-Audio Virtual Cable](https://vb-audio.com/Cable/) または VoiceMeeter を検出 |
| **Linux** | PulseAudio null-sink を自動作成（`pactl` 使用） |

---

## 前提条件

### 全OS共通

| ツール | バージョン | 確認コマンド |
|--------|-----------|-------------|
| **Rust** | 1.75+ | `rustc --version` |
| **Node.js** | 20+ | `node --version` |
| **npm** | 9+ | `npm --version` |

### macOS 追加要件

```bash
# Xcode Command Line Tools
xcode-select --install

# BlackHole（仮想オーディオデバイス）
brew install blackhole-2ch
```

### Windows 追加要件

- **Visual Studio Build Tools** （C++ ビルドツール）
- **WebView2**（Windows 10 以降は標準搭載）
- **VB-Audio Virtual Cable**（https://vb-audio.com/Cable/ からインストール）

### Linux 追加要件

```bash
# Ubuntu/Debian
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libasound2-dev \
  libpulse-dev \
  pkg-config \
  cmake

# Fedora
sudo dnf install -y \
  webkit2gtk4.1-devel \
  openssl-devel \
  curl \
  wget \
  file \
  libappindicator-gtk3-devel \
  librsvg2-devel \
  alsa-lib-devel \
  pulseaudio-libs-devel \
  pkg-config \
  cmake
```

---

## セットアップ

```bash
# 1. リポジトリをクローン
git clone <repository-url>
cd voice-test-tool

# 2. フロントエンドの依存関係をインストール
npm install

# 3. Rust の依存関係は初回ビルド時に自動取得されます
```

---

## 起動方法

### 開発モード（ホットリロード付き）

```bash
npm run tauri dev
```

- フロントエンド（Vite）が `http://localhost:1420` で起動
- Rust バックエンドがコンパイルされ，Tauri ウィンドウが開く
- フロントエンド/バックエンドの変更は自動で反映される
- DevTools が有効（`F12` または右クリック → 検証で開く）

> **初回起動時の注意:** Rust クレートのコンパイルに数分かかります．2回目以降はインクリメンタルビルドにより高速化されます．

### フロントエンドのみ（Rust なし）

```bash
npm run dev
```

ブラウザで `http://localhost:1420` を開くとUIのみ確認可能．ただし Tauri コマンド（オーディオ再生，デバイス管理など）は動作しません．

---

## ビルド（配布用バイナリ生成）

```bash
npm run tauri build
```

ビルド成果物は以下に出力されます：

| OS | 出力先 | 形式 |
|----|--------|------|
| macOS | `src-tauri/target/release/bundle/dmg/` | `.dmg`, `.app` |
| Windows | `src-tauri/target/release/bundle/msi/` | `.msi`, `.exe` |
| Linux | `src-tauri/target/release/bundle/deb/` | `.deb`, `.AppImage` |

---

## 使い方

### 画面構成

```
+----------------------------------------------------------+
|  Header（テーマ切替・言語切替・設定・デバイス状態）           |
+----------+-----------------------------------------------+
|          |                                               |
| Sidebar  |  MainContent                                  |
|          |  - 波形表示（wavesurfer.js）                    |
| - Files  |  - 再生コントロール                              |
| - List   |    (再生/停止/シーク/速度/音量)                   |
| - Scen.  |                                               |
|          |                                               |
+----------+-----------------------------------------------+
|  BottomPanel（ログビューア + エクスポート）                  |
+----------------------------------------------------------+
```

- **Header**: アプリタイトル，言語切替（日本語/英語），テーマ切替（ライト/ダーク/システム），設定ボタン，仮想デバイス状態
- **Sidebar**: 3つのタブ（ファイル/プレイリスト/シナリオ）を切替
- **MainContent**: 選択中のファイルの波形表示と再生コントロール
- **BottomPanel**: テストセッションのログ表示とレポートエクスポート

### 音声ファイルの読込と再生

1. サイドバーの **「ファイル」** タブを選択
2. **「ファイルを追加」** ボタンをクリック（または `Ctrl+O` / `Cmd+O`）
3. 対応形式のオーディオファイルを選択
   - WAV, MP3, FLAC, OGG に対応
4. ファイルリストに追加されたファイルをクリックして選択
5. メインエリアに波形が表示される
6. 再生コントロールで操作：
   - **再生/一時停止**: `Space` キーまたはボタン
   - **停止**: `Escape` キーまたはボタン
   - **シーク**: 波形をクリック，または `←` / `→` で5秒移動
   - **音量**: `↑` / `↓` キーまたはスライダー
   - **再生速度**: 0.5x 〜 2.0x のスライダー

### 仮想マイクデバイス

ヘッダー右側の **デバイス状態インジケータ** から操作：

1. インジケータをクリック
2. **仮想デバイスを作成**（macOS/Windows: 既存のドライバを検出，Linux: PulseAudio null-sink を作成）
3. 作成成功後，インジケータが **「アクティブ」** に変わる
4. テスト対象のアプリケーションのマイク入力にこの仮想デバイスを選択
5. 音声ファイルを再生すると，仮想マイクを通じてテスト対象に入力される

> **macOS:** 事前に BlackHole のインストールが必要です
> **Windows:** 事前に VB-Audio Virtual Cable のインストールが必要です
> **Linux:** `pactl` が利用可能であれば自動で作成されます

### プレイリスト

複数の音声ファイルを順番に再生する機能：

1. サイドバーの **「プレイリスト」** タブを選択
2. **「新規プレイリスト」** で名前を入力して作成
3. 各ファイルに **前無音** と **後無音**（秒）を設定可能
4. ドラッグ&ドロップでファイルの順序を変更
5. **再生ボタン** で順番に再生開始
6. ループモードの有効/無効を切替可能

### テストシナリオ

JSON または YAML 形式のシナリオファイルを読み込み，自動テストを実行：

1. サイドバーの **「シナリオ」** タブを選択
2. **「シナリオ読込」** で JSON/YAML ファイルを選択
3. 読み込まれたシナリオの詳細（ターン数，各ターンの音声ファイル・遅延設定）を確認
4. **「実行」** ボタンでシナリオ実行開始
5. 実行中の操作：
   - **一時停止 / 再開**: 実行を一時的に中断・再開
   - **中止**: 実行をキャンセル
6. 進捗がプログレスバーで表示される
7. ログがボトムパネルにリアルタイムで記録される

### ログとレポート

テスト実行のログを閲覧・エクスポート：

1. **ボトムパネル** のセッションセレクタからテストセッションを選択
2. ログレベルフィルタ（All / Info / Warn / Error）で絞り込み
3. エクスポートボタンで保存先を選択：
   - **JSON**: 構造化データ（プログラム連携向け）
   - **CSV**: スプレッドシートで開ける形式
   - **HTML**: ブラウザで閲覧可能なレポート

### 設定

`Ctrl+,`（macOS: `Cmd+,`）またはヘッダーの歯車アイコンから設定画面を開く：

| タブ | 設定項目 |
|------|---------|
| **オーディオ** | サンプルレート（22050/44100/48000），チャンネル数，バッファサイズ，デフォルト再生速度，デフォルト音量 |
| **デバイス** | 仮想デバイス名，起動時自動作成，デフォルト入力設定，終了時クリーンアップ |
| **外観** | テーマ（ライト/ダーク/システム），言語（日本語/英語），波形カラー |
| **ショートカット** | キーボードショートカット一覧（読み取り専用） |

設定は `config.toml` として OS のコンフィグディレクトリに保存されます：

| OS | パス |
|----|------|
| macOS | `~/Library/Application Support/com.voice-test-tool.VoiceTestTool/config.toml` |
| Windows | `C:\Users\<user>\AppData\Roaming\voice-test-tool\VoiceTestTool\config\config.toml` |
| Linux | `~/.config/VoiceTestTool/config.toml` |

### キーボードショートカット

| 操作 | Windows/Linux | macOS |
|------|---------------|-------|
| 再生/一時停止 | `Space` | `Space` |
| 停止 | `Escape` | `Escape` |
| 次のファイル | `Ctrl+→` | `Cmd+→` |
| 前のファイル | `Ctrl+←` | `Cmd+←` |
| 5秒進む | `→` | `→` |
| 5秒戻る | `←` | `←` |
| 音量上げ | `↑` | `↑` |
| 音量下げ | `↓` | `↓` |
| ファイルを開く | `Ctrl+O` | `Cmd+O` |
| 設定を開く | `Ctrl+,` | `Cmd+,` |
| シナリオ実行 | `F5` | `F5` |
| シナリオ停止 | `Shift+F5` | `Shift+F5` |

> 入力フィールドにフォーカス中はショートカットが無効化されます．

---

## シナリオファイル形式

テストシナリオは JSON または YAML 形式で記述します．音声ファイルのパスはシナリオファイルからの相対パスで指定できます．

### JSON 形式

```json
{
  "name": "基本的な挨拶シナリオ",
  "description": "システムとの基本的な挨拶のやり取りをテスト",
  "metadata": {
    "author": "Test Engineer",
    "version": "1.0.0",
    "created_at": "2025-02-03T10:00:00Z"
  },
  "turns": [
    {
      "audio_file": "test_audio/greeting_hello.wav",
      "expected_transcript": "こんにちは",
      "expected_response": "こんにちは",
      "delay_before_ms": 0,
      "delay_after_ms": 1000
    },
    {
      "audio_file": "test_audio/ask_weather.wav",
      "expected_transcript": "今日の天気は",
      "expected_response": "今日の天気",
      "delay_before_ms": 500,
      "delay_after_ms": 1000
    }
  ]
}
```

### YAML 形式

```yaml
name: 基本的な挨拶シナリオ
description: システムとの基本的な挨拶のやり取りをテスト
metadata:
  author: Test Engineer
  version: 1.0.0
  created_at: 2025-02-03T10:00:00Z

turns:
  - audio_file: test_audio/greeting_hello.wav
    expected_transcript: こんにちは
    expected_response: こんにちは
    delay_before_ms: 0
    delay_after_ms: 1000

  - audio_file: test_audio/ask_weather.wav
    expected_transcript: 今日の天気は
    expected_response: 今日の天気
    delay_before_ms: 500
    delay_after_ms: 1000
```

### フィールド説明

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `name` | string | Yes | シナリオ名 |
| `description` | string | No | シナリオの説明 |
| `metadata` | object | No | 作者・バージョンなどのメタデータ |
| `turns` | array | Yes | テストターンのリスト |
| `turns[].audio_file` | string | Yes | 音声ファイルのパス（相対/絶対） |
| `turns[].expected_transcript` | string | No | 期待される文字起こしテキスト |
| `turns[].expected_response` | string | No | 期待されるシステム応答 |
| `turns[].delay_before_ms` | number | No | 再生前の待機時間（ミリ秒，デフォルト: 0） |
| `turns[].delay_after_ms` | number | No | 再生後の待機時間（ミリ秒，デフォルト: 0） |

---

## プロジェクト構成

```
voice-test-tool/
├── package.json              # フロントエンド依存関係・スクリプト
├── vite.config.ts            # Vite設定
├── tsconfig.json             # TypeScript設定
├── tailwind.config.js        # Tailwind CSS設定
├── index.html                # エントリHTML
│
├── src/                      # フロントエンド (React/TypeScript)
│   ├── App.tsx               # メインアプリケーション
│   ├── main.tsx              # エントリポイント
│   ├── index.css             # グローバルスタイル
│   ├── components/           # UIコンポーネント
│   │   ├── Header/           # ヘッダー・デバイス状態
│   │   ├── Sidebar/          # ファイル・プレイリスト・シナリオパネル
│   │   ├── MainContent/      # 波形表示・再生コントロール
│   │   ├── BottomPanel/      # ログビューア
│   │   ├── Modals/           # 設定モーダル
│   │   └── ui/               # shadcn/ui基盤コンポーネント
│   ├── hooks/                # カスタムフック
│   │   ├── useAudio.ts       # 音声ファイル管理
│   │   ├── usePlayback.ts    # 再生状態ポーリング
│   │   ├── useDevice.ts      # デバイス状態管理
│   │   ├── usePlaylist.ts    # プレイリスト操作
│   │   ├── useScenario.ts    # シナリオ実行管理
│   │   ├── useConfig.ts      # 設定読込・保存
│   │   ├── useWaveSurfer.ts  # wavesurfer.jsライフサイクル
│   │   └── useKeyboardShortcuts.ts  # ホットキー
│   ├── stores/               # Zustand状態管理
│   ├── types/                # TypeScript型定義
│   ├── lib/                  # ユーティリティ・Tauri APIラッパー
│   └── i18n/                 # 国際化（日本語・英語）
│
└── src-tauri/                # バックエンド (Rust)
    ├── Cargo.toml            # Rust依存関係
    ├── tauri.conf.json       # Tauri設定
    └── src/
        ├── lib.rs            # エントリ・コマンド登録
        ├── main.rs           # バイナリエントリ
        ├── error.rs          # エラー型定義
        ├── types.rs          # 共有型定義
        ├── config.rs         # 設定管理（TOML永続化）
        ├── state.rs          # アプリケーション状態
        ├── audio/            # オーディオエンジン
        │   ├── decoder.rs    # WAV/MP3/FLAC/OGG デコーダ
        │   ├── player.rs     # rodioベース再生プレイヤー
        │   └── waveform.rs   # 波形ピーク生成
        ├── device/           # 仮想デバイス管理
        │   ├── macos.rs      # BlackHole/Soundflower検出
        │   ├── windows.rs    # VB-Cable/VoiceMeeter検出
        │   └── linux.rs      # PulseAudio null-sink管理
        ├── scenario/         # テストシナリオ
        │   ├── parser.rs     # JSON/YAMLパーサー
        │   └── executor.rs   # シナリオ実行エンジン
        ├── logging/          # ログ・レポート
        │   └── reporter.rs   # TestLogger + JSON/CSV/HTMLエクスポート
        └── commands/         # Tauriコマンド（IPC境界）
            ├── audio.rs      # ファイル読込・波形取得
            ├── playback.rs   # 再生制御
            ├── device.rs     # デバイス操作
            ├── playlist.rs   # プレイリストCRUD・再生
            ├── scenario.rs   # シナリオ読込・実行
            ├── log.rs        # ログ取得・レポートエクスポート
            └── config.rs     # 設定読込・更新
```

---

## トラブルシューティング

### 初回ビルドが失敗する

```bash
# Rust ツールチェインの更新
rustup update

# npm の依存関係をクリーンインストール
rm -rf node_modules package-lock.json
npm install
```

### macOS で BlackHole が検出されない

1. BlackHole がインストールされているか確認：
   ```bash
   # インストール済みデバイスを確認
   system_profiler SPAudioDataType | grep -i blackhole
   ```
2. システム環境設定 → サウンド で BlackHole デバイスが表示されるか確認
3. アプリを再起動

### Windows で VB-Cable が検出されない

1. VB-Audio Virtual Cable がインストールされているか確認
2. サウンド設定で「CABLE Input」デバイスが表示されるか確認
3. 管理者権限でアプリを起動してみる

### Linux で仮想デバイスが作成できない

```bash
# PulseAudio が起動しているか確認
pulseaudio --check && echo "running" || echo "not running"

# pactl が利用可能か確認
which pactl

# 手動で null-sink を作成してテスト
pactl load-module module-null-sink sink_name=VoiceTestTool \
  sink_properties=device.description="VoiceTestTool"
```

### 音声ファイルが読み込めない

- 対応形式: **WAV**, **MP3**, **FLAC**, **OGG**
- ファイルパスに日本語やスペースが含まれている場合，正しく動作しない可能性があります
- ファイルが壊れていないか，他のプレイヤーで再生できるか確認してください

### 開発モードで Rust の変更が反映されない

`npm run tauri dev` は Rust の変更を検知して自動再コンパイルしますが，稀にキャッシュが古い場合があります：

```bash
# Rust のビルドキャッシュをクリア
cd src-tauri && cargo clean && cd ..

# 再度起動
npm run tauri dev
```

---

## ライセンス

MIT License
