# Tickwise コマンドラインリファレンス

`tickwise` は、株価データのテクニカル分析とニュース要約を行う CLI ツールです。  
このドキュメントでは、主要なコマンドラインオプションとその意味・使いどころを説明します。

---

## 基本的な使い方

# 最もシンプルな例（テクニカル＋ニュース＋LLM）
tickwise -t 7203.T

# ニュースなし・テクニカル分析のみ
tickwise -t AAPL --no-news --no-llm

# ログだけ取りたい（CSVヘッダだけ確認）
tickwise --show-log-header
--ticker は通常必須ですが、--show-log-header だけを使う場合は省略できます。

1. 基本オプション
-t, --ticker <SYMBOL>
対象となる銘柄コードを指定します。

型: String

必須: 通常は必須（--show-log-header のみなら不要）

例:

日本株: 7203.T

米国株: AAPL, MSFT

インデックス系: 特殊文字を含む場合はクォートで囲む
例: 'S&P500'

メモ: ブローカー／データソース側のシンボル表記に合わせて指定します。

2. LLM / OpenAI 関連
--openai-api-key <KEY>
OpenAI の API キーを明示的に指定します。

型: String

既定: なし（環境変数で設定している場合は不要）

用途: 環境変数を使いたくない／一時的に別キーを試したいとき

-O, --no-llm
LLM へのアクセスを完全にスキップします。

型: bool

既定: false

効果:

ニュース要約やテキスト生成を行わず、

テクニカル分析のみを実行します。

用途:

オフライン環境

API クレジットを節約したいとき

LLM 側の挙動を切り離してテクニカルだけ検証したいとき

--llm-provider <NAME>
使用する LLM プロバイダを指定します。

型: String

既定: "openai"

許可値: openai（このバージョンでは OpenAI のみ）

将来的な拡張のためのフックです。現状は変更不要です。

-M, --openai-model <MODEL_NAME>
使用する OpenAI モデルを指定します。

型: String

既定: gpt-4.1-nano

例: gpt-4.1-mini, gpt-4.1

軽いモデルで高速に回したい／重いモデルで精度を見たい、などの切り替えに使います。

-d, --debug-prompt
LLM に送るプロンプトを送信せず、ファイル（debug_prompt.txt）に保存します。

型: bool

既定: false

効果:

実際の API 呼び出しを行わず、

Tickwise が LLM にどのような情報を渡すのか確認できます。

プロンプト設計のデバッグや、LLM なしでのレビュー用に使えます。

-x, --openai-extra-note <TEXT>
OpenAI へ送るプロンプトに、追加の一文を付与します。

型: String

例:

--openai-extra-note "配当利回りよりも成長性を重視して評価して"

--openai-extra-note "短期トレード目線を強めにして欲しい"

自分の投資スタイルを一時的に指示したいときに使えます。

3. ニュース関連オプション
-n, --no-news
ニュース検索を完全にスキップします。

型: bool

既定: false

効果:

ニュース API を呼びません。

テクニカル指標のみで評価を行います。

--brave-api-key <KEY>
Brave News API のキーを指定します。

型: String

既定: なし（環境変数で設定している場合は不要）

-q, --custom-news-query <QUERY>
ニュース検索に使うクエリを自由に指定します。

型: String

例:

-q "TSMC supply chain risk"

-q "レーザーテック 半導体 装置 受注"

銘柄コードでは拾えないテーマや周辺ニュースを狙う場合に使います。

--news-filter
ニュース検索ワードを金融・投資寄りの用語で絞り込みます。

型: bool

既定: false

環境変数 NEWS_FILTER=True でも有効化可能

効果:

一般ニュースよりも、決算／業績／レーティングなど「投資判断に関わりやすい記事」に寄せます。

--news-count <N>
取得するニュース件数の上限を指定します。

型: usize

範囲: 1..50

既定:

明示指定なしの場合

NEWS_COUNT 環境変数があればそれを使用

なければ「モード既定」

フィルタ OFF: 50

フィルタ ON (--news-filter): 20

ニュースが多すぎて扱いづらい場合は --news-count 10 などで絞ると見やすくなります。

--news-freshness <pd|pw|pm|py|all>
どの期間のニュースを対象にするか指定します。

型: String

許可値:

pd : 過去 1 日

pw : 過去 1 週間

pm : 過去 1 ヶ月

py : 過去 1 年

all: 制限なし

既定:

NEWS_FRESHNESS 環境変数があればそれを使用

なければ「モード既定」

フィルタ OFF: pm（1 ヶ月）

フィルタ ON (--news-filter): pw（1 週間）

短期トレード寄りなら pw 〜 pd、長期の構造変化を見たいなら pm や py を選ぶイメージです。

--show-news
ニュース文章をターミナル出力に含めます。

型: bool

既定: false（または環境変数ベースの既定）

効果:

有効化すると、ニュースハイライトが CLI 上に表示されます。

ログだけ取りたい場合は OFF にしておくと画面がスッキリします。

4. テクニカル指標の閾値・オプション
RSI 関連
--buy-rsi <VALUE>
「売られすぎ（買い検討）」とみなす RSI の閾値。

型: f64

既定: 30.0

例:

値を小さくすると、より強い売られすぎ局面だけを拾います（例: --buy-rsi 25）。

--sell-rsi <VALUE>
「買われすぎ（売り／利益確定検討）」とみなす RSI の閾値。

型: f64

既定: 70.0

例:

値を大きくすると、かなりの過熱感が出たときだけ警戒するようになります（例: --sell-rsi 80）。

MACD 関連
--macd-diff-low <VALUE>
MACD の差分が「小さい」と判定される閾値。

型: f64

既定: 2.0

--macd-diff-mid <VALUE>
MACD の差分が「中程度」と判定される閾値。

型: f64

既定: 10.0

Tickwise では、MACD の差分の大きさによって「トレンドの勢い」や「転換の近さ」を段階的に評価します。
diff-low / diff-mid を変えることで、自分の許容する「勢い」の強さをチューニングできます。

-m, --macd-minus-ok
MACD がマイナス圏（ゼロラインより下）でも買いシグナルを許容します。

型: bool

既定: false

効果:

通常は「MACD がマイナス圏 = 下落トレンド／調整中」とみなし、買いに慎重になりますが、

このオプションを付けると、押し目買い・逆張り寄りの判断を許容します。

逆張りスイングトレードを積極的に狙う時に ON にするイメージです。

テクニカル指標の有効化フラグ
これらは「どの指標をスコアに含めるか」を制御します。

--ema : EMA（指数平滑移動平均）分析を有効化

--sma : SMA（単純移動平均）分析を有効化

--bollinger : ボリンジャーバンド分析を有効化

--fibonacci : フィボナッチ・リトレースメント分析を有効化

--stochastics : ストキャスティクス（%K, %D）分析を有効化

--adx : ADX（トレンド強度）分析を有効化

--roc : ROC（Rate of Change）分析を有効化

--vwap : VWAP 分析を有効化

--ichimoku : 一目均衡表分析を有効化

それぞれ bool で、指定したものだけがスコア計算に含まれます。

ボリンジャーバンド関連
--bb-bandwidth-squeeze-pct <VALUE>
ボリンジャーバンドの帯域幅（%BW）が「スクイーズ（縮小）」と判定される閾値（％）。

型: f64

既定: 8.0（例: 8% 以下ならスクイーズ判定）

用途:

レンジ相場からの「エネルギー溜め込み」局面を検出するためのしきい値。

値を小さくすると、より「きつい縮小」だけを拾うようになります。

5. スコアの重み付け（ウェイト）
Tickwise は複数の指標からスコアを作り、それらを合成して総合判断を行います。
以下のオプションは、各カテゴリの重み（ウェイト）を調整するためのものです。

すべて:

型: f64

既定: 1.0

推奨範囲: 0.5 ~ 3.0

各ウェイト
--weight-basic : ベーススコア（RSI, MACD など基本的な要素）

--weight-ema : EMA 関連スコア

--weight-sma : SMA 関連スコア

--weight-bollinger : ボリンジャーバンド関連スコア

--weight-roc : ROC 関連スコア

--weight-adx : ADX 関連スコア（トレンドの強さ）

--weight-stochastics : ストキャスティクス関連スコア

--weight-fibonacci : フィボナッチ関連スコア

--weight-vwap : VWAP 関連スコア

--weight-ichimoku : 一目均衡表関連スコア

例:

bash
コードをコピーする
# トレンドフォロー寄り（ADX と EMA を重くする）
tickwise -t 7203.T --adx --ema --weight-adx 2.0 --weight-ema 1.5

# ボラティリティブレイク狙い（ボリンジャー強め）
tickwise -t 6920.T --bollinger --weight-bollinger 2.5
6. スタンス（視点）の指定
--stance <buyer|seller|holder>
評価の視点を指定します。

型: String

許可値:

buyer : 買いを検討している人の視点

seller : 売り／利益確定を検討している人の視点

holder : すでに保有していてホールド継続を検討する視点（既定）

既定: holder

このスタンスによって、同じ指標でも「評価コメント」や「注意ポイント」のニュアンスが変わります。

7. ログ出力・フォーマット関連
--save-technical-log
テクニカル分析結果をログ（CSV または JSON）として保存します。

型: bool

既定: false

--log-format <csv|json>
テクニカルログの形式を指定します。

型: String

既定: csv

許可値: csv, json

--log-dir <DIR>
ログを保存するディレクトリを指定します。

型: String

既定: log

--data-append
既存の CSV ファイルに追記していきます。

型: bool

既定: false

無効の場合:

毎回新しいファイルを作成します。

--log-flat
銘柄ごとのサブディレクトリを作らず、log-dir 直下にログを保存します。

型: bool

既定: false

有効の場合:

すべての銘柄のログが同じディレクトリにまとまります（バッチ処理などで便利）。

--stdout-log
ログ（CSV / JSON）をファイルではなく標準出力に出します。

型: bool

既定: false

用途:

シェルスクリプトや他ツールとのパイプ連携

例:

bash
コードをコピーする
tickwise -t 7203.T --save-technical-log --stdout-log --log-format json \
  | jq '.'
--show-log-header
現在のオプション設定に基づく CSV ヘッダ行のみ を出力し、終了します。

型: bool

既定: false

特記事項:

このオプションのみを使う場合は --ticker は不要です。

CSV を外部ツールで扱う前に、カラム名を確認したいときに便利です。

8. 出力メッセージ長さの調整
以下は、LLM を使うテキスト出力の最大文字数を調整するオプションです。

型: すべて usize

各項目
--max-note-length

「注意ポイント」の最大文字数

既定: 300

--max-shortterm-length

「1週間短期目線」の最大文字数

既定: 150

--max-midterm-length

「1ヶ月中期目線」の最大文字数

既定: 150

--max-news-length

「ニュースハイライト」の最大文字数

既定: 600

--max-review-length

「総評」の最大文字数

既定: 1000

ターミナル幅や、自分が読みやすい分量に合わせて調整してください。

9. デバッグ・その他
--alias-csv <PATH>
銘柄名のエイリアス定義を含む CSV を指定します。

型: String

用途:

ニュース検索等で、ティッカー以外の正式名称・別名を補助的に使う場合。

-a, --no-alias
エイリアス展開をスキップし、ティッカー／正式名称のみでニュース検索します。

型: bool

既定: false

--silent
通常の出力をすべて抑制し、エラーのみ表示します。

型: bool

既定: false

用途:

バッチ連携や cron からの実行時に、ログファイルだけ残したい場合。

--debug-args
パースされたコマンドライン引数の内容を表示します（デバッグ用）。

型: bool

既定: false

Tickwise にどの値が渡っているか確認したいときに使います。

10. まとめ：よくある組み合わせ例
bash
コードをコピーする
# 1. テクニカル＋ニュース＋LLM（標準的な使い方）
tickwise -t 7203.T --ema --sma --bollinger --vwap --stance holder --show-news

# 2. テクニカルのみ・ログ取得に特化（バッチ処理）
tickwise -t AAPL --no-llm --no-news --save-technical-log --log-format csv \
  --log-dir logs --data-append --silent

# 3. 逆張りスイング寄り（MACD マイナス許容、RSI 厳しめ）
tickwise -t 6920.T --macd-minus-ok --buy-rsi 25 --sell-rsi 75 \
  --bollinger --bb-bandwidth-squeeze-pct 6.0 --stance buyer

# 4. LLM プロンプトの検証（API 叩かずに中身だけ確認）
tickwise -t MSFT --debug-prompt --no-llm