Tickwise

Rust製・高速CLIのテクニカル×ニュース要約アナライザー。
MACD / RSI / EMA / SMA / ボリンジャー / ROC / ADX / ストキャス / フィボ / VWAP / 一目 に対応。
価格データ→スコアリング→総合判定。ニュースは要約して“投資判断の文脈”へ。

なぜ Tickwise？

速い：Rust + 並列I/O。バッチにも日次運用にも◎

精度重視：前日終値は 確定値のみ を採用（曖昧な代替値は使わない）

素直な設計：指標ごとに重み付け→総合スコアを明示

LLMは任意：テクニカル＋ニュースを渡して要約。オフでもフル動作

成果物が残る：CSV / JSON / 標準出力で記録しやすい

セキュア：TechnicalDataGuard で出力／保存範囲を制御

信頼性：cargo audit / cargo deny クリーン、CycloneDX SBOM 同梱

クイックスタート（1分）
# 例: AAPL を解析（ニュース/LLMはオフ）
tickwise --release -- --ticker AAPL --save-technical-log --log_format csv --no_news --no_llm


主なオプション：

--ticker <TICKER>：例 AAPL, 7203.T

--save-technical-log：CSV/JSON ログ保存（--log_format csv|json）

--no_news / --no_llm：ニュース・LLM要約をスキップ

--silent：画面出力抑制（バッチ運用向け）

すべての指標はデフォルト有効。重みは tickwise.env で調整可能。

ニュース要約（任意）

BRAVE_API_KEY（任意）でニュース検索

OPENAI_API_KEY（任意）で LLM 要約

キー未設定時は自動でスキップ。テクニカル解析のみで完結します。

データポリシー（重要）

前日終値：確定値 のみ採用。欠損時はエラーとして扱い、推定・代替値で置換しません。

テクニカル計算は取得した OHLC/終値系列 のみを使用（ベンダ固有の軽量フィールドは不採用）。

出力

標準出力：総合スコア、各指標の評価

CSV/JSON：時系列＋スコアを機械可読で保存（ダッシュボード連携向け）

セキュリティ & SBOM

cargo audit / cargo deny パス

CycloneDX SBOM（sbom.cdx.json）をリリースに同梱

VT スキャン済み（バイナリハッシュをリリースノートに記載）

ライセンス

MIT License

作者

Koz（個人開発）