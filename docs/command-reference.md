# Tickwise コマンドラインリファレンス

Tickwiseは、株価のテクニカル分析とニュース要約を行うCLIツールです。本リファレンスでは、各オプションの機能、型、既定値、および環境変数との対応関係を記述します。

## 1. 基本操作

### 実行例
- **標準的な実行（テクニカル + ニュース + LLM）**
  ```bash
  tickwise --ticker 7203.T
  ```
- **テクニカル分析のみ（ニュース・LLMスキップ）**
  ```bash
  tickwise --ticker AAPL --no-news --no-llm
  ```
- **CSVヘッダーの出力（バッチ処理の準備等）**
  ```bash
  tickwise --show-log-header
  ```

---

## 2. 基本オプション

| オプション | 短縮 | 説明 | 型 | 既定値 | 環境変数 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `--ticker` | `-t` | 分析対象のティッカー記号（例: 7203.T, AAPL） | String | (必須※) | - |
| `--no-env-indicators` | `-I` | `tickwise.env` からの指標設定読み込みを無効化 | bool | false | - |
| `--stance` | - | 分析の視点（`buyer`, `seller`, `holder`） | String | `holder` | `STANCE` |
| `--show-log-header` | - | ログのCSVヘッダーを表示して終了 | bool | false | - |

※ `--show-log-header` 指定時を除き、`--ticker` は必須です。未指定でヘッダー表示を行う場合は内部的に `SPY` が使用されます。

---

## 3. テクニカル分析設定

### 閾値設定
各指標の判定基準となる数値を指定します。

| オプション | 説明 | 型 | 既定値 | 環境変数 |
| :--- | :--- | :--- | :--- | :--- |
| `--buy-rsi` | RSIの「売られすぎ」閾値 | f64 | 30.0 | `BUY_RSI` |
| `--sell-rsi` | RSIの「買われすぎ」閾値 | f64 | 70.0 | `SELL_RSI` |
| `--macd-diff-low` | MACDとSignalの乖離「小」の閾値 | f64 | 2.0 | `MACD_DIFF_LOW` |
| `--macd-diff-mid` | MACDとSignalの乖離「中」の閾値 | f64 | 10.0 | `MACD_DIFF_MID` |
| `--macd-minus-ok` (`-m`) | MACDマイナス圏での買い判定を許可 | bool | false | `MACD_MINUS_OK` |

### 拡張指標の有効化
フラグを指定するか、環境変数を `true` に設定することで有効になります。

| オプション | 指標 | 環境変数 |
| :--- | :--- | :--- |
| `--ema` | 指数平滑移動平均 (EMA) | `EMA` |
| `--sma` | 単純移動平均 (SMA) | `SMA` |
| `--roc` | 変化率 (ROC) | `ROC` |
| `--adx` | トレンド強度 (ADX) | `ADX` |
| `--stochastics` | ストキャスティクス (%K, %D) | `STOCHASTICS` |
| `--bollinger` | ボリンジャーバンド | `BOLLINGER` |
| `--fibonacci` | フィボナッチ・リトレースメント | `FIBONACCI` |
| `--vwap` | VWAP（日足簡易版） | `VWAP` |
| `--ichimoku` | 一目均衡表（転換線・基準線） | `ICHIMOKU` |

### 重み付け (Weight)
各カテゴリのスコアに対する倍率（0.5～3.0推奨）を指定します。

| オプション | 対象カテゴリ | 既定値 | 環境変数 |
| :--- | :--- | :--- | :--- |
| `--weight-basic` | 基本スコア (RSI, MACD) | 1.0 | `WEIGHT_BASIC` |
| `--weight-ema` | EMA スコア | 1.0 | `WEIGHT_EMA` |
| `--weight-sma` | SMA スコア | 1.0 | `WEIGHT_SMA` |
| `--weight-bollinger`| ボリンジャーバンド スコア | 1.0 | `WEIGHT_BOLLINGER` |
| `--weight-roc` | ROC スコア | 1.0 | `WEIGHT_ROC` |
| `--weight-adx` | ADX スコア | 1.0 | `WEIGHT_ADX` |
| `--weight-stochastics`| ストキャスティクス スコア | 1.0 | `WEIGHT_STOCHASTICS` |
| `--weight-fibonacci`| フィボナッチ スコア | 1.0 | `WEIGHT_FIBONACCI` |
| `--weight-vwap` | VWAP スコア | 1.0 | `WEIGHT_VWAP` |
| `--weight-ichimoku`| 一目均衡表 スコア | 1.0 | `WEIGHT_ICHIMOKU` |

---

## 4. ニュース・LLM設定

### 外部サービス連携
| オプション | 短縮 | 説明 | 型 | 既定値 | 環境変数 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `--no-llm` | `-O` | LLM（OpenAI）アクセスをスキップ | bool | false | `NO_LLM` |
| `--no-news` | `-n` | ニュース検索をスキップ | bool | false | `NO_NEWS` |
| `--openai-model` | `-M` | 使用するOpenAIモデル名 | String | `gpt-4.1-nano` | `OPENAI_MODEL` |
| `--openai-api-key` | - | OpenAI API キー | String | - | `OPENAI_API_KEY` |
| `--brave-api-key` | - | Brave Search API キー | String | - | `BRAVE_API_KEY` |

### 検索・プロンプト調整
- `--custom-news-query` (`-q`): 独自の検索クエリを指定します。
- `--news-filter`: クエリを財務用語（財務、決算等）で絞り込みます。
- `--news-count`: 取得件数（1～50）。既定はフィルタON時20、OFF時50。
- `--news-freshness`: 検索期間 (`pd`:1日, `pw`:1週, `pm`:1月, `py`:1年, `all`)。
- `--openai-extra_note` (`-x`): LLMへの指示に追加のメモを付与します。

---

## 5. ログ・出力設定

| オプション | 説明 | 型 | 既定値 | 環境変数 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `--save-technical-log` | 分析結果のファイル保存を有効化 | bool | false | `SAVE_TECHNICAL_LOG` |
| `--log-format` | ログ形式（`csv` または `json`） | String | `csv` | `LOG_FORMAT` |
| `--log-dir` | 保存先ディレクトリ | String | `log` | `LOG_DIR` |
| `--stdout-log` | ログをファイルではなく標準出力に書き出す | bool | false | - |
| `--data-append` | 既存のCSVファイルに追記する | bool | false | `CSV_APPEND` |
| `--log-flat` | ティッカー別のサブディレクトリを作成しない | bool | false | `LOG_FLAT` |
| `--silent` | 標準出力を抑制（エラーのみ表示） | bool | false | - |

---

## 6. 付録：優先順位の定義

各設定値は、以下の優先順位に従って決定されます。

1. **コマンドライン引数 (CLI)**: 指定がある場合、最優先されます。
2. **環境変数 (tickwise.env / shell)**: CLIでの指定がなく、環境変数が定義されている場合に採用されます。
3. **ハードコードされた既定値**: 上記のいずれも指定がない場合に適用されます。

---

## 免責事項
本ツールおよび本リファレンスに記載された情報は、情報提供のみを目的としており、投資勧誘を意図したものではありません。本ツールの分析結果を用いた投資判断は、利用者自身の責任において行ってください。本ツールの利用により生じたいかなる損失についても、開発者は一切の責任を負いません。