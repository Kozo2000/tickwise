## 概要

**Tickwise**は、Rust製のコマンドライン株式テクニカル分析ツールです。  
MACD、RSI、EMA、SMA、ボリンジャーバンド、ROC、ADX、ストキャスティクス、フィボナッチ、VWAP、一目均衡表など多様な指標に対応し、  
API経由で株価データを取得し、スコアリング・重み付け・総合判定を行います。  
ニュース要約（Brave API/OpenAI連携）やCSV/JSON出力も可能です。

---

## 主な機能

- 多様なテクニカル指標による分析・スコア算出
- 指標ごとの重み付け・総合スコア計算
- API経由で株価データ取得（Yahoo Finance/MarketStack）
- ニュース要約（Brave API/OpenAI連携）
- CSV/JSON/標準出力/ファイル保存対応
- コマンドラインオプションによる柔軟な動作切り替え
- セキュアなデータ管理（TechnicalDataGuard構造体）

---

## 使い方

1. 必要なAPIキー（MarketStack, Brave, OpenAI）を環境変数または`.env`ファイルに設定
2. コマンドラインから実行例：

   ```sh
   cargo run --release -- --ticker AAPL --ema --sma --bollinger --save-technical-log --log_format csv
   ```

   主なオプション例：
   - `--ticker <TICKER>` : 銘柄指定（例: AAPL, 7203.T）
   - `--ema`, `--sma`, `--bollinger` : 指標ごとの分析有効化
   - `--save-technical-log` : ログ保存（CSV/JSON）
   - `--no_news` : ニュース要約をスキップ
   - `--no_llm` : LLM要約をスキップ
   - `--silent` : 画面出力を抑制（バッチ用途）

---

## 必要な環境変数

- `MARKETSTACK_API_KEY` : MarketStack APIキー
- `BRAVE_API_KEY` : Brave News APIキー（任意）
- `OPENAI_API_KEY` : OpenAI APIキー（任意）

---

## ライセンス

MIT License

---

## 作者

サンデープログラマ_Koz（個人開発）
