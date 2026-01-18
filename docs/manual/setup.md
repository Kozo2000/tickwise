# 📘 Tickwise セットアップガイド（v1.1）

このドキュメントは、Tickwise を **初めて実行するためのセットアップ専用ガイド** です。
README とは役割を分け、「動かすところまで」に集中して解説します。

---

# 1. セットアップの概要

Tickwise を実行するために必要な準備は次の 4 点です。

1.  **バイナリの入手とハッシュ確認**
2.  **東証の銘柄一覧（Excel → CSV）の準備**
3.  **API キー（OpenAI / Brave）の取得**
4.  **`tickwise.env` の作成**

**所要時間の目安：** 10〜20 分

---

# 2. バイナリの入手とハッシュ確認

実行に必要なプログラム本体を入手し、改ざんや破損がないか検証します。

### 2-1. バイナリのダウンロード
[Binary フォルダ](../../Binary) から、環境に合ったファイルをダウンロードしてください。

- **Windows版**: `tickwise-Win64-v1_1.zip`
- **macOS版**: `tickwise-MacOS-ARM-v1_1.zip`（Apple Silicon搭載Mac用）

### 2-2. ハッシュ（SHA-256）の確認
安全のため、ダウンロードしたファイルが正規のものであるか確認することを推奨します。

- **ハッシュファイル名**: 
  - Windows: `tickwise_v1_1_windows-64_hash_sha256.txt`
  - macOS: `tickwise_v1_1_Mac_ARM_hash_sha256.txt`

- **確認コマンド**:
  - **Windows (PowerShell)**:
    ```powershell
    Get-FileHash .\tickwise-release.zip -Algorithm SHA256
    ```
  - **macOS / Linux**:
    ```bash
    shasum -a 256 tickwise-release.zip
    ```

> [!IMPORTANT]
> 表示された文字列が、配布元のハッシュファイルに記載された値と完全に一致することを確認してください。一致しない場合は実行しないでください。

### 2-3. 展開（解凍）
確認後、zipファイルを適切なフォルダ（例: `C:\tickwise` や `~/tickwise`）に展開します。

---

# 3. 東証の銘柄一覧の準備

Tickwise は銘柄コードの正規化のために、東証が公開している「上場銘柄一覧」を使用します。

### 3-1. 公式ページからダウンロード
1.  [東証：その他統計資料](https://www.jpx.co.jp/markets/statistics-equities/misc/01.html) を開きます。
2.  「上場銘柄一覧」の Excel ファイル（`data_j.xls` または `data_j.xlsx`）をダウンロードします。

### 3-2. CSV（UTF-8）への変換
ダウンロードしたファイルを Excel 等で開き、以下の形式で保存し直してください。

- **保存形式**: `CSV UTF-8 (コンマ区切り) (*.csv)`
- **注意点**: 
    - **必ず UTF-8 を指定**してください（Shift-JIS では文字化けします）。
    - 列やヘッダの構成は**一切変更しないでください**。

### 3-3. 保存場所
展開した Tickwise フォルダの中に保存します。
- 例: `tickwise_data/tse_list.csv` (このパスは後述する設定ファイルで指定します)

---

# 4. API キーの取得

### 4-1. OpenAI API キー（ニュース要約用）
- **取得先**: [OpenAI Platform](https://platform.openai.com/)
- **形式**: `sk-xxxx...`
- ※ LLM による分析レポートの生成に必要です。

### 4-2. Brave News API キー（ニュース取得用）
- **取得先**: [Brave Search API](https://brave.com/search/api/)
- **形式**: `brv-xxxx...`
- ※ 最新ニュースの取得機能を使う場合に必要です。

---

# 5. tickwise.env の作成

設定ファイル `tickwise.env` を作成します。雛形（sample）をコピーして使うのが最も確実です。

> [!WARNING]
> **APIキーの取り扱いについて**
> APIキーは重要な個人情報です。絶対に第三者に公開しないでください。`.env` ファイルはローカル環境でのみ管理し、Git 等の公開設定には含めないよう注意してください。

### 5-1. ファイルの作成
リポジトリ直下の `tickwise.env.sample` をコピーし、ファイル名を `tickwise.env` に変更します。

```text
tickwise.env.sample  →  tickwise.env
```

### 5-2. 最小限の設定（必須項目）
最初に編集が必要なのは API キーのみです。

```ini
#===== API_KEY =====
OPENAI_API_KEY=sk-xxxxx
BRAVE_API_KEY=brv-xxxxx
```

### 5-3. その他の設定（初期値推奨）
以下の項目は、動作確認が完了してから必要に応じて調整してください。

- **テクニカル閾値**: RSI, MACD 等（`BUY_RSI=30.0` など）
- **指標の有効/無効**: EMA, SMA, フィボナッチ等（`EMA=True` など）
- **重み付け（Weight）**: 各指標の重要度
- **モデル設定**: `OPENAI_MODEL=gpt-4o` など

---

# 6. 初回動作チェック

セットアップが成功したか、最小限の機能でテストします。

### 6-1. テスト実行コマンド
ターミナルまたは PowerShell を開き、以下のコマンドを実行してください。

```bash
# トヨタ(7203)を例に、LLMとニュースをオフにして実行
tickwise -t 7203.T --no-llm --no-news
```

### 6-2. 成功の目安
以下のような出力が得られれば成功です：
- [x] Tickwise が起動する
- [x] トヨタ(7203.T)の株価データが取得される
- [x] 各種テクニカル指標の計算結果が表示される

---

# 7. 応用：実行環境の安定化（推奨）

ショートカットやタスクスケジューラから起動する場合、カレントディレクトリが意図せず変更され、設定ファイルやログのパスが不安定になることがあります。

> [!CAUTION]
> **tickwise.env の場所について**
> 現在のバージョンでは、`tickwise.env` は「コマンドを実行しているディレクトリ（カレントディレクトリ）」に置かれている必要があります。パス（PATH）を通した場所から実行する場合も、設定情報が必要な場合はその作業ディレクトリに `.env` を配置してください。

パス情報を環境変数で**絶対パス**として指定すると、ファイルの参照が安定します。

### Windows (PowerShell) の例
```powershell
# 銘柄リスト（CSV）の場所を固定
$env:ALIAS_CSV="C:\tickwise\data\tse_list.csv"

# ログ出力先を固定
$env:LOG_DIR="C:\tickwise\logs"
```

### macOS / Linux の例
```bash
# 銘柄リスト（CSV）の場所を固定
export ALIAS_CSV="$HOME/tickwise/data/tse_list.csv"

# ログ出力先を固定
export LOG_DIR="$HOME/tickwise/logs"
```

---

# 8. トラブルシューティング

- **CSV が文字化けする**
  - 保存時に「CSV UTF-8」を選択しているか再確認してください。
- **`tickwise.env` が認識されない**
  - ファイル名が正しいか、実行バイナリと同じ階層にあるか確認してください。
- **通信エラーが出る**
  - 社内 LAN 等でプロキシ設定が必要な環境では、Web 取得がブロックされる場合があります。