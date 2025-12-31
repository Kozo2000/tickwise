# Tickwise コマンドラインリファレンス

tickwise は、株価のテクニカル分析＋ニュース要約を行う Rust 製 CLI ツールです。
このドキュメントでは、**各オプションが「何をするためのものか」**と、型・既定値・環境変数との関係をまとめます。

# 0. 基本的な呼び出し例
もっとも標準的：テクニカル＋ニュース＋LLM
tickwise -t 7203.T

ニュース・LLMなしでテクニカルだけ
tickwise -t AAPL --no-news --no-llm

ログの CSV ヘッダだけ確認
tickwise --show-log-header


--show-log-header を使うときだけ、--ticker は省略可能です。

# 1. 基本オプション
-t, --ticker <SYMBOL>

目的: 分析対象の銘柄（ティッカー）を指定する。

型: String（内部では Option<String>）

必須:

通常: 必須

--show-log-header 使用時のみ省略可

既定値:

CLI で指定があればその値

--show-log-header 単独時のみ、内部的に "SPY" を使用

環境変数: なし

備考:

日本株: 7203.T

米株: AAPL, MSFT

特殊文字を含む指数など: 'S&P500' のようにクォートで囲む

# 2. テクニカル閾値関連
--buy-rsi <VALUE>

目的: 「売られすぎ（買い検討）」とみなす RSI 閾値を指定する。

型: f64

既定値: 30.0

環境変数: BUY_RSI

決定ルール:

CLI 指定あり → その値

CLI 指定なし（= 30.0 のまま） → BUY_RSI があればそれ、なければ 30.0

--sell-rsi <VALUE>

目的: 「買われすぎ（売り／利確検討）」とみなす RSI 閾値を指定する。

型: f64

既定値: 70.0

環境変数: SELL_RSI

決定ルール:

CLI 指定あり → その値

指定なし → SELL_RSI or 70.0

--macd-diff-low <VALUE>

目的: MACD と Signal の乖離が「小さい」とみなす閾値（転換近辺検知）。

型: f64

既定値: 2.0

環境変数: MACD_DIFF_LOW

決定ルール:

CLI 指定あり → その値

指定なし → MACD_DIFF_LOW or 2.0

--macd-diff-mid <VALUE>

目的: MACD 乖離が「中程度」とみなす閾値（勢いの強さの段階分け）。

型: f64

既定値: 10.0

環境変数: MACD_DIFF_MID

決定ルール:

CLI 指定あり → その値

指定なし → MACD_DIFF_MID or 10.0

-m, --macd-minus-ok

目的: MACD がマイナス圏でも、条件付きで買いシグナルを許可する（逆張り寄り）。

型: bool

既定値: false

環境変数: MACD_MINUS_OK

決定ルール:

true になる条件:

CLI で --macd-minus-ok を指定 または

MACD_MINUS_OK が真

# 3. スタンス（視点）の指定
--stance <buyer|seller|holder>

目的: コメントや評価を「買い手／売り手／ホルダー」どの視点で出すか決める。

型: String

許可値: buyer, seller, holder

既定値（clap）: holder

環境変数: STANCE

決定ルール:

CLI で buyer / seller を指定 → そのまま使用

CLI で holder のまま → STANCE があればそれ、なければ holder

# 4. 拡張テクニカル指標 ON/OFF

どの指標をスコアに含めるかを選択するフラグ群。
CLI フラグ or 環境変数が true なら有効化。

共通仕様:

型: bool

既定値: false

有効化条件:
CLI フラグを付ける または 対応 ENV が真 (get_bool_env)。

指標ごとのフラグ

オプション	目的	ENV

--ema	EMA（指数平滑移動平均）の分析を有効化	EMA

--sma	SMA（単純移動平均）の分析を有効化	SMA

--roc	ROC（変化率）の分析を有効化	ROC

--adx	ADX（トレンド強度）の分析を有効化	ADX

--stochastics	ストキャスティクス（%K, %D）の分析を有効化	STOCHASTICS

--bollinger	ボリンジャーバンドの分析を有効化	BOLLINGER

--fibonacci	フィボナッチ・リトレースメントの分析を有効化	FIBONACCI

--vwap	VWAP の分析を有効化	VWAP

--ichimoku	一目均衡表の分析を有効化	ICHIMOKU


--bb-bandwidth-squeeze-pct <VALUE>

目的: ボリンジャーバンド帯域幅（%）が「スクイーズ」と判定される閾値を指定。

型: f64

既定値: 8.0

環境変数: なし

その他:

0.0〜100.0 の範囲にサニタイズされる（sanitize_percent）

# 5. スコアの重み付け（Weight）

各カテゴリのスコアに掛ける「重み」。
1.0 が標準。大きくするとその指標を強調、小さくすると弱める。

共通仕様:

型: f64

既定値: 1.0

環境変数: WEIGHT_○○ 系

決定ルール（全て同じ）:

CLI で 1.0 以外を指定 → CLI 優先

CLI が 1.0 のまま → ENV があれば ENV 優先

どちらもなければ 1.0

オプション	目的	ENV

--weight-basic	基本スコア（RSI, MACD 等）の重み	WEIGHT_BASIC

--weight-ema	EMA スコアの重み	WEIGHT_EMA

--weight-sma	SMA スコアの重み	WEIGHT_SMA

--weight-bollinger	ボリンジャーバンドスコアの重み	WEIGHT_BOLLINGER

--weight-roc	ROC スコアの重み	WEIGHT_ROC

--weight-adx	ADX スコアの重み	WEIGHT_ADX

--weight-stochastics	ストキャスティクススコアの重み	WEIGHT_STOCHASTICS

--weight-fibonacci	フィボナッチスコアの重み	WEIGHT_FIBONACCI

--weight-vwap	VWAP スコアの重み	WEIGHT_VWAP

--weight-ichimoku	一目均衡表スコアの重み	WEIGHT_ICHIMOKU


# 6. LLM / OpenAI 関連
-O, --no-llm

目的: LLM（OpenAI）へのアクセスを完全にスキップする。

型: bool

既定値: false

環境変数: NO_LLM

効果: ニュース要約などテキスト生成を行わず、テクニカル分析のみ実行。

--llm-provider <NAME>

目的: 使用する LLM プロバイダ名を指定（将来拡張用）。

型: String

許可値: openai

既定値:

CLI が openai のまま → LLM_PROVIDER or "openai"

備考: 現バージョンでは実質 openai 固定。

-M, --openai-model <MODEL_NAME>

目的: 使用する OpenAI モデル名を指定する。

型: String

既定値: "gpt-4.1-nano"

環境変数: OPENAI_MODEL

決定ルール:

CLI 指定あり → そのモデル名

指定なし → OPENAI_MODEL or "gpt-4.1-nano"

--openai-api-key <KEY>

目的: OpenAI API キーを CLI から直接指定する。

型: Option<String>

既定値: なし

環境変数: OPENAI_API_KEY

決定ルール:

CLI 指定あり → そのキー

指定なし → OPENAI_API_KEY or 空文字

-x, --openai-extra-note <TEXT>

目的: LLM へ送るプロンプトに「一言スタイル指定」を追加する。

型: Option<String>

既定値: なし

環境変数: OPENAI_EXTRA_NOTE

例:

"配当より成長性を重視して評価して"

"短期トレード目線を強めにして欲しい"

-d, --debug-prompt

目的: 実際には API を叩かず、プロンプトだけ debug_prompt.txt に保存する。

型: bool

既定値: false

環境変数: DEBUG_PROMPT

用途: プロンプト設計のデバッグ / LLM なしで中身だけ確認したいとき。

# 7. ニュース / Brave 関連
--brave-api-key <KEY>

目的: Brave News API のキーを CLI から指定する。

型: Option<String>

既定値: なし

環境変数: BRAVE_API_KEY

-n, --no-news

目的: ニュース検索そのものをスキップする。

型: bool

既定値: false

環境変数: NO_NEWS

-q, --custom-news-query <QUERY>

目的: ニュース検索クエリを銘柄コード以外の自由な文字列で指定する。

型: Option<String>

既定値: なし

環境変数: CUSTOM_NEWS_QUERY

備考: 空文字・空白のみは無視される。

--news-filter

目的: ニュース検索を投資・財務系キーワードに寄せる。

型: bool

既定値: false

環境変数: NEWS_FILTER

--news-count <N>

目的: 取得するニュース件数の上限を指定する。

型: Option<usize> → Config では usize

範囲: 1..=50

環境変数: NEWS_COUNT

既定値:

CLI 指定あり → その値（1〜50に clamp）

指定なし & NEWS_COUNT あり → それ（clamp）

どちらもなし →

news_filter 有効: 20

無効: 50

--news-freshness <pd|pw|pm|py|all>

目的: ニュースの期間（どこまで遡るか）を指定する。

型: Option<String> → Config では String

許可値:

pd (1 日) / pw (1 週間) / pm (1 ヶ月) / py (1 年) / all (制限なし)

環境変数: NEWS_FRESHNESS

既定値:

CLI 指定あり → その値

指定なし & NEWS_FRESHNESS あり → それ

どちらもなし →

news_filter 有効: "pw"

無効: "pm"

--show-news

目的: ニュース本文のハイライトをターミナルに表示する。

型: bool

既定値: false

環境変数: SHOW_NEWS

# 8. ログ / 出力関連
--save-technical-log

目的: テクニカル分析結果をログ（CSV/JSON）として保存する。

型: bool

既定値: false

環境変数: SAVE_TECHNICAL_LOG

--log-format <csv|json>

目的: テクニカルログの形式を指定する。

型: String

既定値: "csv"

環境変数: LOG_FORMAT（※異常値を入れたときだけ使用される）

備考:

CLI で "csv" or "json" を指定（既定含む）した場合、そのまま使用。

それ以外の文字列を CLI に入れたときのみ LOG_FORMAT or "csv" にフォールバック。

--log-dir <DIR>

目的: ログを保存するディレクトリを指定する。

型: String

既定値: "log"

環境変数: LOG_DIR

決定ルール:

CLI 指定あり → そのディレクトリ

指定なし → LOG_DIR or "log"

--data-append

目的: 既存 CSV に追記する（新規ファイルを作らない）。

型: bool

既定値: false

環境変数: CSV_APPEND

--log-flat

目的: ティッカーごとのサブディレクトリを作らず、すべて同じディレクトリにログを出す。

型: bool

既定値: false

環境変数: LOG_FLAT

--stdout-log

目的: ログ（CSV/JSON）をファイルではなく標準出力に出す。

型: bool

既定値: false

環境変数: なし

用途: jq や他ツールとのパイプ連携向け。

--show-log-header

目的: 現在の設定に基づいた CSV のヘッダ行だけを出力して終了する。

型: bool

既定値: false

環境変数: なし

特記事項:

このオプション単独のときだけ --ticker を省略可能（内部的には "SPY" を使用）。

--silent

目的: 通常の出力をすべて抑制し、エラーのみ表示する。

型: bool

既定値: false

環境変数: なし

用途: cron／バッチ実行時などでログだけ残したい場合。

# 9. プロンプト長さ（LLM テキスト量調整）

LLM に渡す各セクションの最大文字数。ターミナル幅や読みやすさに応じて調整。

共通仕様:

型: usize

環境変数: MAX_○○_LENGTH 系

オプション	目的	既定値	ENV

--max-note-length	「注意ポイント」最大文字数	300	MAX_NOTE_LENGTH

--max-shortterm-length	「1週間短期目線」最大文字数	150	MAX_SHORTTERM_LENGTH

--max-midterm-length	「1ヶ月中期目線」最大文字数	150	MAX_MIDTERM_LENGTH

--max-news-length	「ニュースハイライト」最大文字数	600	MAX_NEWS_LENGTH

--max-review-length	「総評」最大文字数	1000	MAX_REVIEW_LENGTH


# 10. エイリアス・デバッグ系
--alias-csv <PATH>

目的: 銘柄名エイリアス定義 CSV のパスを指定する。

型: Option<String>

既定値: なし

環境変数: ALIAS_CSV

-a, --no-alias

目的: エイリアス展開をスキップし、ティッカー／正式名称だけでニュース検索する。

型: bool

既定値: false

環境変数: なし

--debug-args

目的: パース済みのコマンドライン引数内容を表示する（デバッグ用）。

型: bool

既定値: false

環境変数: なし