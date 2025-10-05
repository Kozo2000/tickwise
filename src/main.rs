use chrono::Local;
use chrono::TimeZone;
use clap::Parser;
use colored::*;
use csv::ReaderBuilder;
use dotenvy::from_filename;
use dotenvy::from_path;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::read_to_string;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Cursor;
use std::io::{BufWriter, Write};
use std::path::Path;
use ta::indicators::{BollingerBands, MovingAverageConvergenceDivergence, RelativeStrengthIndex};
use ta::Next;
use tempfile::NamedTempFile; // JSON用

type BuildCfgResult = Result<(Config, String, HashMap<String, String>), Box<dyn std::error::Error>>;
const EMA_EQ_EPS: f64 = 0.01; // 短期-長期の絶対差が±0.01未満なら「同値圏」
                              //const NEUTRAL_DEADBAND: f64 = 0.05; // 中立の揺れ幅（±5% 未満なら見送り/様子見）

/// コマンドライン引数の構造定義
#[derive(Parser, Debug)]
#[command(name = "tickwise", version, about = "Stock Technical Analysis Tool")]
struct Args {
    #[arg(
        short = 't',
        long,
        required_unless_present = "show_log_header",
        help = "Specify ticker symbol (e.g., AAPL, MSFT, 7203.T). Note: If the ticker contains special characters (e.g., '&'), enclose it in quotes. Example: 'S&P500'"
    )]
    ticker: Option<String>,
    #[arg(
        long,
        help = "Specify your own API key (if not using environment variable)"
    )]
    openai_api_key: Option<String>,
    #[arg(
        long,
        default_value_t = 30.0,
        help = "RSI threshold to detect oversold (default: 30)"
    )]
    buy_rsi: f64,
    #[arg(
        long,
        default_value_t = 70.0,
        help = "RSI threshold to detect overbought (default: 70)"
    )]
    sell_rsi: f64,
    #[arg(
        long,
        default_value_t = 2.0,
        help = "Threshold for MACD small difference (default: 2)"
    )]
    macd_diff_low: f64,
    #[arg(
        long,
        default_value_t = 10.0,
        help = "Threshold for MACD medium difference (default: 10)"
    )]
    macd_diff_mid: f64,
    #[arg(short = 'O', long, help = "Skip LLM access completely")]
    no_llm: bool,
    #[arg(long,
    default_value = "openai",
    value_parser = ["openai"], // ← いまは openai のみ
    help = "LLM provider (only 'openai' supported in this version)")]
    llm_provider: String,
    #[arg(
        short = 'm',
        long,
        help = "Allow buy signals even if MACD is in negative zone"
    )]
    macd_minus_ok: bool,

    #[arg(
        short = 'd',
        long,
        help = "Save OpenAI prompt to a file (debug_prompt.txt) without sending it"
    )]
    debug_prompt: bool,
    #[arg(
        short = 'M',
        long,
        default_value = "gpt-4.1-nano",
        help = "Specify OpenAI model to use"
    )]
    openai_model: String,
    #[arg(
        short = 'a',
        long,
        help = "Skip alias expansion and search news using ticker/formal name only"
    )]
    no_alias: bool,
    #[arg(
        short = 'n',
        long,
        help = "Skip news search entirely (technical analysis only)"
    )]
    no_news: bool,
    #[clap(long)]
    alias_csv: Option<String>,
    #[arg(long, help = "Enable EMA (Exponential Moving Average) analysis")]
    ema: bool,
    #[arg(long, help = "Enable SMA (Simple Moving Average) analysis")]
    sma: bool,
    #[arg(long, help = "Enable Bollinger Bands analysis")]
    bollinger: bool,
    #[arg(long, help = "Enable Fibonacci retracement analysis")]
    fibonacci: bool,
    #[arg(long, help = "Enable Stochastics (%K and %D) analysis")]
    stochastics: bool,
    #[arg(long, help = "Enable ADX (trend strength) analysis")]
    adx: bool,
    #[arg(long, help = "Enable ROC (Rate of Change) analysis")]
    roc: bool,
    #[arg(long, help = "Enable VWAP analysis")]
    vwap: bool,
    #[arg(long, help = "Enable Ichiomku analysis")]
    ichimoku: bool,
    #[arg(
        long,
        default_value_t = 8.0,
        help = "Bollinger Bandwidth threshold (%) for squeeze detection"
    )]
    bb_bandwidth_squeeze_pct: f64,
    #[arg(
        long,
        default_value_t = 1.0,
        help = "Weight multiplier for Basic score (0.5-3.0)"
    )]
    weight_basic: f64,
    #[arg(
        long,
        default_value_t = 1.0,
        help = "Weight multiplier for EMA score (0.5-3.0)"
    )]
    weight_ema: f64,
    #[arg(
        long,
        default_value_t = 1.0,
        help = "Weight multiplier for SMA score (0.5-3.0)"
    )]
    weight_sma: f64,
    #[arg(
        long,
        default_value_t = 1.0,
        help = "Weight multiplier for Bollinger score (0.5-3.0)"
    )]
    weight_bollinger: f64,
    #[arg(
        long,
        default_value_t = 1.0,
        help = "Weight multiplier for ROC score (0.5-3.0)"
    )]
    weight_roc: f64,
    #[arg(
        long,
        default_value_t = 1.0,
        help = "Weight multiplier for ADX score (0.5-3.0)"
    )]
    weight_adx: f64,
    #[arg(
        long,
        default_value_t = 1.0,
        help = "Weight multiplier for Stochastics score (0.5-3.0)"
    )]
    weight_stochastics: f64,
    #[arg(
        long,
        default_value_t = 1.0,
        help = "Weight multiplier for Fibonacci score (0.5-3.0)"
    )]
    weight_fibonacci: f64,
    #[arg(
        long,
        default_value_t = 1.0,
        help = "Weight multiplier for VWAP score (0.5-3.0)"
    )]
    weight_vwap: f64,
    #[arg(
        long,
        default_value_t = 1.0,
        help = "Weight multiplier for Ichimoku score (0.5-3.0)"
    )]
    weight_ichimoku: f64,

    #[arg(long, value_parser = ["buyer","seller","holder"], default_value = "holder",
      help = "視点を選択: buyer|seller|holder（既定: holder）")]
    stance: String,

    #[arg(
        long,
        help = "Specify your Brave API key (if not using environment variable)"
    )]
    brave_api_key: Option<String>,

    #[arg(short = 'q', long, help = "Specify a custom news search query")]
    custom_news_query: Option<String>,
    /// ニュースの検索ワードを財務用語で絞る（既定: False / 環境変数 NEWS_FILTER=True で有効化）
    #[arg(
        long,
        help = "Filter news query with finance terms (default: False; set NEWS_FILTER=True to enable)"
    )]
    news_filter: bool,
    /// ニュース取得件数の上限（1..50）。未指定ならモード既定（OFF:50 / ON:20）か ENV(NEWS_COUNT)
    #[arg(long, value_parser = clap::value_parser!(usize), help = "Max news items to fetch (1..50). Defaults: OFF=50, ON=20, or NEWS_COUNT")]
    news_count: Option<usize>,
    /// ニュースの期間(pd|pw|pm|py|all)。未指定ならモード既定（OFF:pm / ON:pw）か ENV(NEWS_FRESHNESS)
    #[arg(long, value_parser = ["pd","pw","pm","py","all"], help = "News freshness (pd|pw|pm|py|all). Defaults: OFF=pm, ON=pw, or NEWS_FRESHNESS")]
    news_freshness: Option<String>,
    /// ニュースを表示する（既定: env か false）
    #[arg(long, help = "Show news in terminal output")]
    show_news: bool,
    #[arg(short = 'x', long, help = "Add an extra note to the OpenAI prompt")]
    openai_extra_note: Option<String>,

    #[arg(long, help = "Save technical analysis log (CSV or JSON)")]
    save_technical_log: bool,
    #[arg(
        long,
        default_value = "csv",
        help = "Technical log format (csv or json)"
    )]
    log_format: String,
    #[arg(
        long,
        help = "Specify directory to save technical logs",
        default_value = "log"
    )]
    log_dir: String,
    #[arg(long, default_value_t = 300, help = "Max characters for 注意ポイント")]
    max_note_length: usize,
    #[arg(long, default_value_t = 150, help = "Max characters for 1週間短期目線")]
    max_shortterm_length: usize,
    #[arg(long, default_value_t = 150, help = "Max characters for 1ヶ月中期目線")]
    max_midterm_length: usize,
    #[arg(
        long,
        default_value_t = 600,
        help = "Max characters for ニュースハイライト"
    )]
    max_news_length: usize,
    #[arg(long, default_value_t = 1000, help = "Max characters for 総評")]
    max_review_length: usize,
    #[arg(
        long,
        help = "Append to existing CSV file instead of creating a new one"
    )]
    data_append: bool,
    #[arg(long, help = "Omit ticker-based subdirectory for log file")]
    log_flat: bool,
    #[arg(
        long,
        help = "Output log (CSV or JSON) to standard output instead of file"
    )]
    stdout_log: bool,
    #[arg(
        long,
        help = "Show only CSV header row based on current options and exit"
    )]
    show_log_header: bool,
    #[arg(
        long,
        help = "Suppress all normal output (except for errors) for batch execution"
    )]
    silent: bool,
    #[arg(
        long,
        help = "Display parsed command-line arguments (for debugging purposes)"
    )]
    debug_args: bool,
}
/// 拡張テクニカル指標の列挙型（ユーザ指定をベクタで受ける）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ExtensionIndicator {
    Ema,
    Sma,
    Bollinger,
    Roc,
    Adx,
    Stochastics,
    Fibonacci,
    Vwap,
    Ichimoku,
}
/// 拡張指標のカテゴリ分類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum IndicatorCategory {
    Trend,         // トレンド系指標
    Oscillator,    // オシレーター系指標
    Volatility,    // ボラティリティ系指標
    SupportResist, // サポート・レジスタンスなど補助指標
}

impl std::str::FromStr for ExtensionIndicator {
    type Err = String;
    //  入力文字列を拡張指標に変換（大文字小文字を無視して対応）
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ema" => Ok(ExtensionIndicator::Ema),
            "sma" => Ok(ExtensionIndicator::Sma),
            "bollinger" => Ok(ExtensionIndicator::Bollinger),
            "roc" => Ok(ExtensionIndicator::Roc),
            "adx" => Ok(ExtensionIndicator::Adx),
            "stochastics" => Ok(ExtensionIndicator::Stochastics),
            "fibonacci" => Ok(ExtensionIndicator::Fibonacci),
            "vwap" => Ok(ExtensionIndicator::Vwap),
            "ichimoku" => Ok(ExtensionIndicator::Ichimoku),

            _ => Err(format!("❌ 未知の指標: {}", s)),
        }
    }
}

impl ExtensionIndicator {
    /// 拡張指標ごとのカテゴリを返す
    fn category(&self) -> IndicatorCategory {
        match self {
            ind if TREND_INDICATORS.contains(ind) => IndicatorCategory::Trend,
            ind if OSCILLATOR_INDICATORS.contains(ind) => IndicatorCategory::Oscillator,
            ind if VOLATILITY_INDICATORS.contains(ind) => IndicatorCategory::Volatility,
            ind if SUPPORT_INDICATORS.contains(ind) => IndicatorCategory::SupportResist,
            _ => unreachable!("未分類のExtensionIndicatorが存在します"),
        }
    }
}

// カテゴリごとの定義（追加はここだけで済む）
const TREND_INDICATORS: &[ExtensionIndicator] = &[
    ExtensionIndicator::Ema,
    ExtensionIndicator::Sma,
    ExtensionIndicator::Adx,
    ExtensionIndicator::Roc,
    ExtensionIndicator::Ichimoku,
    ExtensionIndicator::Vwap,
];

const OSCILLATOR_INDICATORS: &[ExtensionIndicator] = &[ExtensionIndicator::Stochastics];
const VOLATILITY_INDICATORS: &[ExtensionIndicator] = &[ExtensionIndicator::Bollinger];
const SUPPORT_INDICATORS: &[ExtensionIndicator] = &[ExtensionIndicator::Fibonacci];

/// ユーザ視点の列挙型
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Stance {
    Buyer,
    Seller,
    Holder,
}

// 表示名（ゲージ見出し用）
impl std::fmt::Display for Stance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stance::Buyer => write!(f, "Buyer"),
            Stance::Holder => write!(f, "Holder"),
            Stance::Seller => write!(f, "Seller"),
        }
    }
}

/// 設定情報
#[derive(Debug, Clone)]
struct Config {
    buy_rsi: f64,
    sell_rsi: f64,
    macd_diff_low: f64,
    macd_diff_mid: f64,
    macd_minus_ok: bool,
    enabled_extensions: Vec<ExtensionIndicator>,
    bb_bandwidth_squeeze_pct: f64,

    stance: Stance,

    weight_basic: f64,
    weight_ema: f64,
    weight_sma: f64,
    weight_bollinger: f64,
    weight_roc: f64,
    weight_adx: f64,
    weight_stochastics: f64,
    weight_fibonacci: f64,
    weight_vwap: f64,
    weight_ichimoku: f64,
    brave_api_key: String,
    llm_provider: String,
    openai_model: String,
    openai_api_key: String,
    openai_extra_note: Option<String>,
    no_news: bool,
    custom_news_query: Option<String>,
    news_filter: bool,
    news_count: usize,
    news_freshness: String, // "pd"|"pw"|"pm"|"py"|"all"
    show_news: bool,
    save_technical_log: bool,
    log_format: String,
    log_dir: String,
    silent: bool,
    stdout_log: bool,
    max_note_length: usize,
    max_shortterm_length: usize,
    max_midterm_length: usize,
    max_news_length: usize,
    max_review_length: usize,
    ticker: String,
    alias_csv: Option<String>,
    no_llm: bool,
    debug_prompt: bool,
    data_append: bool,
    log_flat: bool,
    debug_args: bool,
}

/// 時系列データ構造
#[derive(Debug, Deserialize, Clone)]
struct MarketData {
    date: String,
    //    open: f64,
    high: f64,
    low: f64,
    close: f64,
    #[serde(default)]
    name: Option<String>, // MarketStackから取得できる場合に備える
}

/// ハードコードされた正式名称とクエリを保持する構造体
struct HardcodedInfo {
    formal_name: &'static str,
    //    query: &'static str,
}

/// テクニカル指標の分析結果を保持する構造体
struct AnalysisResult {
    indicator_name: String,   // 例: "基本テクニカル分析", "EMA", "SMA"
    description: Vec<String>, // 表示用の複数行テキスト（\n区切りでOK）
    score: f64,               // 元のスコア（-2〜+2の整数値、f64型）
}
// ==== 追加：最終スコアのスナップショット（唯一の真実） ====
struct FinalScoreSnapshot {
    total_score: f64,  // Σ(各スコア×重み)
    total_weight: f64, // 2×Σ(有効指標の重み)  ※各指標レンジ[-2,2]
    score_ratio: f64,  // total_score / total_weight（-1..+1）
}

/// 分析結果を保持する構造体（全出力・ログ・LLMプロンプト共通）
#[derive(Debug)]
struct TechnicalDataEntry {
    ticker: String,                 // ティッカー記号（例: AAPL, MSFT, 7203.T）
    name: String,                   // 企業名（例: NVIDIA Corp、ソフトバンク）
    date: String,                   // データ日付（例: 2025-05-09）
    close: f64,                     // 終値
    previous_close: f64,            // 前日終値
    price_diff: f64,                // 前日比（差額）
    price_diff_percent: f64,        // 前日比（%）
    macd: f64,                      // MACD値
    signal: f64,                    // MACDシグナル値
    prev_macd: f64,                 // 前MACD値（前日比計算用）
    prev_signal: f64,               // 前MACDシグナル値（前日比計算用）
    rsi: f64,                       // RSI値
    ema_short: f64,                 // EMA短期（例: 5日）
    ema_long: f64,                  // EMA長期（例: 20日）
    sma_short: f64,                 // SMA短期
    sma_long: f64,                  // SMA長期
    roc: Option<f64>,               // ROC（変化率）※拡張指標
    adx: Option<f64>,               // ADX（トレンド強度）※拡張指標
    stochastics_k: Option<f64>,     // ストキャスティクス %K
    stochastics_d: Option<f64>,     // ストキャスティクス %D
    bb_upper: f64,                  // ボリンジャーバンド上限
    bb_lower: f64,                  // ボリンジャーバンド下限
    bb_percent_b: f64,              // %Bインジケータ
    bb_bandwidth: f64,              // Bandwidth(%) を 0–100 の実数で保持
    fibo_38_2: Option<f64>,         // フィボナッチ 38.2%
    fibo_50_0: Option<f64>,         // フィボナッチ 50.0%
    fibo_61_8: Option<f64>,         // フィボナッチ 61.8%
    vwap: Option<f64>,              // Vwap
    tenkan_sen: Option<f64>,        // 一目均衡表転換線
    kijun_sen: Option<f64>,         // 一目均衡表基準線
    ema_score: Option<f64>,         // EMAによるスコア
    sma_score: Option<f64>,         // SMAによるスコア
    roc_score: Option<f64>,         // ROCによるスコア
    adx_score: Option<f64>,         // ADXによるスコア
    stochastics_score: Option<f64>, // ストキャスティクスによるスコア
    bollinger_score: Option<f64>,   // ボリンジャーバンドによるスコア
    fibonacci_score: Option<f64>,   // フィボナッチによるスコア
    vwap_score: Option<f64>,        // Vwapによるスコア
    ichimoku_score: Option<f64>,    // 一目均衡表スコア
    signal_score: f64,              // 基本シグナルスコア（MACD/RSIによる）
}

/// TechnicalDataEntry を安全に制御するラッパー構造体   
struct TechnicalDataGuard {
    entry: TechnicalDataEntry, // 内部保持（書き込みは許可された関数のみ）
}

impl TechnicalDataGuard {
    /// 新しいガード付き構造体を初期化
    fn new(ticker: String, date: String) -> Self {
        TechnicalDataGuard {
            entry: TechnicalDataEntry {
                ticker,
                name: String::new(),
                date,
                close: 0.0,
                previous_close: 0.0,
                price_diff: 0.0,
                price_diff_percent: 0.0,
                macd: 0.0,
                signal: 0.0,
                prev_macd: 0.0,
                prev_signal: 0.0,
                rsi: 0.0,
                ema_short: 0.0,
                ema_long: 0.0,
                sma_short: 0.0,
                sma_long: 0.0,
                bb_upper: 0.0,
                bb_lower: 0.0,
                bb_percent_b: 0.0, // ← 新規追加
                bb_bandwidth: 0.0, // ← 新規追加
                roc: None,
                adx: None,
                stochastics_k: None,
                stochastics_d: None,
                fibo_38_2: None,
                fibo_50_0: None,
                fibo_61_8: None,
                vwap: None,
                tenkan_sen: None,
                kijun_sen: None,
                ema_score: None,
                sma_score: None,
                adx_score: None,
                roc_score: None,
                stochastics_score: None,
                bollinger_score: None,
                fibonacci_score: None,
                vwap_score: None,
                ichimoku_score: None,
                signal_score: 0.0,
            },
        }
    }
    fn set_name(&mut self, value: &str) {
        self.entry.name = value.to_string();
    }
    fn set_close(&mut self, value: f64) {
        self.entry.close = value;
    }
    fn set_previous_close(&mut self, value: f64) {
        self.entry.previous_close = value;
    }
    fn set_price_diff(&mut self, value: f64) {
        self.entry.price_diff = value;
    }
    fn set_price_diff_percent(&mut self, value: f64) {
        self.entry.price_diff_percent = value;
    }
    fn set_rsi(&mut self, value: f64) {
        self.entry.rsi = value;
    }
    fn set_macd(&mut self, value: f64) {
        self.entry.macd = value;
    }
    fn set_signal(&mut self, value: f64) {
        self.entry.signal = value;
    }
    fn set_prev_macd(&mut self, value: f64) {
        self.entry.prev_macd = value;
    }
    fn set_prev_signal(&mut self, value: f64) {
        self.entry.prev_signal = value;
    }
    fn set_signal_score(&mut self, value: f64) {
        self.entry.signal_score = value;
    }
    fn set_ema_short(&mut self, value: f64) {
        self.entry.ema_short = value;
    }
    fn set_ema_long(&mut self, value: f64) {
        self.entry.ema_long = value;
    }
    fn set_ema_score(&mut self, value: f64) {
        self.entry.ema_score = Some(value);
    }
    fn set_sma_short(&mut self, value: f64) {
        self.entry.sma_short = value;
    }
    fn set_sma_long(&mut self, value: f64) {
        self.entry.sma_long = value;
    }
    fn set_sma_score(&mut self, value: f64) {
        self.entry.sma_score = Some(value);
    }
    fn set_adx(&mut self, value: f64) {
        self.entry.adx = Some(value);
    }
    fn set_adx_score(&mut self, value: f64) {
        self.entry.adx_score = Some(value);
    }
    fn set_roc(&mut self, value: f64) {
        self.entry.roc = Some(value);
    }
    fn set_roc_score(&mut self, value: f64) {
        self.entry.roc_score = Some(value);
    }
    fn set_stochastics_k(&mut self, value: f64) {
        self.entry.stochastics_k = Some(value);
    }
    fn set_stochastics_d(&mut self, value: f64) {
        self.entry.stochastics_d = Some(value);
    }
    fn set_stochastics_score(&mut self, value: f64) {
        self.entry.stochastics_score = Some(value);
    }
    fn set_bb_upper(&mut self, value: f64) {
        self.entry.bb_upper = value;
    }
    fn set_bb_lower(&mut self, value: f64) {
        self.entry.bb_lower = value;
    }
    fn set_bb_percent_b(&mut self, value: f64) {
        self.entry.bb_percent_b = value;
    }
    fn set_bb_bandwidth(&mut self, value: f64) {
        self.entry.bb_bandwidth = value;
    }
    fn set_bollinger_score(&mut self, value: f64) {
        self.entry.bollinger_score = Some(value);
    }
    fn set_fibo_38_2(&mut self, value: f64) {
        self.entry.fibo_38_2 = Some(value);
    }
    fn set_fibo_50_0(&mut self, value: f64) {
        self.entry.fibo_50_0 = Some(value);
    }
    fn set_fibo_61_8(&mut self, value: f64) {
        self.entry.fibo_61_8 = Some(value);
    }
    fn set_fibonacci_score(&mut self, value: f64) {
        self.entry.fibonacci_score = Some(value);
    }
    fn set_vwap(&mut self, value: f64) {
        self.entry.vwap = Some(value);
    }
    fn set_vwap_score(&mut self, value: f64) {
        self.entry.vwap_score = Some(value);
    }
    fn set_tenkan_sen(&mut self, value: f64) {
        self.entry.tenkan_sen = Some(value);
    }
    fn set_kijun_sen(&mut self, value: f64) {
        self.entry.kijun_sen = Some(value);
    }
    fn set_ichimoku_score(&mut self, value: f64) {
        self.entry.ichimoku_score = Some(value);
    }
    /// get関数
    fn get_name(&self) -> &str {
        &self.entry.name
    }
    fn get_ticker(&self) -> &str {
        &self.entry.ticker
    }
    fn get_date(&self) -> &str {
        &self.entry.date
    }
    fn get_close(&self) -> f64 {
        self.entry.close
    }
    fn get_previous_close(&self) -> f64 {
        self.entry.previous_close
    }
    fn get_price_diff(&self) -> f64 {
        self.entry.price_diff
    }
    fn get_price_diff_percent(&self) -> f64 {
        self.entry.price_diff_percent
    }
    fn get_rsi(&self) -> f64 {
        self.entry.rsi
    }
    fn get_macd(&self) -> f64 {
        self.entry.macd
    }
    fn get_signal(&self) -> f64 {
        self.entry.signal
    }
    fn get_prev_macd(&self) -> f64 {
        self.entry.prev_macd
    }
    fn get_prev_signal(&self) -> f64 {
        self.entry.prev_signal
    }
    fn get_signal_score(&self) -> f64 {
        self.entry.signal_score
    }
    fn get_ema_short(&self) -> f64 {
        self.entry.ema_short
    }
    fn get_ema_long(&self) -> f64 {
        self.entry.ema_long
    }
    fn get_ema_score(&self) -> Option<f64> {
        self.entry.ema_score
    }
    fn get_sma_short(&self) -> f64 {
        self.entry.sma_short
    }
    fn get_sma_long(&self) -> f64 {
        self.entry.sma_long
    }
    fn get_sma_score(&self) -> Option<f64> {
        self.entry.sma_score
    }
    fn get_adx(&self) -> Option<f64> {
        self.entry.adx
    }
    fn get_adx_score(&self) -> Option<f64> {
        self.entry.adx_score
    }
    fn get_roc(&self) -> Option<f64> {
        self.entry.roc
    }
    fn get_roc_score(&self) -> Option<f64> {
        self.entry.roc_score
    }
    fn get_stochastics_k(&self) -> Option<f64> {
        self.entry.stochastics_k
    }
    fn get_stochastics_d(&self) -> Option<f64> {
        self.entry.stochastics_d
    }
    fn get_stochastics_score(&self) -> Option<f64> {
        self.entry.stochastics_score
    }
    fn get_bb_upper(&self) -> f64 {
        self.entry.bb_upper
    }
    fn get_bb_lower(&self) -> f64 {
        self.entry.bb_lower
    }
    fn get_bb_percent_b(&self) -> f64 {
        self.entry.bb_percent_b
    }
    fn get_bb_bandwidth(&self) -> f64 {
        self.entry.bb_bandwidth
    }
    fn get_bollinger_score(&self) -> Option<f64> {
        self.entry.bollinger_score
    }
    fn get_fibo_38_2(&self) -> Option<f64> {
        self.entry.fibo_38_2
    }
    fn get_fibo_50_0(&self) -> Option<f64> {
        self.entry.fibo_50_0
    }
    fn get_fibo_61_8(&self) -> Option<f64> {
        self.entry.fibo_61_8
    }
    fn get_fibonacci_score(&self) -> Option<f64> {
        self.entry.fibonacci_score
    }
    ///フィボナッチリトレースメントライン（38.2%、50%、61.8%）と 終値（close）を比較する必要がある
    fn get_fib_close(&self) -> f64 {
        self.entry.close
    }
    fn get_vwap(&self) -> Option<f64> {
        self.entry.vwap
    }
    fn get_vwap_score(&self) -> Option<f64> {
        self.entry.vwap_score
    }
    fn get_tenkan_sen(&self) -> Option<f64> {
        self.entry.tenkan_sen
    }
    fn get_kijun_sen(&self) -> Option<f64> {
        self.entry.kijun_sen
    }
    fn get_ichimoku_score(&self) -> Option<f64> {
        self.entry.ichimoku_score
    }
}

///ファイル読み込み時に通す共通チェック関数
fn sanitize_ascii_file_lines(path: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let content = read_to_string(path)
        .map_err(|e| format!("❌ ファイル読み込み失敗: {} ({})", path.display(), e))?;
    let mut result = Vec::new();
    for (i, line) in content.lines().enumerate() {
        let line_number = i + 1;
        // ✅ 1行目に限って先頭BOMを除去
        let line = if line_number == 1 {
            if let Some(stripped) = line.strip_prefix('\u{FEFF}') {
                stripped
            } else {
                line
            }
        } else {
            line
        };
        if line.len() > 500 {
            return Err(format!(
                "❌ ファイル {} の {}行目: 行が長すぎます（{}文字）",
                path.display(),
                line_number,
                line.len()
            )
            .into());
        }
        if line.contains('\0') {
            return Err(format!(
                "❌ ファイル {} の {}行目: NULLバイトが含まれています",
                path.display(),
                line_number
            )
            .into());
        }
        if line
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\r')
        {
            return Err(format!(
                "❌ ファイル {} の {}行目: 制御文字を含んでいます",
                path.display(),
                line_number
            )
            .into());
        }
        if line.contains('\u{FEFF}') {
            return Err(format!(
                "❌ ファイル {} の {}行目: BOM（Byte Order Mark）が含まれています",
                path.display(),
                line_number
            )
            .into());
        }
        result.push(line.to_string());
    }
    Ok(result)
}

fn resolve_hardcoded_info(ticker: &str) -> Option<HardcodedInfo> {
    match ticker {
        "QQQ" => Some(HardcodedInfo {
            formal_name: "Invesco QQQ Trust (NASDAQ100)",
            // query: "QQQ OR \"NASDAQ100\"",
        }),
        "SPY" => Some(HardcodedInfo {
            formal_name: "SPDR S&P 500 ETF Trust (S&P500)",
            //query: "SPY OR \"S&P500\"",
        }),
        "ACWI" => Some(HardcodedInfo {
            formal_name: "iShares MSCI ACWI ETF (オールカントリー)",
            //query: "ACWI OR \"オールカントリー\"",
        }),
        "FANG+" => Some(HardcodedInfo {
            formal_name: "NYSE FANG+ Index",
            //query: "FANG+ OR \"FANGプラス指数\"",
        }),
        _ => None,
    }
}

fn initialize_environment_and_config() -> BuildCfgResult {
    // ✅ 環境変数の読み込み（tickwise.env ファイル）
    let env_path = Path::new("tickwise.env");
    if let Ok(lines) = sanitize_ascii_file_lines(env_path) {
        if let Ok(mut tmpfile) = NamedTempFile::new() {
            let content = lines.join("\n");
            if tmpfile.write_all(content.as_bytes()).is_ok() {
                if let Err(e) = from_path(tmpfile.path()) {
                    eprintln!(
                        "⚠️ tickwise.env の読み込みに失敗しました（無視されます）: {}",
                        e
                    );
                }
            }
        }
    }

    let mut args = Args::parse();

    // ✅ show-log-header モード専用ルート
    if args.show_log_header {
        let config = build_config(&args);
        generate_csv_header(&config);
        std::process::exit(0);
    }

    // ✅ ticker の必須チェック
    let raw_ticker = match args.ticker {
        Some(ref t) => t.clone(),
        None => {
            eprintln!("❌ --ticker は必須です");
            std::process::exit(1);
        }
    };

    // ✅ Ticker のサニタイズ
    args.ticker = Some(sanitize_ticker(&raw_ticker).unwrap_or_else(|err| {
        eprintln!("{err}");
        std::process::exit(1);
    }));
    // ✅ ティッカーの正規化（インデックス名をETFに変換）
    args.ticker = Some(normalize_ticker_input(args.ticker.as_deref().unwrap_or("")));

    // ✅ ティッカーのサニタイズ（日本tickerの末尾加工）
    //    - 日本株: #### → ####.T / ####.t → ####.T
    //    - 海外: 大文字化のみ（BRK.B 等はそのまま）
    args.ticker = Some(normalize_ticker(args.ticker.as_deref().unwrap_or("")));

    // ✅ カスタムニュースクエリとLLMノートのサニタイズ
    if let Some(q) = &args.custom_news_query {
        args.custom_news_query = Some(sanitize_news_query(q).unwrap_or_else(|err| {
            eprintln!("{err}");
            std::process::exit(1);
        }));
    }

    if let Some(n) = &args.openai_extra_note {
        args.openai_extra_note = Some(sanitize_llm_note(n).unwrap_or_else(|err| {
            eprintln!("{err}");
            std::process::exit(1);
        }));
    }

    let config = build_config(&args);
    if config.debug_args {
        eprintln!("Config= {:?}", config);
    }
    // ✅ 以降は config.ticker を唯一のソース（SoT）
    let ticker = config.ticker.clone();

    let ticker_name_map = match &config.alias_csv {
        Some(csv_path) => load_alias_csv(csv_path)?,
        None => HashMap::new(),
    };
    // 例: if let Some(code) = jp_code_from_ticker(&ticker) { ticker_name_map.insert(code, hardcoded.formal_name.to_string()); }
    // ハードコードされたティッカー名とクエリを追加
    Ok((config, ticker, ticker_name_map))
}

/// インデックスティッカーの変換
/// 例: "S&P500" → "SPY"
fn normalize_ticker_input(raw: &str) -> String {
    match raw.trim().to_uppercase().as_str() {
        // 米国インデックス
        "S&P500" | "SNP500" | "SP500" => "SPY".to_string(), // S&P500 ETF（最も一般的）
        "NASDAQ100" | "ナスダック100" => "QQQ".to_string(), // NASDAQ100 ETF
        "DOW" | "DJIA" | "ダウ平均" => "DIA".to_string(),   // ダウ平均 ETF

        // 日本インデックス
        "日経平均" | "NIKKEI225" => "1321.T".to_string(), // 日経平均連動ETF
        "TOPIX" => "1306.T".to_string(),                  // TOPIX連動ETF

        // 全世界・全米
        "全世界" | "オールカントリー" | "ACWI" => "ACWI".to_string(), // 全世界ETF
        "全米" | "VTI" | "トータルマーケット" => "VTI".to_string(),   // 全米市場ETF

        // FANG+インデックス
        "FANG+" | "FANGプラス" => "FNGU".to_string(),
        _ => raw.to_string(),
    }
}
/// ティッカーのサニタイズ
fn sanitize_ticker(t: &str) -> Result<String, &'static str> {
    let cleaned = t.trim().to_uppercase();
    // 英数字と.Tのみを許可
    if !cleaned
        .chars()
        .all(|c| c.is_alphanumeric() || c == '.' || c == 'T')
    {
        eprintln!("❌ 無効なティッカー形式: {t}");
        return Err("❌ ティッカーは英数字と.Tのみ使用可能です");
    }
    Ok(cleaned)
}
/// ニュースクエリのサニタイズ
fn sanitize_news_query(q: &str) -> Result<String, &'static str> {
    if q.len() > 200 {
        return Err("❌ ニュースクエリは200文字以内で入力してください");
    }
    if q.contains([';', '|', '`']) {
        return Err("❌ ニュースクエリに使用できない文字が含まれています");
    }
    Ok(q.trim().to_string())
}
/// LLMノートのサニタイズ
fn sanitize_llm_note(note: &str) -> Result<String, &'static str> {
    // 危険な記号のみ禁止
    if note.contains([';', '|', '`']) {
        return Err("❌ 使用できない記号（; | `）が含まれています");
    }
    let cleaned = note
        .trim()
        .replace('\n', " ")
        .replace(|c: char| c.is_whitespace(), " ");
    if cleaned.len() > 2000 {
        return Err("❌ 入力が長すぎます（2000文字以内で入力してください）");
    }
    Ok(cleaned)
}

/// stance setting
fn parse_stance(stance: &str) -> Stance {
    match stance {
        "buyer" => Stance::Buyer,
        "seller" => Stance::Seller,
        _ => Stance::Holder, // 省略は holder にフォールバック
    }
}

/// コンフィグの構築
fn build_config(args: &Args) -> Config {
    Config {
        debug_args: args.debug_args,

        // テクニカル閾値
        buy_rsi: if args.buy_rsi == 30.0 {
            env::var("BUY_RSI")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30.0)
        } else {
            args.buy_rsi
        },
        sell_rsi: if args.sell_rsi == 70.0 {
            env::var("SELL_RSI")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(70.0)
        } else {
            args.sell_rsi
        },
        macd_diff_low: if args.macd_diff_low == 2.0 {
            env::var("MACD_DIFF_LOW")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2.0)
        } else {
            args.macd_diff_low
        },
        macd_diff_mid: if args.macd_diff_mid == 10.0 {
            env::var("MACD_DIFF_MID")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10.0)
        } else {
            args.macd_diff_mid
        },

        // MACDマイナス圏フラグ
        macd_minus_ok: args.macd_minus_ok || get_bool_env("MACD_MINUS_OK"),

        // 投資スタンス設定
        stance: {
            let stance_source = if args.stance == "holder" {
                std::env::var("STANCE").unwrap_or_else(|_| "holder".to_string())
            } else {
                args.stance.clone()
            };
            parse_stance(&stance_source)
        },

        // Weight設定
        weight_basic: get_f64_from_args_or_env(args.weight_basic, "WEIGHT_BASIC", 1.0),
        weight_ema: get_f64_from_args_or_env(args.weight_ema, "WEIGHT_EMA", 1.0),
        weight_sma: get_f64_from_args_or_env(args.weight_sma, "WEIGHT_SMA", 1.0),
        weight_bollinger: get_f64_from_args_or_env(args.weight_bollinger, "WEIGHT_BOLLINGER", 1.0),
        weight_roc: get_f64_from_args_or_env(args.weight_roc, "WEIGHT_ROC", 1.0),
        weight_adx: get_f64_from_args_or_env(args.weight_adx, "WEIGHT_ADX", 1.0),
        weight_stochastics: get_f64_from_args_or_env(
            args.weight_stochastics,
            "WEIGHT_STOCHASTICS",
            1.0,
        ),
        weight_fibonacci: get_f64_from_args_or_env(args.weight_fibonacci, "WEIGHT_FIBONACCI", 1.0),
        weight_vwap: get_f64_from_args_or_env(args.weight_vwap, "WEIGHT_VWAP", 1.0),
        weight_ichimoku: get_f64_from_args_or_env(args.weight_ichimoku, "WEIGHT_ICHIMOKU", 1.0),
        // ✅ 拡張指標の選択（Vec<ExtensionIndicator> に変換）
        enabled_extensions: {
            let mut extensions = Vec::new();
            if args.ema || get_bool_env("EMA") {
                extensions.push(ExtensionIndicator::Ema);
            }
            if args.sma || get_bool_env("SMA") {
                extensions.push(ExtensionIndicator::Sma);
            }
            if args.roc || get_bool_env("ROC") {
                extensions.push(ExtensionIndicator::Roc);
            }
            if args.adx || get_bool_env("ADX") {
                extensions.push(ExtensionIndicator::Adx);
            }
            if args.stochastics || get_bool_env("STOCHASTICS") {
                extensions.push(ExtensionIndicator::Stochastics);
            }
            if args.bollinger || get_bool_env("BOLLINGER") {
                extensions.push(ExtensionIndicator::Bollinger);
            }
            if args.fibonacci || get_bool_env("FIBONACCI") {
                extensions.push(ExtensionIndicator::Fibonacci);
            }
            if args.vwap || get_bool_env("VWAP") {
                extensions.push(ExtensionIndicator::Vwap);
            }
            if args.ichimoku || get_bool_env("ICHIMOKU") {
                extensions.push(ExtensionIndicator::Ichimoku);
            }
            extensions
        },
        bb_bandwidth_squeeze_pct: sanitize_percent(
            args.bb_bandwidth_squeeze_pct, // clap が CLI or デフォルト(8.0) を保証
            0.0,
            100.0,
            "Bollinger bandwidth squeeze threshold (%)",
        ),
        // LLM設定
        llm_provider: if args.llm_provider == "openai" {
            std::env::var("LLM_PROVIDER").unwrap_or_else(|_| "openai".to_string())
        } else {
            args.llm_provider.clone()
        },

        // OpenAI設定
        openai_model: if args.openai_model == "gpt-4.1-nano" {
            env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4.1-nano".to_string())
        } else {
            args.openai_model.clone()
        },
        openai_api_key: {
            if let Some(k) = &args.openai_api_key {
                k.clone()
            } else {
                env::var("OPENAI_API_KEY").ok().unwrap_or_default()
            }
        },
        brave_api_key: {
            if let Some(k) = &args.brave_api_key {
                k.clone()
            } else {
                std::env::var("BRAVE_API_KEY").ok().unwrap_or_default()
            }
        },
        openai_extra_note: args
            .openai_extra_note
            .clone()
            .or_else(|| env::var("OPENAI_EXTRA_NOTE").ok()),
        // ニュース検索設定
        no_news: args.no_news || get_bool_env("NO_NEWS"),
        custom_news_query: args
            .custom_news_query
            .clone()
            .or_else(|| env::var("CUSTOM_NEWS_QUERY").ok())
            .filter(|s| !s.trim().is_empty()),

        news_filter: args.news_filter || get_bool_env("NEWS_FILTER"),
        show_news: args.show_news || get_bool_env("SHOW_NEWS"),

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
        },

        news_freshness: match args.news_freshness.clone() {
            Some(s) => s,
            None => env::var("NEWS_FRESHNESS").unwrap_or_else(|_| {
                if args.news_filter || get_bool_env("NEWS_FILTER") {
                    "pw".to_string()
                } else {
                    "pm".to_string()
                }
            }),
        },

        // ログ設定
        save_technical_log: args.save_technical_log || get_bool_env("SAVE_TECHNICAL_LOG"),
        log_format: if args.log_format != "csv" && args.log_format != "json" {
            env::var("LOG_FORMAT").unwrap_or_else(|_| "csv".to_string())
        } else {
            args.log_format.clone()
        },
        log_dir: if args.log_dir == "log" {
            env::var("LOG_DIR").unwrap_or_else(|_| "log".to_string())
        } else {
            args.log_dir.clone()
        },
        data_append: args.data_append || get_bool_env("CSV_APPEND"),
        log_flat: args.log_flat || get_bool_env("LOG_FLAT"),
        stdout_log: args.stdout_log,
        silent: args.silent,

        // プロンプト構成：数値設定
        max_note_length: get_usize_from_args_or_env(args.max_note_length, "MAX_NOTE_LENGTH", 300),
        max_shortterm_length: get_usize_from_args_or_env(
            args.max_shortterm_length,
            "MAX_SHORTTERM_LENGTH",
            150,
        ),
        max_midterm_length: get_usize_from_args_or_env(
            args.max_midterm_length,
            "MAX_MIDTERM_LENGTH",
            150,
        ),
        max_news_length: get_usize_from_args_or_env(args.max_news_length, "MAX_NEWS_LENGTH", 600),
        max_review_length: get_usize_from_args_or_env(
            args.max_review_length,
            "MAX_REVIEW_LENGTH",
            1000,
        ),

        // その他
        ticker: args.ticker.clone().unwrap_or_else(|| "SPY".to_string()),

        alias_csv: args
            .alias_csv
            .clone()
            .or_else(|| env::var("ALIAS_CSV").ok()),
        no_llm: args.no_llm || get_bool_env("NO_LLM"),
        debug_prompt: args.debug_prompt || get_bool_env("DEBUG_PROMPT"),
    }
}

///　環境変数からbool値（"true"/"false"）を取得し、bool型に変換する
fn get_bool_env(key: &str) -> bool {
    env::var(key)
        .map(|v| v.trim().to_lowercase() == "true")
        .unwrap_or(false)
}
/// CLI/env/defaultからf64値を取得し、0.5〜3.0範囲にサニタイズする
fn get_f64_from_args_or_env(arg_val: f64, env_key: &str, default: f64) -> f64 {
    // CLI引数がデフォルト値と異なる場合は優先
    let value = if (arg_val - default).abs() > f64::EPSILON {
        arg_val
    } else {
        match std::env::var(env_key) {
            Ok(env_val) => match env_val.parse::<f64>() {
                Ok(parsed) => parsed,
                Err(_) => {
                    eprintln!("⚠️ 環境変数 {} の値が無効（f64パース失敗）。デフォルト値({})を使用します。", env_key, default);
                    default
                }
            },
            Err(_) => default, // 環境変数未定義はデフォルト
        }
    };

    // ✅ サニタイズチェック（ロジック内注釈で明記）
    // - 型変換: .parse::<f64>() で文字列・桁落ち・超大数の誤入力をf64に変換（失敗時デフォルト）
    // - NaN/inf: is_finite()で無効化
    // - 負値: 0.0未満を無効化
    // - 許容範囲: 0.5～3.0以外を無効化
    if !value.is_finite() {
        eprintln!(
            "⚠️ 無効なweight値 (NaN/inf検出): {}。デフォルト値({})を使用します。",
            value, default
        );
        return default;
    }

    if value < 0.0 {
        eprintln!(
            "⚠️ 無効なweight値 (負の数): {}。デフォルト値({})を使用します。",
            value, default
        );
        return default;
    }

    if !(0.5..=3.0).contains(&value) {
        eprintln!(
            "⚠️ 無効なweight値 (範囲外): {}。デフォルト値({})を使用します。",
            value, default
        );
        return default;
    }

    value
}

/// 百分率用サニタイズ（0.0〜100.0 を許容）
fn sanitize_percent(value: f64, min: f64, max: f64, label: &str) -> f64 {
    if !value.is_finite() {
        eprintln!(
            "⚠️ Invalid {} (NaN/inf): {} -> clamped to {}",
            label, value, min
        );
        return min;
    }
    let clamped_value = value.clamp(min, max);
    if (clamped_value - value).abs() > f64::EPSILON {
        eprintln!(
            "⚠️ {} out of range: {} -> clamped to {}..={} (used {})",
            label, value, min, max, clamped_value
        );
    }
    clamped_value
}

/// コマンドライン引数がデフォルト値なら環境変数から取得し、usizeで返す
fn get_usize_from_args_or_env(arg_val: usize, env_key: &str, default: usize) -> usize {
    if arg_val != default {
        arg_val
    } else {
        get_usize_env(env_key, default)
    }
}
/// 環境変数からusize型の値を取得し、失敗時はデフォルト値を返す
fn get_usize_env(key: &str, default: usize) -> usize {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
/// ティッカーの正規化　"9432.T" / "9432" -> "9432.T", "NVDA" などはそのまま
fn normalize_ticker(raw: &str) -> String {
    let up = raw.trim().to_ascii_uppercase();
    if up.ends_with(".T") {
        up
    } else if up.ends_with(".t") {
        up.trim_end_matches(".t").to_string() + ".T"
    } else if up.len() == 4 && up.chars().all(|c| c.is_ascii_digit()) {
        format!("{up}.T")
    } else {
        up
    }
}

/// JP Code 四桁Tickerの加工　"9432.T" / "9432" -> Some("9432"), "NVDA" などは None
fn jp_code_from_ticker(t: &str) -> Option<String> {
    let up = t.trim().to_ascii_uppercase();
    if let Some(code) = up.strip_suffix(".T") {
        return (code.len() == 4 && code.chars().all(|c| c.is_ascii_digit()))
            .then(|| code.to_string());
    }
    (up.len() == 4 && up.chars().all(|c| c.is_ascii_digit())).then_some(up)
}
/// Yahoo Finance から市場データを取得する
/// Yahoo v8/chart: use only meta.chartPreviousClose, meta.currency, indicators.quote[0].(o/h/l/c), timestamp. Do NOT use previousClose/regularMarket*/adjclose.
async fn fetch_market_data(
    ticker: &str,
    //config: &Config,
) -> Result<Vec<MarketData>, Box<dyn std::error::Error>> {
    let ysym = if let Some(code) = jp_code_from_ticker(ticker) {
        format!("{}.T", code)
    } else {
        ticker.trim().to_string()
    };

    let url = format!(
        "https://query2.finance.yahoo.com/v8/finance/chart/{}?interval=1d&range=3mo",
        urlencoding::encode(&ysym)
    );

    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Tickwise)")
        .gzip(true)
        .brotli(true)
        .build()?;

    let text = client
        .get(&url)
        .header("accept", "application/json")
        .send()
        .await?
        .text()
        .await?;

    let json: Value = serde_json::from_str(&text)?;
    if json.get("chart").is_none() || !json["chart"]["error"].is_null() {
        return Err("❌ Yahoo /v8 chart からの取得に失敗しました。".into());
    }

    let result = json["chart"]["result"]
        .as_array()
        .ok_or("❌ chart.result 配列なし")?;
    if result.is_empty() {
        return Err("❌ chart.result が空です。".into());
    }

    let r0 = &result[0];
    let timestamps = r0["timestamp"]
        .as_array()
        .ok_or("❌ timestamp がありません。")?;
    let q0 = &r0["indicators"]["quote"][0];
    let highs = q0["high"].as_array().ok_or("❌ high がありません。")?;
    let lows = q0["low"].as_array().ok_or("❌ low がありません。")?;
    let closes = q0["close"].as_array().ok_or("❌ close がありません。")?;

    let n = timestamps
        .len()
        .min(highs.len())
        .min(lows.len())
        .min(closes.len());
    let mut out: Vec<MarketData> = Vec::with_capacity(n);

    for i in 0..n {
        let ts = match timestamps[i].as_i64() {
            Some(v) => v,
            None => continue,
        };
        let (h, l, c) = (highs[i].as_f64(), lows[i].as_f64(), closes[i].as_f64());
        if let (Some(h), Some(l), Some(c)) = (h, l, c) {
            let date = chrono::Utc
                .timestamp_opt(ts, 0)
                .single()
                .ok_or("❌ timestamp 変換失敗")?
                .date_naive()
                .to_string();
            out.push(MarketData {
                date,
                high: h,
                low: l,
                close: c,
                name: None,
            });
        }
    }

    if out.len() < 2 {
        return Err("❌ 時系列データが2件未満のため、テクニカル指標を構築できません。".into());
    }

    Ok(out)
}

/// エイリアスCSVの読み込み
fn load_alias_csv(path: &str) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let lines = sanitize_ascii_file_lines(Path::new(path))?;
    let content = lines.join("\n");
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(Cursor::new(content)); // ← 文字列からCSVとして読み込む

    let mut map = HashMap::new();
    for result in rdr.records() {
        let record = result?;
        let code = record
            .get(1)
            .ok_or("❌ CSVの2列目（証券コード）が存在しません")?
            .trim();
        let name = record
            .get(2)
            .ok_or("❌ CSVの3列目（企業名）が存在しません")?
            .trim();
        if !code.is_empty() && !name.is_empty() {
            //let ticker = format!("{}.T", code);
            map.insert(code.to_string(), name.to_string());
        }
    }

    Ok(map)
}

// テクニカル指標（RSI/MACD）を計算し、前日比やスコアを組み立てて Guard を返す
fn build_basic_technical_entry(
    config: &Config,
    data: &[MarketData],
    ticker_name_map: &HashMap<String, String>,
    //fetched_company_name: Option<String>,
) -> Result<TechnicalDataGuard, Box<dyn std::error::Error>> {
    // データが2件未満では前日比の評価ができないため中断
    if data.len() < 2 {
        return Err("❌ 時系列データが2件未満のため、テクニカル指標を構築できません。".into());
    }

    let hardcoded = resolve_hardcoded_info(&config.ticker); // ← SoT（正規化後の大文字）

    let latest = &data[data.len() - 1];
    let previous = &data[data.len() - 2];

    let alias_name_opt =
        jp_code_from_ticker(&config.ticker).and_then(|code| ticker_name_map.get(&code).cloned());

    // 優先順位：エイリアス >  LLM取得名 > ティッカー文字列
    let name = alias_name_opt // 4桁JP alias（あるなら最優先）
        .or(latest.name.clone()) // APIから来た会社名
        //.or(fetched_company_name) // 追加取得の会社名
        .or_else(|| hardcoded.map(|h| h.formal_name.to_string())) // ★ハードコード名
        .unwrap_or_else(|| config.ticker.clone());

    // 株価差と前日比（%）を算出
    let diff = latest.close - previous.close;
    let diff_percent = if previous.close != 0.0 {
        diff / previous.close * 100.0
    } else {
        0.0
    };

    // 終値をベクトル化（MACD, RSI計算用）
    let closes: Vec<f64> = data.iter().map(|d| d.close).collect();

    // RSI（14期間）で割高・割安を評価
    let mut rsi_calc =
        RelativeStrengthIndex::new(14).map_err(|e| format!("❌ RSI初期化失敗: {e}"))?;
    let rsi = closes
        .iter()
        .cloned()
        .map(|close_value| rsi_calc.next(close_value))
        .last()
        .ok_or("❌ RSIの計算に失敗しました")?;

    // ===== MACDは“1回だけ”計算して、前日値と当日値を同時取得（無駄排除）=====
    let mut macd_calc = MovingAverageConvergenceDivergence::new(12, 26, 9)
        .map_err(|e| format!("❌ MACD初期化失敗: {e}"))?;

    let mut prev_macd: f64 = 0.0;
    let mut prev_signal: f64 = 0.0;
    let mut macd: f64 = 0.0;
    let mut signal: f64 = 0.0;

    for (index, close_value) in closes.iter().cloned().enumerate() {
        let out = macd_calc.next(close_value);
        if index == closes.len() - 2 {
            prev_macd = out.macd;
            prev_signal = out.signal;
        }
        if index == closes.len() - 1 {
            macd = out.macd;
            signal = out.signal;
        }
    }

    // ここから判定（macd-minus-ok を“買い方向”に反映）
    let macd_diff = (macd - signal).abs();
    let macd_up = macd > signal && (macd > 0.0 || config.macd_minus_ok);
    let macd_down = macd < signal;
    let rsi_low = rsi <= config.buy_rsi;
    let rsi_high = rsi >= config.sell_rsi;

    // シグナルスコア（既存ロジックを踏襲）
    let mut signal_score: f64 = match (macd_up, macd_down, rsi_high, rsi_low) {
        // RSIが過熱でMACDが上昇 → 売り（異常なMACD乖離）
        (true, _, true, _) if macd_diff > 100.0 => -2.0,

        // RSIが過熱でMACDが上昇 → 売り
        (true, _, true, _) => -1.0,

        // RSIが過熱かつMACDが下降 → 売り
        (_, true, true, _) => -1.0,

        // RSIが割安かつMACDが上昇 → 買い（強いシグナル）
        (true, _, _, true) => 2.0,

        // RSIが割安でもMACDが下降 → 買い（やや反発期待）
        (_, true, _, true) => 1.0,

        // RSI中立かつMACD上昇 → MACD乖離小 → 様子見（+1）
        (true, _, false, false) if macd_diff < config.macd_diff_low => 1.0,

        // RSI中立かつMACD上昇 → MACD乖離中 → 買い（+2）
        (true, _, false, false) if macd_diff < config.macd_diff_mid => 2.0,

        // RSI中立かつMACD上昇 → MACD乖離大 → 買い（+2）
        (true, _, false, false) => 2.0,

        // RSI中立かつMACD下降 → MACD乖離小 → 様子見（0）
        (_, true, false, false) if macd_diff < config.macd_diff_low => 0.0,

        // RSI中立かつMACD下降 → 弱含み（-1）
        (_, true, false, false) => -1.0,

        // それ以外は中立
        _ => 0.0,
    };

    // ★ゲート：--macd-minus-ok が無効で MACD<0 のとき、買い方向のプラススコアを0にクランプ
    if !config.macd_minus_ok && macd < 0.0 && signal_score > 0.0 {
        signal_score = 0.0;
    }

    // セキュアアクセス構造（TechnicalDataGuard）で構造体へ代入
    let mut guard = TechnicalDataGuard::new(config.ticker.clone(), latest.date.clone());

    guard.set_name(&name);
    guard.set_close(latest.close);
    guard.set_previous_close(previous.close);
    guard.set_price_diff(diff);
    guard.set_price_diff_percent(diff_percent);
    guard.set_rsi(rsi);
    guard.set_macd(macd);
    guard.set_prev_macd(prev_macd);
    guard.set_prev_signal(prev_signal);
    guard.set_signal(signal);
    guard.set_signal_score(signal_score);

    Ok(guard)
}

fn evaluate_all_selected_extensions(
    config: &Config,
    data: &[MarketData],
    guard: &mut TechnicalDataGuard,
) -> Result<(), Box<dyn std::error::Error>> {
    for indicator in &config.enabled_extensions {
        match indicator {
            ExtensionIndicator::Ema => {
                evaluate_and_store_ema(data, guard)?;
            }
            ExtensionIndicator::Sma => {
                evaluate_and_store_sma(data, guard)?;
            }
            ExtensionIndicator::Bollinger => {
                evaluate_and_store_bollinger(data, guard)?;
            }
            ExtensionIndicator::Roc => {
                evaluate_and_store_roc(data, guard)?;
            }
            ExtensionIndicator::Adx => {
                evaluate_and_store_adx(data, guard)?;
            }
            ExtensionIndicator::Stochastics => {
                evaluate_and_store_stochastics(data, guard)?;
            }
            ExtensionIndicator::Fibonacci => {
                evaluate_and_store_fibonacci(data, guard)?;
            }
            ExtensionIndicator::Vwap => {
                evaluate_and_store_vwap(data, guard)?;
            }
            ExtensionIndicator::Ichimoku => {
                evaluate_and_store_ichimoku(data, guard)?;
            } //   _ => {}
        }
    }

    Ok(())
}

/// EMA（指数平滑移動平均）を計算し、ガード構造体にセキュアに格納する
fn evaluate_and_store_ema(
    data: &[MarketData],
    guard: &mut TechnicalDataGuard,
) -> Result<(), Box<dyn std::error::Error>> {
    use ta::indicators::ExponentialMovingAverage;

    if data.len() < 20 {
        return Err("❌ EMA計算には最低20日分のデータが必要です".into());
    }

    let closes: Vec<f64> = data.iter().map(|d| d.close).collect();

    let mut ema_short =
        ExponentialMovingAverage::new(5).map_err(|e| format!("❌ EMA短期 初期化失敗: {e}"))?;
    let mut ema_long =
        ExponentialMovingAverage::new(20).map_err(|e| format!("❌ EMA長期 初期化失敗: {e}"))?;

    let mut ema_short_val = 0.0;
    let mut ema_long_val = 0.0;

    for close in closes.iter().cloned() {
        ema_short_val = ema_short.next(close);
        ema_long_val = ema_long.next(close);
    }

    guard.set_ema_short(ema_short_val);
    guard.set_ema_long(ema_long_val);

    let diff = ema_short_val - ema_long_val;

    // 📏 5段階スコア：±2（強）±1（通常）0（同値圏 ±0.3）
    let ema_score: f64 = match diff {
        d if d > 2.0 => 2.0,        // 短期が大幅に上 → 強い上昇トレンド
        d if d > 0.5 => 1.0,        // やや上昇トレンド
        d if d.abs() <= 0.5 => 0.0, // 同値圏 → トレンドなし
        d if d < -2.0 => -2.0,      // 短期が大幅に下 → 強い下降トレンド
        _ => -1.0,                  // やや下降トレンド
    };
    guard.set_ema_score(ema_score);

    Ok(())
}

/// SMA（単純移動平均）を評価して構造体に格納（セキュアアクセス）
fn evaluate_and_store_sma(
    data: &[MarketData],
    guard: &mut TechnicalDataGuard,
) -> Result<(), Box<dyn std::error::Error>> {
    // 5日/20日SMA（※マジックナンバーは今後config化も可）
    use ta::indicators::SimpleMovingAverage;
    use ta::Next;

    let closes: Vec<f64> = data.iter().map(|d| d.close).collect();

    let mut sma_short = SimpleMovingAverage::new(5)?; // 短期SMA（5日）
    let mut sma_long = SimpleMovingAverage::new(20)?; // 長期SMA（20日）

    let short = closes
        .iter()
        .cloned()
        .map(|c| sma_short.next(c))
        .last()
        .unwrap_or(0.0);
    let long = closes
        .iter()
        .cloned()
        .map(|c| sma_long.next(c))
        .last()
        .unwrap_or(0.0);

    guard.set_sma_short(short);
    guard.set_sma_long(long);

    // スコアロジック
    let diff = short - long;

    let sma_score: f64 = match diff {
        d if d > 2.0 => 2.0,                   // 強いゴールデンクロス
        d if d > 0.5 => 1.0,                   // 緩やかな上昇
        d if (-0.5..=0.5).contains(&d) => 0.0, // 同値圏（絶対値0.5以下）
        d if d < -2.0 => -2.0,                 // 強いデッドクロス
        _ => -1.0,                             // 緩やかな下降
    };

    guard.set_sma_score(sma_score);
    Ok(())
}

/// ADXを評価して構造体に格納（セキュアアクセス　ADX　taクレートで用意されていない為演算）
fn evaluate_and_store_adx(
    data: &[MarketData],
    guard: &mut TechnicalDataGuard,
) -> Result<(), Box<dyn std::error::Error>> {
    //ADX計算に必要なデータ量（15）
    if data.len() < 15 {
        return Err("❌ ADXの計算には最低14期間以上のデータが必要です".into());
    }

    let mut trs = Vec::new();
    let mut plus_dm = Vec::new();
    let mut minus_dm = Vec::new();

    for i in 1..data.len() {
        let today = &data[i];
        let yesterday = &data[i - 1];

        let high_diff = today.high - yesterday.high;
        let low_diff = yesterday.low - today.low;

        let tr = (today.high - today.low)
            .max((today.high - yesterday.close).abs())
            .max((today.low - yesterday.close).abs());
        trs.push(tr);

        plus_dm.push(if high_diff > low_diff && high_diff > 0.0 {
            high_diff
        } else {
            0.0
        });
        minus_dm.push(if low_diff > high_diff && low_diff > 0.0 {
            low_diff
        } else {
            0.0
        });
    }

    let period = 14;

    let atr = trs.iter().rev().take(period).sum::<f64>() / period as f64;
    let p_dm = plus_dm.iter().rev().take(period).sum::<f64>();
    let m_dm = minus_dm.iter().rev().take(period).sum::<f64>();

    let p_di = 100.0 * (p_dm / atr);
    let m_di = 100.0 * (m_dm / atr);
    let dx = 100.0 * ((p_di - m_di).abs() / (p_di + m_di));

    // ADX = 直近のDX（単純版）
    let adx = dx;

    // スコア評価（20がトレンドの分岐点とされる）
    let adx_score: f64 = match adx {
        a if a >= 50.0 => 2.0,
        a if a >= 30.0 => 1.0,
        a if a >= 20.0 => 0.0,
        a if a >= 10.0 => -1.0,
        _ => -2.0,
    };

    // ✅ セキュアに構造体へ格納
    guard.set_adx(adx);
    guard.set_adx_score(adx_score);

    Ok(())
}
/// ROC（変化率）を計算して構造体に格納（スコア含む）
fn evaluate_and_store_roc(
    data: &[MarketData],
    guard: &mut TechnicalDataGuard,
) -> Result<(), Box<dyn std::error::Error>> {
    if data.len() < 11 {
        return Err("❌ ROC計算には11日分以上のデータが必要です".into());
    }

    let latest_close = data.last().unwrap().close;
    let previous_close = data[data.len() - 11].close;

    let roc = ((latest_close - previous_close) / previous_close) * 100.0;

    // スコア判定
    let roc_score: f64 = match roc {
        r if r > 10.0 => 2.0,   // 非常に強い上昇
        r if r > 3.0 => 1.0,    // 緩やかな上昇
        r if r >= -3.0 => 0.0,  // 同値圏（±3%以内）
        r if r < -10.0 => -2.0, // 非常に強い下落
        _ => -1.0,              // 緩やかな下落
    };

    guard.set_roc(roc);
    guard.set_roc_score(roc_score);

    Ok(())
}
fn evaluate_and_store_stochastics(
    data: &[MarketData],
    guard: &mut TechnicalDataGuard,
) -> Result<(), Box<dyn std::error::Error>> {
    if data.len() < 14 {
        return Err("❌ ストキャスティクス計算には14日以上のデータが必要です".into());
    }

    let period = 14; // ストキャスティクス %K の計算に使用する期間
    let mut highest_highs = Vec::new();
    let mut lowest_lows = Vec::new();
    let mut closes = Vec::new();

    for i in 0..data.len() {
        closes.push(data[i].close);
        let start = (i + 1).saturating_sub(period);
        let high = data[start..=i]
            .iter()
            .map(|d| d.high)
            .fold(f64::MIN, f64::max);
        let low = data[start..=i]
            .iter()
            .map(|d| d.low)
            .fold(f64::MAX, f64::min);
        highest_highs.push(high);
        lowest_lows.push(low);
    }

    let last = data.len() - 1;
    let high = highest_highs[last];
    let low = lowest_lows[last];
    let close = closes[last];

    let percent_k = if high != low {
        ((close - low) / (high - low)) * 100.0
    } else {
        0.0
    };

    let mut percent_ds = Vec::new();
    for i in (last + 1 - 3)..=last {
        let high = highest_highs[i];
        let low = lowest_lows[i];
        let close = closes[i];
        let k = if high != low {
            ((close - low) / (high - low)) * 100.0
        } else {
            0.0
        };
        percent_ds.push(k);
    }

    let percent_d = percent_ds.iter().copied().sum::<f64>() / percent_ds.len() as f64;

    // セキュアに構造体へ格納
    guard.set_stochastics_k(percent_k);
    guard.set_stochastics_d(percent_d);

    // スコア計算
    let stoch_score: f64 = match percent_k {
        k if k >= 90.0 => -2.0,
        k if k >= 80.0 => -1.0,
        k if k <= 10.0 => 2.0,
        k if k <= 20.0 => 1.0,
        _ => 0.0,
    };

    guard.set_stochastics_score(stoch_score);

    Ok(())
}

/// ボリンジャーバンドの計算と格納（%B と Bandwidth も計算）    
fn evaluate_and_store_bollinger(
    data: &[MarketData],
    guard: &mut TechnicalDataGuard,
) -> Result<(), Box<dyn std::error::Error>> {
    let closes: Vec<f64> = data.iter().map(|d| d.close).collect();

    // 一般的な設定
    let period: usize = 20;
    if closes.len() < period {
        return Err("❌ BollingerBands: データ数が20未満です。".into());
    }
    let stddev_multiplier: f64 = 2.0;

    // インジケータ（既存のものを流用）
    let mut bb = BollingerBands::new(period, stddev_multiplier)?;

    // 最新バンド値
    let mut upper: f64 = 0.0;
    let mut lower: f64 = 0.0;

    for &price in &closes {
        let bands = bb.next(price);
        upper = bands.upper;
        lower = bands.lower;
    }

    let current_price: f64 = *closes.last().unwrap_or(&0.0);

    // 保存（上限・下限）
    guard.set_bb_upper(upper);
    guard.set_bb_lower(lower);

    // 中心線（SMA）= (upper + lower)/2（2σ対称なのでこれでOK）
    let mid: f64 = (upper + lower) * 0.5;

    // %B = (Close - Lower) / (Upper - Lower)
    let denom = upper - lower;
    let percent_b: f64 = if denom != 0.0 {
        (current_price - lower) / denom
    } else {
        0.0
    };

    // Bandwidth(%) = (Upper - Lower) / Mid * 100
    let bandwidth_pct: f64 = if mid != 0.0 {
        (upper - lower) / mid * 100.0
    } else {
        0.0
    };

    // 保存（%B と Bandwidth）
    guard.set_bb_percent_b(percent_b);
    guard.set_bb_bandwidth(bandwidth_pct);

    // 既存の5段階スコアはそのまま踏襲
    let bollinger_score: f64 = match current_price {
        p if p > upper * 1.02 => -2.0, // 上限より2%以上上 → 強い売り
        p if p > upper => -1.0,        // 上限突破 → やや売り
        p if p < lower * 0.98 => 2.0,  // 下限より2%以上下 → 強い買い
        p if p < lower => 1.0,         // 下限割れ → やや買い
        _ => 0.0,                      // バンド内 → 中立
    };
    guard.set_bollinger_score(bollinger_score);

    Ok(())
}

/// フィボナッチを算出し、仕様に沿ってスコア化（SoT：ここで一度だけ決める）
fn evaluate_and_store_fibonacci(
    data: &[MarketData],
    guard: &mut TechnicalDataGuard,
) -> Result<(), Box<dyn std::error::Error>> {
    if data.len() < 2 {
        guard.set_fibonacci_score(0.0);
        return Err("❌ フィボナッチ計算には最低2件のデータが必要です".into());
    }

    let highs: Vec<f64> = data.iter().map(|d| d.high).collect();
    let lows: Vec<f64> = data.iter().map(|d| d.low).collect();
    let high = highs.iter().cloned().fold(f64::MIN, f64::max);
    let low = lows.iter().cloned().fold(f64::MAX, f64::min);
    let span = high - low;
    if span <= 0.0 {
        // 変動がないと閾値を作れない→中立
        guard.set_fibonacci_score(0.0);
        return Ok(());
    }

    let f38 = high - span * 0.382;
    let f50 = high - span * 0.500;
    let f62 = high - span * 0.618;

    guard.set_fibo_38_2(f38);
    guard.set_fibo_50_0(f50);
    guard.set_fibo_61_8(f62);

    // ── スコア判定（唯一の真実：ここで一度だけ） ──
    // ポジ/ネガの帯をガチッと固定。0 は「50% 近傍のみ」。
    let close = guard.get_fib_close();
    let eps = 0.50; // 50%近傍の許容幅

    let score = if (close - f50).abs() <= eps {
        0.0 // 50%±eps → 0
    } else if close > f38 {
        2.0 // 38.2%より上 → +2
    } else if close > f50 + eps && close < f38 {
        1.0 // 50%超〜38.2%未満 → +1
    } else if close < f62 {
        -2.0 // 61.8%より下 → -2
    } else if close < f50 - eps && close > f62 {
        -1.0 // 61.8%超〜50%未満 → -1
    } else {
        // ここに来るのは 50%±eps の外側ギリ/端バッファなど
        // 境界丸めのブレを避けるため 0 に落とさない
        if close > f50 {
            1.0
        } else {
            -1.0
        }
    };

    guard.set_fibonacci_score(score);
    Ok(())
}

/// VWAP（日足簡易版）を計算し、構造体にセキュアに格納
fn evaluate_and_store_vwap(
    data: &[MarketData],
    guard: &mut TechnicalDataGuard,
) -> Result<(), Box<dyn std::error::Error>> {
    use ta::indicators::SimpleMovingAverage;
    use ta::Next;

    // マジックナンバー回避のため注釈（一般的に14日で使用される）
    let period = 14;

    if data.len() < period {
        return Err(format!(
            "❌ VWAP計算に必要なデータが不足しています（{}日必要）",
            period
        )
        .into());
    }

    // Typical Price = (High + Low + Close) / 3
    let typical_prices: Vec<f64> = data
        .iter()
        .map(|d| (d.high + d.low + d.close) / 3.0)
        .collect();

    let mut sma = SimpleMovingAverage::new(period)?;
    let vwap = typical_prices
        .iter()
        .cloned()
        .map(|p| sma.next(p))
        .last()
        .unwrap_or(0.0);

    let close = guard.get_close(); // セキュアアクセスで終値取得
    let diff = close - vwap;
    let vwap_score: f64 = match diff {
        d if d >= 4.0 => 2.0,
        d if d >= 1.0 => 1.0,
        d if d <= -4.0 => -2.0,
        d if d <= -1.0 => -1.0,
        _ => 0.0,
    };

    guard.set_vwap(vwap); // VWAP値を格納
    guard.set_vwap_score(vwap_score); // スコアも格納

    Ok(())
}
/// 一目均衡表の転換線・基準線を計算し、構造体に格納する関数。
/// 現在の終値を基に転換線 > 基準線 ならスコア+1、逆は-1、同値は0とする。
fn evaluate_and_store_ichimoku(
    data: &[MarketData],
    guard: &mut TechnicalDataGuard,
) -> Result<(), Box<dyn std::error::Error>> {
    // 一目均衡表に必要な期間の長さ（転換線: 9期間、基準線: 26期間）
    if data.len() < 26 {
        return Err("❌ 一目均衡表の評価には最低26期間のデータが必要です".into());
    }

    // 転換線（過去9日間の高値と安値の平均）
    let recent_9 = &data[data.len() - 9..];
    let high_9 = recent_9.iter().map(|d| d.high).fold(f64::MIN, f64::max);
    let low_9 = recent_9.iter().map(|d| d.low).fold(f64::MAX, f64::min);
    let tenkan = (high_9 + low_9) / 2.0;

    // 基準線（過去26日間の高値と安値の平均）
    let recent_26 = &data[data.len() - 26..];
    let high_26 = recent_26.iter().map(|d| d.high).fold(f64::MIN, f64::max);
    let low_26 = recent_26.iter().map(|d| d.low).fold(f64::MAX, f64::min);
    let kijun = (high_26 + low_26) / 2.0;

    // 構造体へ格納（セキュアアクセス経由）
    guard.set_tenkan_sen(tenkan);
    guard.set_kijun_sen(kijun);

    let diff = tenkan - kijun;

    // 📏 5段階スコア：±2（強）±1（通常）0（同値圏 ±0.5）
    let ichimoku_score: f64 = match diff {
        d if d > 2.0 => 2.0,        // 強いゴールデンクロス
        d if d > 0.5 => 1.0,        // ややゴールデンクロス
        d if d.abs() <= 0.5 => 0.0, // 同値圏
        d if d < -2.0 => -2.0,      // 強いデッドクロス
        _ => -1.0,                  // ややデッドクロス
    };

    guard.set_ichimoku_score(ichimoku_score);

    Ok(())
}

///出力先セレクター
fn select_output_target(
    config: &Config,
    guard: &TechnicalDataGuard,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut results: Vec<AnalysisResult> = Vec::new();

    for indicator in &config.enabled_extensions {
        match indicator {
            ExtensionIndicator::Ema => {
                let ema_score_value = guard.get_ema_score().unwrap_or(0.0);
                results.push(AnalysisResult {
                    indicator_name: "EMA".to_string(),
                    description: Vec::new(),
                    score: ema_score_value,
                });
            }
            ExtensionIndicator::Sma => {
                let sma_score_value = guard.get_sma_score().unwrap_or(0.0);
                results.push(AnalysisResult {
                    indicator_name: "SMA".to_string(),
                    description: Vec::new(),
                    score: sma_score_value,
                });
            }
            ExtensionIndicator::Roc => {
                let roc_score_value = guard.get_roc_score().unwrap_or(0.0);
                results.push(AnalysisResult {
                    indicator_name: "ROC".to_string(),
                    description: Vec::new(),
                    score: roc_score_value,
                });
            }
            ExtensionIndicator::Adx => {
                let adx_score_value = guard.get_adx_score().unwrap_or(0.0);
                results.push(AnalysisResult {
                    indicator_name: "ADX".to_string(),
                    description: Vec::new(),
                    score: adx_score_value,
                });
            }
            ExtensionIndicator::Stochastics => {
                let stoch_score_value = guard.get_stochastics_score().unwrap_or(0.0);
                results.push(AnalysisResult {
                    indicator_name: "Stochastics".to_string(),
                    description: Vec::new(),
                    score: stoch_score_value,
                });
            }
            ExtensionIndicator::Bollinger => {
                let bollinger_score_value = guard.get_bollinger_score().unwrap_or(0.0);
                results.push(AnalysisResult {
                    indicator_name: "Bollinger".to_string(),
                    description: Vec::new(),
                    score: bollinger_score_value,
                });
            }
            ExtensionIndicator::Fibonacci => {
                let fibonacci_score_value = guard.get_fibonacci_score().unwrap_or(0.0);
                results.push(AnalysisResult {
                    indicator_name: "Fibonacci".to_string(),
                    description: Vec::new(),
                    score: fibonacci_score_value,
                });
            }
            ExtensionIndicator::Vwap => {
                let vwap_score_value = guard.get_vwap_score().unwrap_or(0.0);
                results.push(AnalysisResult {
                    indicator_name: "VWAP".to_string(),
                    description: Vec::new(),
                    score: vwap_score_value,
                });
            }
            ExtensionIndicator::Ichimoku => {
                let ichimoku_score_value = guard.get_ichimoku_score().unwrap_or(0.0);
                results.push(AnalysisResult {
                    indicator_name: "Ichimoku".to_string(),
                    description: Vec::new(),
                    score: ichimoku_score_value,
                });
            }
        }
    }

    if config.save_technical_log {
        save_technical_log(config, guard, &results)?;
    }

    if !config.silent {
        technical_render_to_terminal(config, guard);
    }

    Ok(())
}

/// テクニカル分析結果を画面に出力（セキュアアクセスにより構造体を直接参照せず、TechnicalDataGuard経由で取得）
fn technical_render_to_terminal(config: &Config, guard: &TechnicalDataGuard) {
    // ① メイン情報
    display_main_info(config, guard);

    // ② 基本テクニカル分析
    let basic_result = render_basic(config, guard);
    display_analysis_result(&basic_result);

    // ③ 拡張テクニカル分析
    let extension_results = render_extension(config, guard);
    for extension_result in extension_results {
        match extension_result {
            ExtensionResult::Header(ref s) => println!("{}", s),
            ExtensionResult::Analysis(ref result) => display_analysis_result(result),
        }
    }

    // ④ 総合スコア表示
    render_final_score(config, guard);
}

/// 拡張テクニカル指標のレンダリング処理（仕様厳守）
enum ExtensionResult {
    Header(String),
    Analysis(AnalysisResult),
}
/// 拡張テクニカル指標のレンダリング処理（仕様厳守）
fn render_extension(config: &Config, guard: &TechnicalDataGuard) -> Vec<ExtensionResult> {
    let mut extension_results = Vec::new();

    let mut categorized_map: HashMap<IndicatorCategory, Vec<ExtensionIndicator>> = HashMap::new();
    for indicator in &config.enabled_extensions {
        let category = indicator.category();
        categorized_map
            .entry(category)
            .or_default()
            .push(indicator.clone());
    }

    for category in [
        IndicatorCategory::Trend,
        IndicatorCategory::Oscillator,
        IndicatorCategory::Volatility,
        IndicatorCategory::SupportResist,
    ] {
        if let Some(indicators) = categorized_map.get(&category) {
            match category {
                IndicatorCategory::Trend => extension_results
                    .push(ExtensionResult::Header("---トレンド系指標---".to_string())),
                IndicatorCategory::Oscillator => extension_results.push(ExtensionResult::Header(
                    "---オシレーター系指標---".to_string(),
                )),
                IndicatorCategory::Volatility => extension_results.push(ExtensionResult::Header(
                    "---ボラティリティ系指標---".to_string(),
                )),
                IndicatorCategory::SupportResist => extension_results.push(
                    ExtensionResult::Header("---補助トレンド系指標---".to_string()),
                ),
            }

            for indicator in indicators {
                let rendered = match indicator {
                    ExtensionIndicator::Ema => render_ema(config, guard),
                    ExtensionIndicator::Sma => render_sma(config, guard),
                    ExtensionIndicator::Bollinger => render_bollinger(config, guard),
                    ExtensionIndicator::Roc => render_roc(config, guard),
                    ExtensionIndicator::Adx => render_adx(config, guard),
                    ExtensionIndicator::Stochastics => render_stochastics(config, guard),
                    ExtensionIndicator::Fibonacci => render_fibonacci(config, guard),
                    ExtensionIndicator::Vwap => render_vwap(config, guard),
                    ExtensionIndicator::Ichimoku => render_ichimoku(config, guard),
                };
                extension_results.push(ExtensionResult::Analysis(rendered));
            }
        }
    }
    extension_results
}

// 最終スコアのスナップショットを一度だけ算出（SoT）
fn calculate_final_score_snapshot(
    config: &Config,
    guard: &TechnicalDataGuard,
) -> FinalScoreSnapshot {
    // 分子：既存ロジック
    let total_score = calculate_final_score(config, guard);

    // 分母：有効な指標の重み合計×2.0（欠損スコアでもenabledなら含める）
    let mut sum_weights = config.weight_basic;
    for ext in &config.enabled_extensions {
        sum_weights += match ext {
            ExtensionIndicator::Ema => config.weight_ema,
            ExtensionIndicator::Sma => config.weight_sma,
            ExtensionIndicator::Roc => config.weight_roc,
            ExtensionIndicator::Adx => config.weight_adx,
            ExtensionIndicator::Stochastics => config.weight_stochastics,
            ExtensionIndicator::Bollinger => config.weight_bollinger,
            ExtensionIndicator::Fibonacci => config.weight_fibonacci,
            ExtensionIndicator::Vwap => config.weight_vwap,
            ExtensionIndicator::Ichimoku => config.weight_ichimoku,
        };
    }
    let total_weight = 2.0 * sum_weights;
    let score_ratio = if total_weight != 0.0 {
        total_score / total_weight
    } else {
        0.0
    };

    FinalScoreSnapshot {
        total_score,
        total_weight,
        score_ratio,
    }
}

/// 総合スコアを表示（計算結果とグラフを含む）
fn render_final_score(config: &Config, guard: &TechnicalDataGuard) {
    let snap = calculate_final_score_snapshot(config, guard);
    let lines =
        compose_final_score_lines_stance(&snap, &config.stance, /*include_gauge=*/ true);
    for line in lines {
        println!("{}", line);
    }
}

/// 最終スコア（基本＋拡張指標の加重合計）を計算
fn calculate_final_score(config: &Config, guard: &TechnicalDataGuard) -> f64 {
    let mut total_score = guard.get_signal_score() * config.weight_basic;

    for ext in &config.enabled_extensions {
        let weighted_score = match ext {
            ExtensionIndicator::Ema => guard.get_ema_score().unwrap_or(0.0) * config.weight_ema,
            ExtensionIndicator::Sma => guard.get_sma_score().unwrap_or(0.0) * config.weight_sma,
            ExtensionIndicator::Roc => guard.get_roc_score().unwrap_or(0.0) * config.weight_roc,
            ExtensionIndicator::Adx => guard.get_adx_score().unwrap_or(0.0) * config.weight_adx,
            ExtensionIndicator::Stochastics => {
                guard.get_stochastics_score().unwrap_or(0.0) * config.weight_stochastics
            }
            ExtensionIndicator::Bollinger => {
                guard.get_bollinger_score().unwrap_or(0.0) * config.weight_bollinger
            }
            ExtensionIndicator::Fibonacci => {
                guard.get_fibonacci_score().unwrap_or(0.0) * config.weight_fibonacci
            }
            ExtensionIndicator::Vwap => guard.get_vwap_score().unwrap_or(0.0) * config.weight_vwap,
            ExtensionIndicator::Ichimoku => {
                guard.get_ichimoku_score().unwrap_or(0.0) * config.weight_ichimoku
            }
        };
        total_score += weighted_score;
    }

    total_score
}

/// 銘柄名・日付・価格情報を表示（セキュアアクセスによりTechnicalDataEntryを直接参照せず、TechnicalDataGuard経由で取得）
fn display_main_info(config: &Config, guard: &TechnicalDataGuard) {
    // ← ここでの固定見出し（MACDマイナス許容）は削除

    if (config.buy_rsi - 30.0).abs() > f64::EPSILON {
        println!(
            "{}",
            format!(
                "🔧 --buy-rsi={:.2} を指定 → RSIが{:.2}以下で買い圏とみなされます",
                config.buy_rsi, config.buy_rsi
            )
            .red()
        );
    }
    if (config.sell_rsi - 70.0).abs() > f64::EPSILON {
        println!(
            "{}",
            format!(
                "🔧 --sell-rsi={:.2} を指定 → RSIが{:.2}以上で売り圏とみなされます",
                config.sell_rsi, config.sell_rsi
            )
            .red()
        );
    }
    if (config.macd_diff_low - 2.0).abs() > f64::EPSILON {
        println!(
            "{}",
            format!(
                "🔧 --macd-diff-low={:.2} を指定 → MACD差が{:.2}未満ならスコアを中立に補正します",
                config.macd_diff_low, config.macd_diff_low
            )
            .red()
        );
    }
    if (config.macd_diff_mid - 10.0).abs() > f64::EPSILON {
        println!(
            "{}",
            format!(
                "🔧 --macd-diff-mid={:.2} を指定 → MACD差が{:.2}以上でスコアを強化します",
                config.macd_diff_mid, config.macd_diff_mid
            )
            .red()
        );
    }
    //  ── JST 現在日時を取得,基本情報表示 ──
    let now = Local::now();
    let date_jst = now.format("%Y-%m-%d").to_string();
    let time_jst = now.format("%H:%M").to_string();

    println!("\n📊 銘柄: {}（{}）", guard.get_name(), guard.get_ticker());
    println!("📅 日時: {} {} JST", date_jst, time_jst);
    println!("💰 現在値　: {:.2}", guard.get_close());
    println!("💰 前日終値: {:.2}", guard.get_previous_close());

    let diff = guard.get_price_diff();
    let percent = guard.get_price_diff_percent();
    let diff_str = if diff > 0.0 {
        format!("{:+.2} ({:+.2}%)", diff, percent).green()
    } else if diff < 0.0 {
        format!("{:+.2} ({:+.2}%)", diff, percent).red()
    } else {
        format!("{:+.2} ({:+.2}%)", diff, percent).normal()
    };
    println!("📊 前日比: {}", diff_str);

    // ← ここで動的ラベルを差し込む
    let macd_minus_label = if config.macd_minus_ok {
        if guard.get_macd() < 0.0 && guard.get_macd() > guard.get_signal() {
            "※『MACDマイナス許容』設定: 有効（今回“適用対象”）"
        } else {
            "※『MACDマイナス許容』設定: 有効（今回“未適用”）"
        }
    } else {
        "※『MACDマイナス許容』設定: 無効"
    };
    println!("{}", macd_minus_label);
    println!(); // 空行
}

/// 基本テクニカル分析（MACD + RSI）を行い、AnalysisResultに格納する
fn render_basic(config: &Config, guard: &TechnicalDataGuard) -> AnalysisResult {
    let rsi = guard.get_rsi();
    let macd = guard.get_macd();
    let signal = guard.get_signal();
    let prev_macd = guard.get_prev_macd();
    let prev_signal = guard.get_prev_signal();
    let weight = config.weight_basic;

    let score = guard.get_signal_score();
    let adjusted_score = score * weight;

    let mut description_lines: Vec<String> = Vec::new();

    // ★ ここから追加：MACDとRSIの数値を明示的に表示
    description_lines.push("基本テクニカル分析（MACDとRSIによる評価）".to_string());
    description_lines.push(format!("📈 MACD: {:.4} / Signal: {:.4}", macd, signal));
    description_lines.push(format!("📊 RSI: {:.2}", rsi));
    // ★ ここまで追加

    // ① トレンド継続 or 転換
    if prev_macd < prev_signal && macd > signal {
        description_lines.push("⚠️ MACDがゴールデンクロス → 上昇トレンド転換の可能性".to_string());
    } else if macd > signal {
        description_lines.push(
            "⚠️ MACDがSignalを上回る状態が継続 → 上昇トレンドが維持されている可能性".to_string(),
        );
    } else if prev_macd > prev_signal && macd < signal {
        description_lines.push("⚠️ MACDがデッドクロス → 下落トレンド転換の可能性".to_string());
    } else if macd < signal {
        description_lines
            .push("⚠️ MACDがSignalを下回る状態が継続 → 弱含みトレンドが継続中".to_string());
    }

    // ② 乖離の大きさ
    let macd_diff = macd - signal;
    if macd_diff >= 5.0 {
        description_lines.push(format!(
            "⚠️ MACDがSignalより大幅に上回っています（+{:.2}）→ 過熱感がある可能性があります",
            macd_diff
        ));
    } else if macd_diff <= -5.0 {
        description_lines.push(format!(
            "⚠️ MACDがSignalより大幅に下回っています（{:.2}）→ 割安感がある可能性があります",
            macd_diff
        ));
    }

    // ③ RSIの極端な値
    if rsi <= 5.0 {
        description_lines.push("⚠️ RSIが 0% に近い極端な売られすぎ → 反発に警戒".to_string());
    } else if rsi >= 95.0 {
        description_lines.push("⚠️ RSIが 100% に近い極端な買われすぎ → 反転下落に注意".to_string());
    }

    // ④ スコア評価の要約
    match score {
        2.0 => {
            if rsi < 30.0 {
                description_lines.push(
                    "🟢 [基本スコア:+2] RSIが極端に割安 → 強い買いシグナル → スコア+2加点"
                        .to_string(),
                );
            } else {
                description_lines
                    .push("🟢 [基本スコア:+2] MACDが強い上昇トレンド → スコア+2加点".to_string());
            }
        }
        1.0 => {
            if rsi < 40.0 {
                description_lines.push(
                    "🟢 [基本スコア:+1] RSIが割安圏 → 買いシグナル → スコア+1加点".to_string(),
                );
            } else {
                description_lines
                    .push("🟢 [基本スコア:+1] MACDが上昇傾向 → スコア+1加点".to_string());
            }
        }
        0.0 => {
            description_lines
                .push("⚪️ [基本スコア:0] RSI・MACDともに中立 → スコアなし".to_string());
        }
        -1.0 => {
            if rsi > 60.0 {
                description_lines.push(
                    "🔴 [基本スコア:-1] RSIが割高圏 → 売りシグナル → スコア-1減点".to_string(),
                );
            } else {
                description_lines
                    .push("🔴 [基本スコア:-1] MACDが下降傾向 → スコア-1減点".to_string());
            }
        }
        -2.0 => {
            if rsi > 70.0 {
                description_lines.push(
                    "🔴 [基本スコア:-2] RSIが極端に割高 → 強い売りシグナル → スコア-2減点"
                        .to_string(),
                );
            } else {
                description_lines
                    .push("🔴 [基本スコア:-2] MACDが強い下降トレンド → スコア-2減点".to_string());
            }
        }
        _ => {
            description_lines.push("💡 [基本スコア:?] 不明または不正なスコア条件".to_string());
        }
    }

    // ⑤ スコア調整情報
    description_lines.push(format!(
        "📝 スコア調整値({:.1}) = スコア({}) × Weight({:.1})",
        adjusted_score, score, weight
    ));

    // 結果構造体にまとめる
    AnalysisResult {
        indicator_name: "基本テクニカル分析".to_string(),
        description: description_lines,
        score,
    }
}
/// AnalysisResultを受け取り、内容を画面に出力する関数
fn display_analysis_result(result: &AnalysisResult) {
    for line in &result.description {
        println!("{}", line);
    }
    println!(); // 改行
}

/// スコア分類
fn classify_score(score_ratio: f64) -> &'static str {
    match score_ratio {
        r if r >= 0.8 => "🟢 強い買い",
        r if r >= 0.6 => "🟢 買い優勢",
        r if r >= 0.4 => "🟢 買い傾向あり",
        r if r >= 0.2 => "🟡 やや買い寄り",
        r if r >= 0.0 => "🟡 買い気配",
        r if r >= -0.2 => "⚪️ 様子見（中立）",
        r if r >= -0.4 => "🟠 売り気配",
        r if r >= -0.6 => "🟠 やや売り寄り",
        r if r >= -0.8 => "🔴 売り傾向あり",
        _ => "🔴 強い売り",
    }
}

/// 総合スコア行（スタンス別）。ゲージ有無は引数で切替
/// 端末/LLM共通：総合スコアの表示行を生成（スタンス別、ゲージ有無切替）
/// ★ 0除算ガードはこの関数内だけで完結
fn compose_final_score_lines_stance(
    snap: &FinalScoreSnapshot,
    stance: &Stance,
    include_gauge: bool,
) -> Vec<String> {
    let mut lines = Vec::new();

    // 1) 総合スコア見出し（例: -12.0/-22.0-22.0）
    lines.push(format!(
        "🧮 総合スコア: {s:.1} ({w:.1}〜-{w:.1})の範囲",
        s = snap.total_score,
        w = if snap.total_weight.is_finite() {
            snap.total_weight.abs()
        } else {
            0.0
        }
    ));
    lines.push(format!(
        "トータルスコア（スタンス：{}）",
        stance_caption(stance)
    ));

    // ★ ここでのみ 0除算を防ぐ（UI層に限定）
    let weight_abs = if snap.total_weight.is_finite() {
        snap.total_weight.abs()
    } else {
        0.0
    };
    let has_weight = weight_abs > f64::EPSILON;

    match stance {
        // Buyer / Seller：単極ゲージ（右→左に塗る）。％は 0..=100、丸め＝四捨五入で統一。
        Stance::Buyer | Stance::Seller => {
            // 基準は Buyer％。Seller％は 100 - Buyer％ で相補（表示・判定とも統一）
            let buyer_percent: u8 = if !has_weight {
                50
            } else {
                let p = ((weight_abs + snap.total_score).clamp(0.0, 2.0 * weight_abs)
                    / (2.0 * weight_abs))
                    * 100.0;
                p.round().clamp(0.0, 100.0) as u8
            };
            let seller_percent: u8 = 100u8.saturating_sub(buyer_percent);

            // 5段階のラベル＋色（🟢🟡⚪️🟠🔴）をスタンス別に割当
            let (percent, mark, action_text) = match *stance {
                Stance::Buyer => {
                    let p = buyer_percent;
                    let (m, t) = if p >= 90 {
                        ("🟢", "積極的に買う")
                    } else if p >= 61 {
                        ("🟡", "買う")
                    }
                    // 61–89 を「買う」
                    else if (40..=60).contains(&p) {
                        ("⚪️", "中立")
                    }
                    // 40–60 を厳密に中立
                    else if p >= 20 {
                        ("🟠", "買いを推奨しない")
                    } else {
                        ("🔴", "買わない")
                    };
                    (p, m, t)
                }
                Stance::Seller => {
                    let p = seller_percent;
                    let (m, t) = if p >= 90 {
                        ("🟢", "積極的に売る")
                    } else if p >= 61 {
                        ("🟡", "売る")
                    }
                    // 61–89 を「売る」
                    else if (40..=60).contains(&p) {
                        ("⚪️", "中立")
                    }
                    // 40–60 を厳密に中立
                    else if p >= 20 {
                        ("🟠", "売りを推奨しない")
                    } else {
                        ("🔴", "売らない")
                    };
                    (p, m, t)
                }
                _ => unreachable!("Holder 分岐は外側で処理済み"),
            };
            // 判定行（％はスタンスごとの percent を表示）
            lines.push(format!(
                "→ 判定: {mark} {action} {pct}%",
                action = action_text,
                pct = percent
            ));

            if include_gauge {
                // グラフの横に「100％/0％」を付与（仕様固定文言）
                let gauge = match stance {
                    Stance::Buyer => {
                        render_unipolar_gauge_rtl(percent, "買い 100％", "0％ 買わない", 25)
                    }
                    Stance::Seller => {
                        render_unipolar_gauge_rtl(percent, "売り 100％", "0％ 売らない", 25)
                    }
                    _ => String::new(),
                };

                if !gauge.is_empty() {
                    // 判定の“色ボール”とゲージ色を一致させる（'█' のみ着色）
                    // ※ 既存の colored::* を使用（新規関数なし／副作用なし）
                    let colored_g = match mark {
                        "🟢" => gauge.replace("█", &"█".green().to_string()),
                        "🟡" => gauge.replace("█", &"█".yellow().to_string()),
                        "⚪️" => gauge.replace("█", &"█".white().to_string()),
                        "🟠" => gauge.replace("█", &"█".truecolor(255, 165, 0).to_string()),
                        "🔴" => gauge.replace("█", &"█".red().to_string()),
                        _ => gauge,
                    };
                    lines.push(colored_g);
                }
            }
        }

        // Holder：両極ゲージ。符号付き％は score_ratio*100（重みゼロ時は0固定）
        // Holder：判定の色（🟢🟡⚪️🟠🔴）とバー色を完全一致させる。
        // 0除算時は score_ratio=0% 固定（中央'|'のみ、無塗り）。
        Stance::Holder => {
            let holder_pct: i32 = if has_weight {
                (snap.score_ratio * 100.0).round() as i32
            } else {
                0
            };
            // 判定テキスト（色付き絵文字＋語彙）は仕様の分類関数を使用
            let action_text = classify_score(snap.score_ratio);
            // 表記は「→ 判定: <action>  スコア比率 ±X%」のまま（スペース2つを保持）
            lines.push(format!(
                "→ 判定: {action} スコア比率 {:+}%",
                holder_pct,
                action = action_text
            ));

            if include_gauge {
                let g = render_bipolar_gauge_lr(snap.score_ratio, 51);
                lines.push(g);
            }
        }
    }

    lines.push(String::new());
    lines
}

/// EMA の表示（文言ロジックだけ修正：同値圏なら DC/GC を出さない）
fn render_ema(config: &Config, guard: &TechnicalDataGuard) -> AnalysisResult {
    let weight = config.weight_ema;
    let mut description_lines: Vec<String> = Vec::new();

    description_lines.push("📊 【EMA（指数平滑移動平均）】".to_string());

    let short = guard.get_ema_short();
    let long = guard.get_ema_long();
    description_lines.push(format!("短期EMA: {:.2} / 長期EMA: {:.2}", short, long));

    let diff = short - long;
    let status = match diff {
        d if d > EMA_EQ_EPS => "🟢 ゴールデンクロス進行中（短期EMAが長期EMAを上回る）",
        d if d < -EMA_EQ_EPS => "📉 デッドクロス進行中（短期EMAが長期EMAを下回る）",
        _ => "➡️ EMAが同値圏（差が±0.01未満）→ スコア変動なし",
    };
    description_lines.push(status.to_string());

    // ★ スコアはSoT原則で Guard に格納済みの値のみを参照（再計算しない）
    let base_score = guard.get_ema_score().unwrap_or(0.0).round() as i32;
    let adjusted = base_score as f64 * weight;

    description_lines.push(format!(
        "📝 スコア調整値({:.1}) = スコア({}) × Weight({:.1})",
        adjusted, base_score, weight
    ));

    AnalysisResult {
        indicator_name: "EMA".to_string(),
        description: description_lines,
        score: base_score as f64,
    }
}

/// SMA（単純移動平均）の表示（セキュアアクセス：TechnicalDataGuard経由）
fn render_sma(config: &Config, guard: &TechnicalDataGuard) -> AnalysisResult {
    let short = guard.get_sma_short();
    let long = guard.get_sma_long();
    let weight = config.weight_sma;

    let mut description_lines: Vec<String> = Vec::new();
    description_lines.push("📊 【SMA（単純移動平均）】".to_string());
    description_lines.push(format!("短期SMA: {:.2} / 長期SMA: {:.2}", short, long));

    // クロス判定出力
    if short > long {
        description_lines.push("📈 ゴールデンクロス発生中（短期SMAが長期SMAを上回る）".to_string());
    } else if short < long {
        description_lines.push("📉 デッドクロス発生中（短期SMAが長期SMAを下回る）".to_string());
    } else {
        description_lines.push("➖ SMAが一致：クロス傾向なし".to_string());
    }

    // スコア取得と調整出力
    match guard.get_sma_score().map(|v| v as i32) {
        Some(base_score) => {
            let adjusted_score = base_score as f64 * weight;
            description_lines.push(rank_sma_score(Some(base_score)).to_string());
            description_lines.push(format!(
                "📝 スコア調整値({:.1}) = スコア({}) × Weight({:.1})",
                adjusted_score, base_score, weight
            ));

            AnalysisResult {
                indicator_name: "SMA".to_string(),
                description: description_lines,
                score: base_score as f64,
            }
        }
        None => {
            description_lines.push("⚠️ SMAスコア情報なし".to_string());
            AnalysisResult {
                indicator_name: "SMA".to_string(),
                description: description_lines,
                score: 0.0,
            }
        }
    }
}

fn rank_sma_score(sma_score: Option<i32>) -> &'static str {
    match sma_score {
        Some(2) => "🟢 短期SMAが長期より大幅に上 → 強い上昇トレンド → スコア+2加点",
        Some(1) => "🟢 短期SMAが長期よりやや上 → 上昇トレンド → スコア+1加点",
        Some(0) => "➡️ SMAが同値圏 → スコア変動なし",
        Some(-1) => "🔴 短期SMAが長期よりやや下 → 下降トレンド → スコア-1減点",
        Some(-2) => "🔴 短期SMAが長期より大幅に下 → 強い下降トレンド → スコア-2減点",
        _ => "⚠️ SMAスコア情報なし",
    }
}
/// ADXの表示（セキュアアクセス: TechnicalDataGuard経由）
/// ADXスコアのランク評価（スコア → ラベル文字列）
fn rank_adx_score(adx_score: Option<i32>) -> &'static str {
    match adx_score {
        Some(2) => "🟢 ADXが非常に強い（40以上）→ 強いトレンド継続 → スコア+2加点",
        Some(1) => "🟢 ADXがやや強い（25以上40未満）→ トレンド発生 → スコア+1加点",
        Some(0) => "➡️ ADXが中立（20以上25未満）→ 様子見",
        Some(-1) => "🔴 ADXがやや弱い（15以上20未満）→ トレンド弱まる → スコア-1減点",
        Some(-2) => "🔴 ADXが非常に弱い（15未満）→ トレンド消失 → スコア-2減点",
        _ => "⚠️ ADXスコア不明",
    }
}

fn render_adx(config: &Config, guard: &TechnicalDataGuard) -> AnalysisResult {
    let mut description_lines: Vec<String> = Vec::new();
    description_lines.push("📊 【ADX（平均方向性指数）】".to_string());

    match guard.get_adx() {
        Some(adx) => {
            description_lines.push(format!("現在のADX: {:.2}", adx));

            // 極端な値の警告
            if adx >= 50.0 {
                description_lines.push(format!(
                    "⚠️ ADXが50以上（{:.2}）→ 非常に強いトレンド → 反転リスクに警戒",
                    adx
                ));
            } else if adx <= 10.0 {
                description_lines.push(format!(
                    "⚠️ ADXが10以下（{:.2}）→ トレンド不在（レンジ相場） → 仕掛け注意",
                    adx
                ));
            }

            match guard.get_adx_score().map(|v| v as i32) {
                Some(base_score) => {
                    let adjusted_score = base_score as f64 * config.weight_adx;
                    description_lines.push(rank_adx_score(Some(base_score)).to_string());
                    description_lines.push(format!(
                        "📝 スコア調整値({:.1}) = スコア({}) × Weight({:.1})",
                        adjusted_score, base_score, config.weight_adx
                    ));
                    AnalysisResult {
                        indicator_name: "ADX".to_string(),
                        description: description_lines,
                        score: base_score as f64,
                    }
                }
                None => {
                    description_lines.push("⚠️ ADXスコア情報なし".to_string());
                    AnalysisResult {
                        indicator_name: "ADX".to_string(),
                        description: description_lines,
                        score: 0.0,
                    }
                }
            }
        }
        None => {
            description_lines.push("⚠️ ADXデータなし".to_string());
            AnalysisResult {
                indicator_name: "ADX".to_string(),
                description: description_lines,
                score: 0.0,
            }
        }
    }
}

/// ROC（変化率）の表示（セキュアアクセス：TechnicalDataGuard経由）
fn rank_roc_score(roc_score: Option<i32>) -> &'static str {
    match roc_score {
        Some(2) => "🟢 ROCが大幅上昇 → 強い上昇トレンド → スコア+2加点",
        Some(1) => "🟢 ROCが上昇傾向 → スコア+1加点",
        Some(0) => "➡️ ROCが安定圏（±3%）→ スコア変動なし",
        Some(-1) => "🔴 ROCがやや下降 → スコア-1減点",
        Some(-2) => "🔴 ROCが大幅下降 → 強い下降トレンド → スコア-2減点",
        _ => "⚠️ ROCスコア情報なし",
    }
}

fn render_roc(config: &Config, guard: &TechnicalDataGuard) -> AnalysisResult {
    let mut description_lines: Vec<String> = Vec::new();
    description_lines.push("📊  【ROC（変化率）】".to_string());

    match guard.get_roc() {
        Some(roc) => {
            description_lines.push(format!("10日間のROC: {:.2}%", roc));

            // 極端なROCに対する警告
            if roc >= 15.0 {
                description_lines.push(format!(
                    "⚠️ ROCが+15%以上（{:.2}%）→ 短期的な過熱上昇、反落に警戒",
                    roc
                ));
            } else if roc <= -15.0 {
                description_lines.push(format!(
                    "⚠️ ROCが-15%以下（{:.2}%）→ パニック売りの可能性、反発に備えた注視を",
                    roc
                ));
            }

            match guard.get_roc_score().map(|v| v as i32) {
                Some(base_score) => {
                    let adjusted_score = base_score as f64 * config.weight_roc;
                    description_lines.push(rank_roc_score(Some(base_score)).to_string());
                    description_lines.push(format!(
                        "📝 スコア調整値({:.1}) = スコア({}) × Weight({:.1})",
                        adjusted_score, base_score, config.weight_roc
                    ));
                    AnalysisResult {
                        indicator_name: "ROC".to_string(),
                        description: description_lines,
                        score: base_score as f64,
                    }
                }
                None => {
                    description_lines.push("⚠️ ROCスコア情報なし".to_string());
                    AnalysisResult {
                        indicator_name: "ROC".to_string(),
                        description: description_lines,
                        score: 0.0,
                    }
                }
            }
        }
        None => {
            description_lines.push("⚠️ ROCデータが不足しています".to_string());
            AnalysisResult {
                indicator_name: "ROC".to_string(),
                description: description_lines,
                score: 0.0,
            }
        }
    }
}

/// ストキャスティクスの表示（セキュアアクセス：TechnicalDataGuard経由）
/// ストキャスティクススコアの判定文字列
fn rank_stochastics_score(stochastics_score: Option<i32>) -> &'static str {
    match stochastics_score {
        Some(2) => "🟢 %Kが10%以下 → 強い売られすぎと判断 → 買いシグナル → スコア+2加点",
        Some(1) => "🟢 %Kが20%以下 → 売られすぎと判断 → 買いシグナル → スコア+1加点",
        Some(0) => "➡️ %Kが中立圏（20〜80%） → シグナルなし → スコア変動なし",
        Some(-1) => "🔴 %Kが80%以上 → 買われすぎと判断 → 売りシグナル → スコア-1減点",
        Some(-2) => "🔴 %Kが90%以上 → 強い買われすぎと判断 → 売りシグナル → スコア-2減点",
        _ => "⚠️ ストキャスティクススコア不明",
    }
}
fn render_stochastics(config: &Config, guard: &TechnicalDataGuard) -> AnalysisResult {
    let mut description_lines: Vec<String> = Vec::new();
    description_lines.push("📊 【ストキャスティクス】".to_string());

    let k_opt = guard.get_stochastics_k();
    let d_opt = guard.get_stochastics_d();

    match (k_opt, d_opt) {
        (Some(k), Some(d)) => {
            description_lines.push(format!("現在の%K: {:.2}% / 現在の%D: {:.2}%", k, d));
            if k == 0.0 && d == 0.0 {
                description_lines.push("⚠️ %Kおよび%Dが0.00%に張り付き → 極端な売られすぎ水準 → リバウンドの可能性あり（注目シグナル）".to_string());
            }
        }
        (Some(_), None) => {
            description_lines.push("⚠️ %Dデータが不足しています".to_string());
        }
        (None, _) => {
            description_lines.push("⚠️ %Kデータが不足しています".to_string());
        }
    }

    match guard.get_stochastics_score().map(|v| v as i32) {
        Some(base_score) => {
            let adjusted_score = base_score as f64 * config.weight_stochastics;
            description_lines.push(rank_stochastics_score(Some(base_score)).to_string());
            description_lines.push(format!(
                "📝 スコア調整値({:.1}) = スコア({}) × Weight({:.1})",
                adjusted_score, base_score, config.weight_stochastics
            ));
            AnalysisResult {
                indicator_name: "ストキャスティクス".to_string(),
                description: description_lines,
                score: base_score as f64,
            }
        }
        None => {
            description_lines.push("⚠️ ストキャスティクススコア情報なし".to_string());
            AnalysisResult {
                indicator_name: "ストキャスティクス".to_string(),
                description: description_lines,
                score: 0.0,
            }
        }
    }
}

/// ボリンジャーバンドの表示（セキュアアクセス：TechnicalDataGuard経由）
fn render_bollinger(config: &Config, guard: &TechnicalDataGuard) -> AnalysisResult {
    let upper: f64 = guard.get_bb_upper();
    let lower: f64 = guard.get_bb_lower();
    let percent_b: f64 = guard.get_bb_percent_b();
    let bandwidth_pct: f64 = guard.get_bb_bandwidth();
    let weight: f64 = config.weight_bollinger;

    let mut description_lines: Vec<String> = Vec::new();
    description_lines.push("📊 【ボリンジャーバンド】".to_string());
    description_lines.push(format!("上限 {:.2} / 下限 {:.2}", upper, lower));

    if (upper - lower).abs() < f64::EPSILON {
        description_lines
            .push("⚠️ バンド幅が0に近いため、%b/帯幅の解釈に注意（計算不安定）".to_string());
    }

    description_lines.push(format!(
        "%b indicator: {:.2} / 帯幅(Bandwidth): {:.1}%",
        percent_b, bandwidth_pct
    ));
    // ✅ 帯幅のスクイーズ判定
    let bw = guard.get_bb_bandwidth(); // ← これを使う
    let th = config.bb_bandwidth_squeeze_pct; // 設定のしきい値(%)

    if bw <= th {
        description_lines.push(format!(
            "⚠️ スクイーズ進行中（帯幅が設定閾値 {:.1}% 以下）",
            th
        ));
    } else {
        description_lines.push(format!(
            "ℹ️ 帯幅は設定閾値 {:.1}% を上回り、スクイーズ未発生",
            th
        ));
    }

    // ✅ 位置情報は“%b”だけで1行に統一（重複/矛盾を排除）
    let state_line = if percent_b > 1.0 {
        format!(
            "⚠️ 上限ブレイク（%b {:.2}）→ 伸び一巡後の反動に注意",
            percent_b
        )
    } else if percent_b < 0.0 {
        format!(
            "⚠️ 下限ブレイク（%b {:.2}）→ リバウンド/続落の分岐に注意",
            percent_b
        )
    } else {
        "➡️ 現在価格がバンド内 → 中立".to_string()
    };
    description_lines.push(state_line);

    // ✅ スコアは数値出力のみ（説明は重複させない）
    //    ただし“バンド内”かつ ±評価が付いた時だけ、短いヒントを追加
    match guard.get_bollinger_score().map(|v| v as i32) {
        Some(base) => {
            let adjusted = base as f64 * weight;

            if (0.0..=1.0).contains(&percent_b) && base != 0 {
                let hint = if base > 0 {
                    "🟢 バンド内で下側優位（反発寄り）"
                } else {
                    "🔴 バンド内で上側優位（過熱寄り）"
                };
                description_lines.push(format!(
                    "{} → スコア{:+}{}",
                    hint,
                    base,
                    if base > 0 { "加点" } else { "減点" }
                ));
            }

            description_lines.push(format!(
                "📝 スコア調整値({:.1}) = スコア({}) × Weight({:.1})",
                adjusted, base, weight
            ));

            AnalysisResult {
                indicator_name: "ボリンジャーバンド".to_string(),
                description: description_lines,
                score: base as f64,
            }
        }
        None => {
            description_lines.push("⚠️ ボリンジャーバンドスコア情報なし".to_string());
            AnalysisResult {
                indicator_name: "ボリンジャーバンド".to_string(),
                description: description_lines,
                score: 0.0,
            }
        }
    }
}

/// フィボナッチリトレースメントの表示（セキュアアクセス：TechnicalDataGuard経由）
fn render_fibonacci(config: &Config, guard: &TechnicalDataGuard) -> AnalysisResult {
    let weight = config.weight_fibonacci;
    let mut description_lines: Vec<String> = Vec::new();
    description_lines.push("📊 【フィボナッチリトレースメント】".to_string());
    description_lines.push("💡 トレンド内の押し目や戻り目を判断するための価格帯".to_string());

    if let (Some(level_38_2), Some(level_50), Some(level_61_8)) = (
        guard.get_fibo_38_2(),
        guard.get_fibo_50_0(),
        guard.get_fibo_61_8(),
    ) {
        description_lines.push(format!(
            "38.2%: {:.2} / 50.0%: {:.2} / 61.8%: {:.2}",
            level_38_2, level_50, level_61_8
        ));

        // base_score は Guard に保存済み（-2,-1,0,1,2）
        let base_score = guard.get_fibonacci_score().map(|v| v.round() as i32);
        let band_line = match base_score {
            Some(2) => format!(
                "🟢 終値が50%（{:.2}）より+2.00超 → 非常に強い上昇 → スコア+2",
                level_50
            ),
            Some(1) => format!(
                "🟢 終値が50%（{:.2}）より+0.50超 → 上昇傾向 → スコア+1",
                level_50
            ),
            Some(0) => format!("➡️ 終値が50%（{:.2}）±0.50内 → 中立（0）", level_50),
            Some(-1) => format!(
                "🔴 終値が50%（{:.2}）より-0.50超 → 下降傾向 → スコア-1",
                level_50
            ),
            Some(-2) => format!(
                "🔴 終値が50%（{:.2}）より-2.00超 → 非常に強い下落 → スコア-2",
                level_50
            ),
            Some(other) => format!("⚠️ 想定外スコア({}) → 中立扱い（0）", other), // フォールバック
            None => "⚠️ フィボナッチスコア情報なし".to_string(),
        };
        description_lines.push(band_line);
    } else {
        description_lines.push("⚠️ フィボナッチデータが不足しています".to_string());
    }

    match guard.get_fibonacci_score().map(|v| v as i32) {
        Some(base_score) => {
            let adjusted_score = base_score as f64 * weight;

            description_lines.push(format!(
                "📝 スコア調整値({:.1}) = スコア({}) × Weight({:.1})",
                adjusted_score, base_score, weight
            ));
            AnalysisResult {
                indicator_name: "フィボナッチ".to_string(),
                description: description_lines,
                score: base_score as f64,
            }
        }
        None => {
            description_lines.push("⚠️ フィボナッチスコア情報なし".to_string());
            AnalysisResult {
                indicator_name: "フィボナッチ".to_string(),
                description: description_lines,
                score: 0.0,
            }
        }
    }
}

/// VWAP（日足簡易版）の表示（セキュアアクセス：TechnicalDataGuard経由）
fn rank_vwap_score(vwap_score: Option<i32>) -> &'static str {
    match vwap_score {
        Some(2) => "🟢 VWAPが現在価格より大幅に下 → 強い買いシグナル → スコア+2加点",
        Some(1) => "🟢 VWAPが現在価格よりやや下 → 買いシグナル → スコア+1加点",
        Some(0) => "➡️ VWAPと同水準（±1.0以内）→ スコア変動なし",
        Some(-1) => "🔴 VWAPが現在価格よりやや上 → 売りシグナル → スコア-1減点",
        Some(-2) => "🔴 VWAPが現在価格より大幅に上 → 強い売りシグナル → スコア-2減点",
        _ => "⚠️ VWAPスコア不明",
    }
}

/// VWAP（日足簡易版）の表示（セキュアアクセス：TechnicalDataGuard経由）
fn render_vwap(config: &Config, guard: &TechnicalDataGuard) -> AnalysisResult {
    let weight = config.weight_vwap;
    let mut description_lines: Vec<String> = Vec::new();
    description_lines.push("📊 【VWAP（日足簡易版）】".to_string());
    description_lines.push("⚠️ VWAPは高値・安値・終値をもとに算出した簡易版です。正式な出来高加重平均価格とは異なる場合があります。".to_string());

    if let Some(vwap_value) = guard.get_vwap() {
        description_lines.push(format!("VWAP値: {:.2}", vwap_value));
    } else {
        description_lines.push("⚠️ VWAPデータが不足しています".to_string());
    }

    match guard.get_vwap_score().map(|v| v as i32) {
        Some(base_score) => {
            let adjusted_score = base_score as f64 * weight;
            description_lines.push(rank_vwap_score(Some(base_score)).to_string());
            description_lines.push(format!(
                "📝 スコア調整値({:.1}) = スコア({}) × Weight({:.1})",
                adjusted_score, base_score, weight
            ));
            AnalysisResult {
                indicator_name: "VWAP".to_string(),
                description: description_lines,
                score: base_score as f64,
            }
        }
        None => {
            description_lines.push("⚠️ VWAPスコア情報なし".to_string());
            AnalysisResult {
                indicator_name: "VWAP".to_string(),
                description: description_lines,
                score: 0.0,
            }
        }
    }
}

/// 一目均衡表の転換線・基準線・スコアを表示（セキュアアクセス：TechnicalDataGuard経由）
fn rank_ichimoku_score(ichimoku_score: Option<i32>) -> &'static str {
    match ichimoku_score {
        Some(2) => "🟢 転換線が基準線より大幅に上 → 強い買い圧力 → スコア+2加点",
        Some(1) => "🟢 転換線が基準線よりやや上 → 買い優勢 → スコア+1加点",
        Some(0) => "➡️ 転換線と基準線が同値圏 → トレンドなし → スコア変動なし",
        Some(-1) => "🔴 転換線が基準線よりやや下 → 売り優勢 → スコア-1減点",
        Some(-2) => "🔴 転換線が基準線より大幅に下 → 強い売り圧力 → スコア-2減点",
        _ => "⚠️ 一目均衡表スコア不明",
    }
}
/// 一目均衡表の転換線・基準線・スコアを表示（セキュアアクセス：TechnicalDataGuard経由）
fn render_ichimoku(config: &Config, guard: &TechnicalDataGuard) -> AnalysisResult {
    let weight = config.weight_ichimoku;
    let mut description_lines: Vec<String> = Vec::new();
    description_lines.push("📊 【一目均衡表】".to_string());

    if let (Some(tenkan), Some(kijun)) = (guard.get_tenkan_sen(), guard.get_kijun_sen()) {
        description_lines.push(format!("転換線: {:.2} / 基準線: {:.2}", tenkan, kijun));
        if tenkan > kijun {
            description_lines
                .push("📈 ゴールデンクロス発生中（転換線が基準線を上回る）".to_string());
        } else if tenkan < kijun {
            description_lines.push("📉 デッドクロス発生中（転換線が基準線を下回る）".to_string());
        } else {
            description_lines.push("➡️ 転換線と基準線が交差中（横ばい）".to_string());
        }
        if kijun != 0.0 {
            let gap_ratio = ((tenkan - kijun) / kijun).abs() * 100.0;
            if gap_ratio < 1.0 {
                description_lines.push(format!(
                    "💡 クロス直後の接近状態（乖離 {:.2}%）→ トレンド確定には弱い傾向",
                    gap_ratio
                ));
            } else if gap_ratio > 5.0 {
                description_lines.push(format!(
                    "💡 クロス乖離が大きい（乖離 {:.2}%）→ 強いトレンドの可能性",
                    gap_ratio
                ));
            }
        }
    } else {
        description_lines.push("⚠️ 一目均衡表データが不足しています".to_string());
    }

    match guard.get_ichimoku_score().map(|v: f64| v as i32) {
        Some(base_score) => {
            let adjusted_score = base_score as f64 * weight;
            description_lines.push(rank_ichimoku_score(Some(base_score)).to_string());
            description_lines.push(format!(
                "📝 スコア調整値({:.1}) = スコア({}) × Weight({:.1})",
                adjusted_score, base_score, weight
            ));
            AnalysisResult {
                indicator_name: "一目均衡表".to_string(),
                description: description_lines,
                score: base_score as f64,
            }
        }
        None => {
            description_lines.push("⚠️ 一目均衡表スコア情報なし".to_string());
            AnalysisResult {
                indicator_name: "一目均衡表".to_string(),
                description: description_lines,
                score: 0.0,
            }
        }
    }
}
/// 単極ゲージ（Seller/Buyerの見た目長さ差を解消）。例: 「Buyer [.....█████] Seller」
fn render_unipolar_gauge_rtl(
    percent: u8,
    left_label: &str,
    right_label: &str,
    width: usize,
) -> String {
    // 充填は四捨五入で統一（Seller/Buyerの見た目長さ差を解消）
    let w = width.max(10);
    let filled = (((percent as usize) * w) + 50) / 100; // round(percent * w / 100)
    let empty = w - filled;

    // filled + empty = width を常に満たす
    let bar = format!("[{}{}]", ".".repeat(empty), "█".repeat(filled));
    format!(
        "{left} {bar} {right}",
        left = left_label,
        right = right_label
    )
}

/// 両極ゲージ（中央'|'、正は左へ・負は右へ塗る）。例: 「買い [█████|..........] 売り」
/// 幅は呼び出し側で 50 指定（ここでは安全側で最小12を確保）。中央'|'を配置。
#[allow(clippy::needless_range_loop)]
fn render_bipolar_gauge_lr(score_ratio: f64, width: usize) -> String {
    // 幅は呼び出し側で 50 指定（ここでは安全側で最小12を確保）。中央'|'を配置。
    let w = width.max(12);
    let mid = w / 2;

    // |score_ratio| を中央半幅にスケールし、丸めて段階化。上限は mid。
    let blocks = ((score_ratio.abs() * mid as f64).round() as usize).min(mid);

    // ベースは '.' で埋め、中央に '|'。filled + empty = w を維持。
    let mut v = vec!['.'; w];
    if mid < w {
        v[mid] = '|';
    }

    // 向きは仕様固定：左=買い、右=売り。
    // 正（買い）は左側（midの左）へ、負（売り）は右側（midの右）へ塗る。
    if score_ratio > 0.0 {
        for i in (mid.saturating_sub(blocks))..mid {
            v[i] = '█';
        }
    } else if score_ratio < 0.0 {
        for i in (mid + 1)..(mid + 1 + blocks) {
            if i < w {
                v[i] = '█';
            }
        }
    }

    // バー色は判定色と一致させる（'█' のみ着色）。色名は get_color_for_score に準拠。
    let color = get_color_for_score(score_ratio);
    let bar: String = v
        .into_iter()
        .map(|c| {
            if c == '█' {
                match color {
                    "green" => c.to_string().green().to_string(),
                    "yellow" => c.to_string().yellow().to_string(),
                    "orange" => c.to_string().truecolor(255, 165, 0).to_string(),
                    "red" => c.to_string().red().to_string(),
                    "white" => c.to_string().white().to_string(), // 中立近傍
                    _ => c.to_string(),
                }
            } else {
                c.to_string()
            }
        })
        .collect();

    // 文言は仕様固定：「買い+100％[バー] -100% 売り」
    format!("買い+100％[{bar}] -100% 売り")
}

fn stance_caption(s: &Stance) -> &'static str {
    match s {
        Stance::Buyer => "Buyer",
        Stance::Holder => "Holder",
        Stance::Seller => "Seller",
    }
}
///グラフ色分け
fn get_color_for_score(score_ratio: f64) -> &'static str {
    match score_ratio {
        r if r >= 0.8 => "green", // 強い買い
        r if r >= 0.6 => "green", // やや買い
        r if r >= 0.4 => "green",
        r if r >= 0.2 => "yellow",
        r if r >= 0.0 => "yellow",
        r if r >= -0.2 => "white",
        r if r >= -0.4 => "orange",
        r if r >= -0.6 => "orange",
        r if r >= -0.8 => "red",
        _ => "red",
    }
}
///CSVヘッダー出力
fn generate_csv_header(config: &Config) {
    let mut headers = vec![
        "ticker",
        "date",
        "close",
        "prev_close",
        "diff",
        "diff_pct",
        "macd",
        "signal",
        "rsi",
        "score",
    ];

    for indicator in &config.enabled_extensions {
        match indicator {
            ExtensionIndicator::Ema => {
                headers.extend_from_slice(&["ema_short", "ema_long", "ema_score"]);
            }
            ExtensionIndicator::Sma => {
                headers.extend_from_slice(&["sma_short", "sma_long", "sma_score"]);
            }
            ExtensionIndicator::Roc => {
                headers.extend_from_slice(&["roc", "roc_score"]);
            }
            ExtensionIndicator::Adx => {
                headers.extend_from_slice(&["adx", "adx_score"]);
            }
            ExtensionIndicator::Stochastics => {
                headers.extend_from_slice(&["stoch_k", "stoch_d", "stoch_score"]);
            }
            ExtensionIndicator::Bollinger => {
                headers.extend_from_slice(&[
                    "bb_upper",
                    "bb_lower",
                    "percent_b",
                    "bandwidth_%",
                    "bb_score",
                ]);
            }
            ExtensionIndicator::Fibonacci => {
                headers.extend_from_slice(&["fibo_38_2", "fibo_50_0", "fibo_61_8", "fibo_score"]);
            }
            ExtensionIndicator::Vwap => {
                headers.extend_from_slice(&["vwap", "vwap_score"]);
            }
            ExtensionIndicator::Ichimoku => {
                headers.extend_from_slice(&["tenkan", "kijun", "ichimoku_score"]);
            }
        }
    }

    headers.push("final_score");

    println!("{}", headers.join(",")); // ✅ 出力ここで完結
}

// ログ保存処理（CSV/JSON対応、TechnicalDataGuard経由でセキュア設計）
fn save_technical_log(
    config: &Config,
    guard: &TechnicalDataGuard,
    results: &[AnalysisResult],
) -> Result<(), Box<dyn std::error::Error>> {
    let snap = calculate_final_score_snapshot(config, guard); // ← SoT: 計算は一度だけ

    match config.log_format.to_lowercase().as_str() {
        "csv" => {
            if config.stdout_log {
                let row = generate_technical_csv_row(guard, results, &snap)?;
                println!("{}", row);
                return Ok(());
            }
            let base_dir = Path::new(&config.log_dir);
            let dir_path = if config.log_flat {
                base_dir.to_path_buf()
            } else {
                base_dir.join(guard.get_ticker())
            };
            create_dir_all(&dir_path)?;

            let file = OpenOptions::new()
                .create(true)
                .append(config.data_append)
                .write(true)
                .open(dir_path.join(format!("{}.csv", guard.get_ticker())))?;
            let mut writer = BufWriter::new(file);
            let row = generate_technical_csv_row(guard, results, &snap)?;
            writeln!(writer, "{}", row)?;
            Ok(())
        }
        "json" => save_technical_log_json(guard, config, &snap),
        other => Err(format!("❌ 未対応のログフォーマット: {}", other).into()),
    }
}

// ② CSV行生成：final_score はスナップショットから（再計算しない）
fn generate_technical_csv_row(
    guard: &TechnicalDataGuard,
    results: &[AnalysisResult],
    snap: &FinalScoreSnapshot, // ← 追加
) -> Result<String, Box<dyn std::error::Error>> {
    let mut values = vec![
        guard.get_ticker().to_string(),
        guard.get_date().to_string(),
        format!("{:.2}", guard.get_close()),
        format!("{:.2}", guard.get_previous_close()),
        format!("{:+.2}", guard.get_price_diff()),
        format!("{:+.2}", guard.get_price_diff_percent()),
        format!("{:.4}", guard.get_macd()),
        format!("{:.4}", guard.get_signal()),
        format!("{:.2}", guard.get_rsi()),
        (guard.get_signal_score() as i32).to_string(),
    ];

    for res in results {
        /* 既存の match ブロックは変更なし */
        match res.indicator_name.as_str() {
            "EMA" => {
                values.push(format!("{:.2}", guard.get_ema_short()));
                values.push(format!("{:.2}", guard.get_ema_long()));
                values.push(score_to_string(Some(res.score)));
            }
            "SMA" => {
                values.push(format!("{:.2}", guard.get_sma_short()));
                values.push(format!("{:.2}", guard.get_sma_long()));
                values.push(score_to_string(Some(res.score)));
            }
            "ROC" => {
                values.push(opt_f64(guard.get_roc()));
                values.push(score_to_string(Some(res.score)));
            }
            "ADX" => {
                values.push(opt_f64(guard.get_adx()));
                values.push(score_to_string(Some(res.score)));
            }
            "Stochastics" => {
                values.push(opt_f64(guard.get_stochastics_k()));
                values.push(opt_f64(guard.get_stochastics_d()));
                values.push(score_to_string(Some(res.score)));
            }
            "Bollinger" => {
                values.push(format!("{:.2}", guard.get_bb_upper()));
                values.push(format!("{:.2}", guard.get_bb_lower()));
                values.push(format!("{:.2}", guard.get_bb_percent_b()));
                values.push(format!("{:.2}", guard.get_bb_bandwidth()));
                values.push(score_to_string(Some(res.score)));
            }
            "Fibonacci" => {
                values.push(opt_f64(guard.get_fibo_38_2()));
                values.push(opt_f64(guard.get_fibo_50_0()));
                values.push(opt_f64(guard.get_fibo_61_8()));
                values.push(score_to_string(Some(res.score)));
            }
            "VWAP" => {
                values.push(opt_f64(guard.get_vwap()));
                values.push(score_to_string(Some(res.score)));
            }
            "Ichimoku" => {
                values.push(opt_f64(guard.get_tenkan_sen()));
                values.push(opt_f64(guard.get_kijun_sen()));
                values.push(score_to_string(Some(res.score)));
            }
            _ => {}
        }
    }

    // ここだけ差し替え
    values.push(snap.total_score.to_string()); // 互換維持のため to_string() のまま

    Ok(values.join(","))
}

//Option<f64> の値を小数点2桁で文字列
fn opt_f64(val: Option<f64>) -> String {
    val.map(|v| format!("{:.2}", v)).unwrap_or_default()
}
//Option<i32> のスコア値を文字列に変換
fn score_to_string(val: Option<f64>) -> String {
    val.map(|v| format!("{:.2}", v)).unwrap_or_default()
}

// TechnicalDataGuardを通じてJSONログを1行出力（構造体の直接参照なし・セキュア設計対応）
// ③ JSON出力：同じスナップショットを使用（再計算しない）
fn save_technical_log_json(
    guard: &TechnicalDataGuard,
    config: &Config,
    snap: &FinalScoreSnapshot, // ← 追加
) -> Result<(), Box<dyn Error>> {
    let json_str = generate_technical_json_string(config, guard, snap)?;

    if config.stdout_log {
        println!("{}", json_str);
        return Ok(());
    }
    let base_dir = Path::new(&config.log_dir);
    let dir_path = if config.log_flat {
        base_dir.to_path_buf()
    } else {
        base_dir.join(guard.get_ticker())
    };
    create_dir_all(&dir_path)?;

    let file = OpenOptions::new()
        .create(true)
        .append(config.data_append)
        .write(true)
        .open(dir_path.join(format!("{}.json", guard.get_ticker())))?;
    let mut writer = BufWriter::new(file);
    writeln!(writer, "{}", json_str)?;
    Ok(())
}

// 拡張指標の設定に基づき、セキュアに構造体からJSON文字列を生成
// ④ JSON生成：final_score はスナップショットから（他は不変）
fn generate_technical_json_string(
    config: &Config,
    guard: &TechnicalDataGuard,
    snap: &FinalScoreSnapshot, // ← 追加
) -> Result<String, Box<dyn std::error::Error>> {
    let mut json_obj = json!({
        "ticker": guard.get_ticker(),
        "date": guard.get_date(),
        "close": guard.get_close(),
        "prev_close": guard.get_previous_close(),
        "diff": guard.get_price_diff(),
        "diff_pct": guard.get_price_diff_percent(),
        "macd": guard.get_macd(),
        "signal": guard.get_signal(),
        "rsi": guard.get_rsi(),
        "score": guard.get_signal_score()
    });

    for ext in &config.enabled_extensions {
        /* 既存の match はそのまま */
        match ext {
            ExtensionIndicator::Ema => {
                json_obj["ema_short"] = json!(guard.get_ema_short());
                json_obj["ema_long"] = json!(guard.get_ema_long());
                json_obj["ema_score"] = json!(guard.get_ema_score());
            }
            ExtensionIndicator::Sma => {
                json_obj["sma_short"] = json!(guard.get_sma_short());
                json_obj["sma_long"] = json!(guard.get_sma_long());
                json_obj["sma_score"] = json!(guard.get_sma_score());
            }
            ExtensionIndicator::Roc => {
                json_obj["roc"] = json!(guard.get_roc());
                json_obj["roc_score"] = json!(guard.get_roc_score());
            }
            ExtensionIndicator::Adx => {
                json_obj["adx"] = json!(guard.get_adx());
                json_obj["adx_score"] = json!(guard.get_adx_score());
            }
            ExtensionIndicator::Stochastics => {
                json_obj["stoch_k"] = json!(guard.get_stochastics_k());
                json_obj["stoch_d"] = json!(guard.get_stochastics_d());
                json_obj["stoch_score"] = json!(guard.get_stochastics_score());
            }
            ExtensionIndicator::Bollinger => {
                json_obj["bb_upper"] = json!(guard.get_bb_upper());
                json_obj["bb_lower"] = json!(guard.get_bb_lower());
                json_obj["percent_b"] = json!(guard.get_bb_percent_b());
                json_obj["bandwidth_%"] = json!(guard.get_bb_bandwidth());
                json_obj["bb_score"] = json!(guard.get_bollinger_score());
            }
            ExtensionIndicator::Fibonacci => {
                json_obj["fibo_38_2"] = json!(guard.get_fibo_38_2());
                json_obj["fibo_50_0"] = json!(guard.get_fibo_50_0());
                json_obj["fibo_61_8"] = json!(guard.get_fibo_61_8());
                json_obj["fibo_score"] = json!(guard.get_fibonacci_score());
            }
            ExtensionIndicator::Vwap => {
                json_obj["vwap"] = json!(guard.get_vwap());
                json_obj["vwap_score"] = json!(guard.get_vwap_score());
            }
            ExtensionIndicator::Ichimoku => {
                json_obj["tenkan"] = json!(guard.get_tenkan_sen());
                json_obj["kijun"] = json!(guard.get_kijun_sen());
                json_obj["ichimoku_score"] = json!(guard.get_ichimoku_score());
            }
        }
    }

    // ✅ 再計算禁止：スナップショットから入れる
    json_obj["final_score"] = json!(snap.total_score);

    Ok(serde_json::to_string(&json_obj)?)
}

///ニュース記事構造体
#[derive(Debug, Clone)]
struct Article {
    title: String,
    url: String,
    //  description: String,
    published_at: Option<String>,
}

// ===== 0) フローコントローラ：取得→整形→(必要なら)出力、同じ行を返す =====
// ニュースの取得と表示を司る。未設定/失敗は“スキップ明示”で継続する。
// 取得だけに専念し、整形は compose_news_lines、出力は print_lines_to_terminal に委譲

// --- 修正：収集→整形→出力しつつ、Vec<Article> を返す ---
async fn news_flow_controller(
    guard: &TechnicalDataGuard,
    config: &Config,
) -> Result<Vec<Article>, Box<dyn std::error::Error>> {
    // Braveキーは Config 経由のみ
    let brave_key_opt = {
        let s = config.brave_api_key.as_str();
        if s.trim().is_empty() {
            None
        } else {
            Some(s)
        }
    };

    // 収集（未設定/失敗は空Vec。再収集・追加整形はしない）
    let articles: Vec<Article> = match brave_key_opt {
        None => Vec::new(),
        Some(k) => run_news_once(guard, config, Some(k))
            .await
            .unwrap_or_default(), // 失敗時も空Vec
    };

    // 整形→出力（唯一の生成点＋プリンタ経由）
    let lines = compose_news_lines(guard, config, &articles);
    // 出力は show_news オプション時のみ
    if config.show_news {
        let brave_key_missing = config.brave_api_key.trim().is_empty();
        if brave_key_missing {
            println!("【注記】ニュース検索は BRAVE_API_KEY 未設定のためスキップ。");
        } else {
            print_lines_to_terminal(&lines);
        }
    }
    Ok(articles)
}

// ===== 1) 検索ワード加工：ログ用の1行（SoTはここ） =====
fn build_news_query_line_for_log(guard: &TechnicalDataGuard, config: &Config) -> String {
    let (country, _search_lang, _ui_lang) = news_locale_for_ticker(guard.get_ticker());
    let query_string = if let Some(ref custom) = config.custom_news_query {
        custom.clone()
    } else if country == "JP" {
        build_news_query_jp(
            guard.get_name(),
            jp_code_from_ticker(guard.get_ticker()).as_deref(),
            guard.get_ticker(),
            config.news_filter,
        )
    } else {
        build_news_query_us(
            guard.get_ticker(),
            Some(guard.get_name()),
            config.news_filter,
        )
    };
    let mode_tag = if config.news_filter {
        "[q-filtered]"
    } else {
        "[q-unfiltered]"
    };
    let freshness_log = if config.news_freshness.eq_ignore_ascii_case("all") {
        "all".to_string()
    } else {
        config.news_freshness.clone()
    };
    format!(
        "News query {mode}: {query}   (count={count}, freshness={fresh})",
        mode = mode_tag,
        query = query_string,
        count = config.news_count,
        fresh = freshness_log
    )
}
// ===== 2) 取得：Braveから収集→重複除去→日付降順（無音） =====
async fn run_news_once(
    guard: &TechnicalDataGuard,
    config: &Config,
    brave_key: Option<&str>, // 変更点：キーは Option で受ける
) -> Result<Vec<Article>, Box<dyn std::error::Error>> {
    let (country, search_lang, ui_lang) = news_locale_for_ticker(guard.get_ticker());

    let query_string = if let Some(ref custom) = config.custom_news_query {
        custom.clone()
    } else if country == "JP" {
        build_news_query_jp(
            guard.get_name(),
            jp_code_from_ticker(guard.get_ticker()).as_deref(),
            guard.get_ticker(),
            config.news_filter,
        )
    } else {
        build_news_query_us(
            guard.get_ticker(),
            Some(guard.get_name()),
            config.news_filter,
        )
    };

    let freshness_opt = if config.news_freshness.eq_ignore_ascii_case("all") {
        None
    } else {
        Some(config.news_freshness.as_str())
    };

    let mut articles: Vec<Article> = Vec::new();

    // キー未設定(None)のときは“無音で空”を返す（呼び出し側で「スキップ」と表示）
    if let Some(api_key) = brave_key {
        if let Ok(fetched) = fetch_articles_from_brave(
            &query_string,
            api_key,
            country,
            search_lang,
            ui_lang,
            config.news_count,
            freshness_opt,
        )
        .await
        {
            articles.extend(fetched);
        }
    }

    // URL正規化で重複排除
    let mut seen = std::collections::HashSet::new();
    articles.retain(|a| seen.insert(normalize_url(&a.url)));

    // 日付文字列の降順（None は最小扱い）
    articles.sort_by(|l, r| {
        let lk = l.published_at.as_deref().unwrap_or("");
        let rk = r.published_at.as_deref().unwrap_or("");
        rk.cmp(lk)
    });

    Ok(articles)
}

// ===== 3) 整形（唯一の生成点）：端末/LLM 共通の行を作る =====
fn compose_news_lines(
    guard: &TechnicalDataGuard,
    config: &Config,
    articles: &[Article],
) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(build_news_query_line_for_log(guard, config));
    lines.push(String::new());

    let cap = config.news_count;
    let shown = articles.len().min(cap);
    lines.push(format!(
        "=== News[{}]: {} 件（最大{}件表示） ===",
        guard.get_ticker(),
        shown,
        cap
    ));

    if shown == 0 {
        lines.push("（該当なし）".to_string());
        return lines;
    }

    for (index, article) in articles.iter().take(cap).enumerate() {
        let date_text = article.published_at.as_deref().unwrap_or("-");
        lines.push(format!(
            "{:02}. {} ({})",
            index + 1,
            article.title,
            date_text
        ));
        lines.push(format!("    {}", article.url));
    }
    lines
}

// ===== 4) 出力（プリンタ）：渡された行を順に出すだけ =====
fn print_lines_to_terminal(lines: &[String]) {
    for line in lines {
        println!("{}", line);
    }
}

// ===== 補助：クエリビルド／API呼び出し／ユーティリティ =====

// JP銘柄：社名/コード/ティッカーの OR、filter時は財務語を AND
fn build_news_query_jp(
    name_ja: &str,
    code_opt: Option<&str>,
    ticker: &str,
    use_filter: bool,
) -> String {
    if !use_filter {
        return format!(r#""{}""#, name_ja);
    }
    let entity_clause = match code_opt {
        Some(code) => format!(r#"("{}" OR {} OR {})"#, name_ja, code, ticker),
        None => format!(r#"("{}" OR {})"#, name_ja, ticker),
    };
    let finance_clause = r#"(決算 OR 業績 OR IR OR プレスリリース OR 開示 OR 適時開示 OR 配当 OR ガイダンス OR 提携 OR 買収 OR 株価 OR 株式 OR 投資家 OR "press release" OR earnings OR revenue OR profit OR guidance OR dividend OR "SEC filing")"#;
    format!(
        "{entity} AND {finance}",
        entity = entity_clause,
        finance = finance_clause
    )
}

// US銘柄：会社名/ティッカーの OR、filter時は財務語を AND
fn build_news_query_us(ticker: &str, company_name: Option<&str>, use_filter: bool) -> String {
    if !use_filter {
        return company_name
            .map(|n| format!(r#""{}""#, n))
            .unwrap_or_else(|| format!(r#""{}""#, ticker.to_ascii_uppercase()));
    }
    let ticker_upper = ticker.to_ascii_uppercase();
    let entity_clause = company_name
        .map(|n| format!("(\"{}\" OR {})", n, ticker_upper))
        .unwrap_or_else(|| format!("({})", ticker_upper));
    let finance_clause = "(stock OR earnings OR guidance OR \"SEC filing\" OR revenue OR profit OR dividend OR investor OR shareholder OR acquisition OR merger)";
    format!("{} AND {}", entity_clause, finance_clause)
}

// ティッカーからロケール判定
fn news_locale_for_ticker(ticker: &str) -> (&'static str, &'static str, &'static str) {
    if ticker.to_ascii_uppercase().ends_with(".T") {
        ("JP", "jp", "ja-JP")
    } else {
        ("US", "en", "en-US")
    }
}

// Brave News API 呼び出し（count/freshness は引数で制御）
async fn fetch_articles_from_brave(
    query_string: &str,
    api_key: &str,
    country: &str,
    search_lang: &str,
    ui_lang: &str,
    max_count: usize,
    freshness_opt: Option<&str>,
) -> Result<Vec<Article>, Box<dyn Error>> {
    let base = format!(
        "https://api.search.brave.com/res/v1/news/search?q={}&country={}&search_lang={}&ui_lang={}&count={}&offset=0&spellcheck=0",
        urlencoding::encode(query_string),
        country,
        search_lang,
        ui_lang,
        max_count
    );
    let url = if let Some(f) = freshness_opt {
        format!("{base}&freshness={f}")
    } else {
        base
    };

    let client = reqwest::Client::builder().gzip(true).build()?;
    let resp = client
        .get(&url)
        .header("Accept", "application/json")
        .header("Accept-Encoding", "gzip")
        .header("X-Subscription-Token", api_key)
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(format!("Brave API リクエスト失敗: {}", resp.status()).into());
    }

    let body: serde_json::Value = resp.json().await?;
    let mut out = Vec::new();
    if let Some(results) = body.get("results").and_then(|v| v.as_array()) {
        for item in results {
            let title = item
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("タイトル不明")
                .to_string();
            let url = item
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let published_at = item
                .get("page_fetched")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| {
                    item.get("page_age")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                });
            out.push(Article {
                title,
                url,
                published_at,
            });
        }
    }
    Ok(out)
}

// URL の #/?/末尾/ を落として正規化
fn normalize_url(url_str: &str) -> String {
    let mut u = url_str;
    if let Some(p) = u.find('#') {
        u = &u[..p];
    }
    if let Some(p) = u.find('?') {
        u = &u[..p];
    }
    u.trim_end_matches('/').to_string()
}

// --- LLMフロー入口：整形→保存→送信（送信はプロバイダ分岐） ---
async fn llm_flow_controller(
    config: &Config,
    guard: &TechnicalDataGuard,
    news_articles: Option<&[Article]>,
) -> Result<(), Box<dyn std::error::Error>> {
    let lines = compose_llm_prompt_lines(config, guard, news_articles).await?;
    let prompt = lines.join("\n");

    if config.debug_prompt {
        // 保存は送信有無に関係なく実行
        save_prompt_to_file(&prompt)?;
    }
    if config.silent {
        return Ok(()); // 送信スキップ
    }

    match config.llm_provider.as_str() {
        "openai" => openai_send_prompt(config, &prompt).await?,
        "gemini" => return Err("Gemini provider is not implemented yet.".into()),
        "azure" => return Err("Azure OpenAI provider is not implemented yet.".into()),
        other => return Err(format!("Unknown LLM provider: {}", other).into()),
    }
    Ok(())
}

/// --- LLM向けプロンプト行を唯一生成（SoT/DRY、ゲージ無し） ---
async fn compose_llm_prompt_lines(
    config: &Config,
    guard: &TechnicalDataGuard,
    news_articles: Option<&[Article]>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut lines = Vec::new();

    match &config.stance {
        Stance::Buyer => {
            lines.push(
                "私はこの株を持っておらず購入者を検討しています。買い手の視点でコメントください。"
                    .to_string(),
            );
            lines.push(String::new());
        }
        Stance::Seller => {
            lines.push(
                "私はこの株を売ろうと思っています。売り手の視点でコメントください。".to_string(),
            );
            lines.push(String::new());
        }
        Stance::Holder => {}
    }

    if config.macd_minus_ok {
        lines.push("⚠️ MACDがマイナス圏かつシグナルより上回っている場合に、買いシグナルを許容する設定が有効です".to_string());
        lines.push(String::new());
    }

    lines.push(format!(
        "📊 銘柄: {}（{}）",
        guard.get_name(),
        guard.get_ticker()
    ));
    lines.push(format!("📅 日付: {}", guard.get_date()));
    lines.push(format!("💰 終値: {:.2}", guard.get_close()));
    lines.push(format!("💰 前日終値: {:.2}", guard.get_previous_close()));
    lines.push(format!(
        "📊 前日比: {:+.2} ({:+.2}%)",
        guard.get_price_diff(),
        guard.get_price_diff_percent()
    ));
    lines.push(String::new());

    let basic = render_basic(config, guard);
    lines.extend(basic.description.clone());
    lines.push(String::new());

    let ext_results = render_extension(config, guard);
    for item in ext_results {
        if let ExtensionResult::Analysis(res) = item {
            lines.extend(res.description.clone());
            lines.push(String::new());
        }
    }

    let snap = calculate_final_score_snapshot(config, guard);
    for line in compose_final_score_lines_stance(&snap, &config.stance, true) {
        if !line.is_empty() {
            lines.push(line);
        }
    }
    lines.push(String::new());

    let mut news_task_directive =
        "対象が0件なら『株価に関係する評価対象ニュースはありません』と 1 行だけ記載。".to_string();

    if !config.no_news {
        let brave_key_missing = config.brave_api_key.trim().is_empty();

        if brave_key_missing {
            lines.push("【注記】ニュース検索は BRAVE_API_KEY 未設定のためスキップ。".to_string());
            lines.push(String::new());
            news_task_directive =
                "この実行ではニュース検索をスキップ。ニュース節には『ニュース検索をスキップ』と 1 行だけ記載。"
                    .to_string();
        } else {
            match news_articles {
                None => {
                    lines.push("【注記】ニュース取得に失敗したためスキップ。".to_string());
                    lines.push(String::new());
                    news_task_directive =
                        "この実行ではニュース取得に失敗しスキップ。ニュース節には『取得失敗によりスキップ』と 1 行だけ記載。"
                            .to_string();
                }
                Some([]) => {
                    lines.push("【注記】対象期間に該当ニュースなし。".to_string());
                    lines.push(String::new());
                }
                Some(slice) => {
                    let news_lines = compose_news_lines(guard, config, slice);
                    lines.extend(news_lines);
                    lines.push(String::new());
                    news_task_directive =
                        "以下の見出し群を、\
                        Tier A（一次性・数量性・直接性・近接性・信頼性が高い）/ \
                        Tier B（中）/ Tier C（低＝論評・再掲など）に仕分ける。\
                        各記事に対し、価格影響度（高/中/低/微小）を判定。\
                        Tier A/B は必ず列挙し、影響度が『低/微小』でも \
                        『ニュース価値は高いが価格影響は軽微（理由：金額相対小/反映が遠い/既報の焼き直し等）』と 1 行で明記。\
                        Tier C は“参考（価格影響なし）”として最大3件まで、非採用理由を 1 語（再掲/論評/一次性なし 等）で添える。\
                        新規数値の創作は禁止。"
                            .to_string();
                }
            }
        }
    }

    lines.push("【タスク】".to_string());
    lines.push(format!(
        "1. 投資家が注意すべきポイント（{}文字以内）",
        config.max_note_length
    ));
    lines.push(format!(
        "2. 1週間の短期目線（{}文字以内）",
        config.max_shortterm_length
    ));
    lines.push(format!(
        "3. 1ヶ月の中期目線（{}文字以内）",
        config.max_midterm_length
    ));
    lines.push(format!(
        "4. ニュースハイライト（{}字以内、株価に影響する情報のみ。芸能/スポーツ/宣伝は除外。{}）",
        config.max_news_length, news_task_directive
    ));
    lines.push(format!("5. 総評（{}字以内）", config.max_review_length));
    lines.push(String::new());

    lines.push("【執筆ガイド（ルールのみ）】".to_string());
    lines.push(
        "- 上のテクニカル出力の数値のみを根拠として使用。未提示の価格や新規数値の創作は禁止。"
            .to_string(),
    );
    lines.push("- レンジ/目安は、提示された水準（終値/EMA/SMA/VWAP/ボリ下限上限/フィボ各値）からのみ導出。".to_string());
    lines.push("- オシレーター用語は厳密に：RSI<30/ストキャス%K<20=売られすぎ、RSI>70/％K>80=買われすぎ。逆転表現は禁止。".to_string());
    let macd = guard.get_macd();
    let signal = guard.get_signal();
    let macd_policy = match (config.macd_minus_ok, macd < 0.0 && macd > signal) {
        (true, true) => "※『MACDマイナス許容』設定: 有効（今回“適用対象”）",
        (true, false) => "※『MACDマイナス許容』設定: 有効（今回“未適用”）",
        (false, _) => "※『MACDマイナス許容』設定: 無効",
    };
    lines.push(macd_policy.to_string());
    lines.push(
        "- ニュース0件時は“テクニカル主導”と明記。件数>0なら冒頭に要点の箇条書きから入る。"
            .to_string(),
    );
    lines.push("- 少なくとも2つのシナリオ（例：短期反発/続落/レンジ）を提示し、各々「条件→行動（エントリー/撤退/利確帯）」を具体化。".to_string());
    lines.push("- 小数は原則2桁。桁飛び・丸め過ぎ・矛盾記述は禁止。".to_string());
    lines.push("- 指標の略称は禁止。例　BBはダメ。ボリンジャーバンドと正しく出力".to_string());
    lines.push("【記述順序ルール】".to_string());
    lines.push("- 中期の反転条件は「EMA長期 → 一目基準線 → VWAP と Fib 38.2%（併記） → SMA長期」の順で列挙。".to_string());
    lines.push("- 短期シナリオは「SMA短期/EMA短期の回復 → Fib 50% → 一目転換線 → 一目基準線」を利確帯として段階記述。".to_string());
    lines.push("- 用語は「ボリンジャーバンド下限/上限」で統一（初出のみ正式名。以後は“BB下限/BB上限”略称可）。".to_string());
    lines.push(String::new());

    if let Some(note) = &config.openai_extra_note {
        if !note.trim().is_empty() {
            lines.push(format!("📝 追加ノート: {}", note.trim()));
        }
    }

    Ok(lines)
}

// --- OpenAI送信（キー未設定はヒント表示で優しくスキップ） ---
async fn openai_send_prompt(
    config: &Config,
    prompt: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if config.no_llm {
        return Ok(());
    }
    if config.llm_provider.trim() != "openai" {
        return Ok(());
    }

    let openai_key = if !config.openai_api_key.trim().is_empty() {
        config.openai_api_key.trim().to_string()
    } else {
        std::env::var("OPENAI_API_KEY").unwrap_or_default()
    };

    if openai_key.trim().is_empty() {
        eprintln!("⚠️ OpenAI APIキーが未設定のため送信をスキップしました。");
        eprintln!("   対応: `--openai-api-key <KEY>` または 環境変数 `OPENAI_API_KEY` を設定してください。");
        eprintln!("   tickwise.envファイルを使う場合は、`OPENAI_API_KEY=sk-xxxxxxxx` のように記述してください。");
        return Ok(());
    }

    let client = reqwest::Client::new();
    let res = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(&openai_key)
        .json(&serde_json::json!({
            "model": config.openai_model,
            "messages": [{ "role": "user", "content": prompt }],
        }))
        .send()
        .await?;

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        match status.as_u16() {
            400 => eprintln!("❌ 不正なリクエスト(400)。モデル名やパラメータを確認してください。"),
            401 => eprintln!("❌ 認証エラー(401)。APIキーが不正/期限切れの可能性。"),
            403 => eprintln!("⛔ アクセス拒否(403)。権限不足または機能が無効化。"),
            429 => eprintln!("⏳ レート制限(429)。時間を置いて再実行してください。"),
            500..=599 => eprintln!(
                "🛠️ 一時的な障害({}).時間を置いて再試行してください。",
                status
            ),
            _ => eprintln!("❌ リクエスト失敗({}): {}", status, body),
        }
        return Err(format!("OpenAI request failed: {}", status).into());
    }

    let json: serde_json::Value = res.json().await?;
    let content = json["choices"]
        .get(0)
        .and_then(|c| c["message"]["content"].as_str())
        .ok_or("OpenAI APIのレスポンス形式が不正です")?;
    println!("\n=== LLM Response  by {} ===\n", config.openai_model);
    println!("{}", content);
    Ok(())
}

// プロンプトを debug_prompt.txt に保存（短い版）
fn save_prompt_to_file(prompt: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::Write;
    let mut file = File::create("debug_prompt.txt")?;
    file.write_all(prompt.as_bytes())?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    from_filename("tickwise.env").ok();

    // ✅ 初期化（設定・キー・CSVエイリアス）
    let (config, ticker, ticker_name_map) = initialize_environment_and_config()?;

    // ✅ 株価データ取得
    //let market_data_list = fetch_market_data(&ticker, &marketstack_key,&config).await?;
    let market_data_list = fetch_market_data(&ticker).await?;

    let mut sorted_data = market_data_list.clone();
    sorted_data.sort_by(|a, b| a.date.cmp(&b.date));

    // ✅ 基本分析の構造体（セキュア）生成
    let mut guard = build_basic_technical_entry(
        &config,
        &sorted_data,
        &ticker_name_map,
        //fetched_company_name,
    )?;

    // ✅ 拡張分析スコアを必要に応じて格納（セキュア）
    if let Err(e) = evaluate_all_selected_extensions(&config, &sorted_data, &mut guard) {
        eprintln!("❌ 拡張テクニカル指標の評価に失敗しました: {}", e);
    }

    // ✅ 画面出力（構造体読み出しのみ）
    select_output_target(&config, &guard)?;
    // 画面表示＋記事の取得
    let articles = news_flow_controller(&guard, &config).await?;

    // LLM送信
    if !config.no_llm {
        let news_arg: Option<&[Article]> = if config.no_news {
            None
        } else {
            Some(articles.as_slice())
        };
        llm_flow_controller(&config, &guard, news_arg).await?;
    }

    Ok(())
}
