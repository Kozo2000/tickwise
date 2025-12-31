# 1. 基本
-t, --ticker <STRING>

型: String（Option<String>）

必須:

通常: 必須（required_unless_present = "show_log_header"）

--show-log-header 使用時のみ省略可

既定値:

通常: なし（指定が必要）

--show-log-header 単独時のみ、内部的に "SPY" が使われる
（ticker: args.ticker.clone().unwrap_or_else(|| "SPY".to_string())）

環境変数: なし

# 2. テクニカル閾値
--buy-rsi <VALUE>

型: f64

既定値（clap）: 30.0

環境変数: BUY_RSI

決定ロジック:

CLI で --buy-rsi を指定した場合: その値が優先

指定なし（= 30.0 のまま）の場合:
BUY_RSI があればそれを f64 として採用、なければ 30.0

--sell-rsi <VALUE>

型: f64

既定値: 70.0

環境変数: SELL_RSI

決定ロジック:

CLI 指定あり → その値

指定なし → SELL_RSI or 70.0

--macd-diff-low <VALUE>

型: f64

既定値: 2.0

環境変数: MACD_DIFF_LOW

決定ロジック:

CLI 指定あり → その値

指定なし → MACD_DIFF_LOW or 2.0

--macd-diff-mid <VALUE>

型: f64

既定値: 10.0

環境変数: MACD_DIFF_MID

決定ロジック:

CLI 指定あり → その値

指定なし → MACD_DIFF_MID or 10.0

-m, --macd-minus-ok

型: bool

既定値: false

環境変数: MACD_MINUS_OK

決定ロジック:

true になる条件:

CLI フラグ --macd-minus-ok を付ける または

MACD_MINUS_OK が真（get_bool_env が true）

# 3. スタンス
--stance <buyer|seller|holder>

型: String

許可値（clap）: "buyer", "seller", "holder"

既定値（clap）: "holder"

環境変数: STANCE

決定ロジック:

CLI で --stance を buyer / seller に指定 → その値が優先

CLI で holder のまま（デフォルト）の場合 →
STANCE があればそれを使用、なければ "holder"

その後 parse_stance(&stance_source) で内部 enum に変換

# 4. 拡張テクニカル指標フラグ
共通仕様

型: すべて bool

既定値: すべて false

環境変数:

EMA

SMA

ROC

ADX

STOCHASTICS

BOLLINGER

FIBONACCI

VWAP

ICHIMOKU

決定ロジック（すべて同じパターン）:

if args.ema || get_bool_env("EMA") {
    extensions.push(ExtensionIndicator::Ema);
}


つまり:

CLI で --ema を立てる

or 環境変数 EMA=true
→ いずれかで有効化され、enabled_extensions に入る

対象フラグ一覧

--ema

--sma

--roc

--adx

--stochastics

--bollinger

--fibonacci

--vwap

--ichimoku

--bb-bandwidth-squeeze-pct <VALUE>

型: f64

既定値: 8.0

環境変数: なし（sanitize_percent のみ）

決定ロジック:

CLI or デフォルト 8.0 を受け取り、sanitize_percent(value, 0.0, 100.0, "...") で
0.0〜100.0 の範囲にクリップ

# 5. Weight（重み付け）
共通仕様

型: f64

既定値（clap）: 1.0

環境変数:

WEIGHT_BASIC

WEIGHT_EMA

WEIGHT_SMA

WEIGHT_BOLLINGER

WEIGHT_ROC

WEIGHT_ADX

WEIGHT_STOCHASTICS

WEIGHT_FIBONACCI

WEIGHT_VWAP

WEIGHT_ICHIMOKU

決定ロジック: get_f64_from_args_or_env(arg_value, ENV_NAME, 1.0)

実装パターンから見ると:

CLI 側が既定値 1.0 のまま → ENV があれば ENV 優先

CLI で指定 → CLI 優先

どちらもなし → 1.0

対象オプション

--weight-basic

--weight-ema

--weight-sma

--weight-bollinger

--weight-roc

--weight-adx

--weight-stochastics

--weight-fibonacci

--weight-vwap

--weight-ichimoku

# 6. LLM / OpenAI 関連
-O, --no-llm

型: bool

既定値: false

環境変数: NO_LLM

決定ロジック:

no_llm: args.no_llm || get_bool_env("NO_LLM")

--llm-provider <NAME>

型: String

既定値（clap）: "openai"

許可値（clap）: "openai"

環境変数: LLM_PROVIDER

決定ロジック:

llm_provider: if args.llm_provider == "openai" {
    std::env::var("LLM_PROVIDER").unwrap_or_else(|_| "openai".to_string())
} else {
    args.llm_provider.clone()
}


現状、clap 側で "openai" しか許可されていないため、
実質 "openai" 固定（将来の拡張用フック）。

-M, --openai-model <MODEL_NAME>

型: String

既定値（clap）: "gpt-4.1-nano"

環境変数: OPENAI_MODEL

決定ロジック:

CLI 指定あり → そのモデル名

指定なし → OPENAI_MODEL or "gpt-4.1-nano"

--openai-api-key <KEY>

型: Option<String>

既定値: なし

環境変数: OPENAI_API_KEY

決定ロジック:

CLI 指定あり → そのキー

指定なし → OPENAI_API_KEY or ""（空文字）

--openai-extra-note <TEXT>

型: Option<String>

既定値: なし

環境変数: OPENAI_EXTRA_NOTE

決定ロジック:

CLI 指定あり → その文字列

指定なし → OPENAI_EXTRA_NOTE or None

-d, --debug-prompt

型: bool

既定値: false

環境変数: DEBUG_PROMPT

決定ロジック:

debug_prompt: args.debug_prompt || get_bool_env("DEBUG_PROMPT")

# 7. Brave / ニュース関連
--brave-api-key <KEY>

型: Option<String>

既定値: なし

環境変数: BRAVE_API_KEY

決定ロジック:

CLI 指定あり → そのキー

指定なし → BRAVE_API_KEY or ""

-n, --no-news

型: bool

既定値: false

環境変数: NO_NEWS

決定ロジック:

no_news: args.no_news || get_bool_env("NO_NEWS")

-q, --custom-news-query <QUERY>

型: Option<String>

既定値: None

環境変数: CUSTOM_NEWS_QUERY

決定ロジック:

CLI 指定あり → その文字列

指定なし → CUSTOM_NEWS_QUERY（空 or 空白のみなら破棄）

--news-filter

型: bool

既定値: false

環境変数: NEWS_FILTER

決定ロジック:

news_filter: args.news_filter || get_bool_env("NEWS_FILTER")

--show-news

型: bool

既定値: false

環境変数: SHOW_NEWS

決定ロジック:

show_news: args.show_news || get_bool_env("SHOW_NEWS")

--news-count <N>

型: Option<usize>（ただし Config では usize）

許容範囲: 1..=50（clamp(1, 50)）

既定値:

NEWS_COUNT があればそれを使用（clamp 1..50）

それもなければ:

news_filter 有効（CLI or ENV）: 20

news_filter 無効: 50

環境変数: NEWS_COUNT

決定ロジック:

news_count: match args.news_count {
    Some(n_count) => n_count.clamp(1, 50),
    None => env::var("NEWS_COUNT")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .map(|n_count| n_count.clamp(1, 50))
        .unwrap_or(if args.news_filter || get_bool_env("NEWS_FILTER") {
            20
        } else {
            50
        }),
}

--news-freshness <pd|pw|pm|py|all>

型: Option<String>（Config では String）

許可値: "pd", "pw", "pm", "py", "all"

既定値:

NEWS_FRESHNESS があればそれ

なければ:

news_filter 有効: "pw"

無効: "pm"

環境変数: NEWS_FRESHNESS

# 8. ログ / 出力関連
--save-technical-log

型: bool

既定値: false

環境変数: SAVE_TECHNICAL_LOG

決定ロジック:

save_technical_log: args.save_technical_log || get_bool_env("SAVE_TECHNICAL_LOG")

--log-format <csv|json>

型: String

既定値（clap）: "csv"

環境変数: LOG_FORMAT（※現行コードではほぼ使われない）

決定ロジック:

log_format: if args.log_format != "csv" && args.log_format != "json" {
    env::var("LOG_FORMAT").unwrap_or_else(|_| "csv".to_string())
} else {
    args.log_format.clone()
}


実質:

CLI で "csv" / "json" を指定（またはデフォルト "csv"） → そのまま使用

それ以外の文字列を CLI で渡したときだけ LOG_FORMAT or "csv"

つまり 通常運用では LOG_FORMAT 環境変数は効かない

--log-dir <DIR>

型: String

既定値（clap）: "log"

環境変数: LOG_DIR

決定ロジック:

CLI で --log-dir 指定あり → その値

指定なし（＝ "log" のまま） → LOG_DIR or "log"

--data-append

型: bool

既定値: false

環境変数: CSV_APPEND

決定ロジック:

data_append: args.data_append || get_bool_env("CSV_APPEND")

--log-flat

型: bool

既定値: false

環境変数: LOG_FLAT

決定ロジック:

log_flat: args.log_flat || get_bool_env("LOG_FLAT")

--stdout-log

型: bool

既定値: false

環境変数: なし
（stdout_log: args.stdout_log）

--show-log-header

型: bool

既定値: false

環境変数: なし

補足:

required_unless_present = "show_log_header" により
--show-log-header 単体のときだけ ticker が不要

このとき内部では ticker = "SPY" が使われる

--silent

型: bool

既定値: false

環境変数: なし
（silent: args.silent）

--debug-args

型: bool

既定値: false

環境変数: なし
（debug_args: args.debug_args）

# 9. プロンプト長さ（LLM 側テキスト）
共通仕様

型: usize

既定値:

max_note_length: 300

max_shortterm_length: 150

max_midterm_length: 150

max_news_length: 600

max_review_length: 1000

環境変数:

MAX_NOTE_LENGTH

MAX_SHORTTERM_LENGTH

MAX_MIDTERM_LENGTH

MAX_NEWS_LENGTH

MAX_REVIEW_LENGTH

決定ロジック: get_usize_from_args_or_env(arg_value, ENV_NAME, default)

# 10. エイリアス関連
--alias-csv <PATH>

型: Option<String>

既定値: なし

環境変数: ALIAS_CSV

決定ロジック:

CLI 指定あり → そのパス

指定なし → ALIAS_CSV or None

-a, --no-alias

型: bool

既定値: false

環境変数: なし
（Config には直接出てこないが、ニュース検索側の挙動制御用）

# 11. デバッグ系その他

-n, --no-news → 上述ニュース系

-O, --no-llm → 上述 LLM 系

--silent / --debug_args → 上述