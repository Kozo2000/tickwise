# 📘 Tickwise セットアップガイド（v3.2）

このドキュメントは、Tickwise を 初めて実行するためのセットアップ専用ガイド です。
README とは役割を分け、「動かすところまで」 に集中しています。

ネットワーク環境や実行方法（ダブルクリック／ショートカット／タスク実行等）により
挙動が変わる場合があるため、その前提で記述しています。

# 1. はじめに

Tickwise を実行するために必要な準備は次の 3 点です。

東証の銘柄一覧（Excel → CSV）の準備

API キー（OpenAI / Brave）の取得

tickwise.env の作成

所要時間の目安：5〜15 分

# 2. 東証の銘柄一覧（Excel → CSV）を準備する

（迷わない最短ルート）

Tickwise は日本株の銘柄コード正規化のために、
東証が公開している「上場銘柄一覧（Excel）」 を CSV に変換して使用します。

2-1. 銘柄一覧が置いてある公式ページ（直リンク）

以下の URL を開いてください：

https://www.jpx.co.jp/markets/statistics-equities/misc/01.html

このページに 「上場銘柄一覧」 の Excel ファイルがあります。

※ 東証のサイト構成変更により URL が変わる可能性があります。

2-2. ダウンロードするファイルの目印

Excel 形式（.xls / .xlsx）

ファイル名は時期により異なります

data_j.xls

data_j.xlsx

meigara_list.xlsx

など

Excel のリンクであれば基本的に問題ありません。

2-3. CSV（UTF-8）に変換する

Excel または互換ソフトでファイルを開き、
「名前を付けて保存 → CSV UTF-8（カンマ区切り）」 を選択します。

注意点：

必ず CSV UTF-8 を選択（SJIS CSV は文字化けします）

列やヘッダは変更しない

保存後、拡張子が .csv になっていることを確認

2-4. 保存場所の例
tickwise_data/tse_list.csv


このパスは後で設定します。

# 3. API キーの取得
3-1. OpenAI API キー（ニュース要約に使用）

https://platform.openai.com/

「API Keys」→「Create new secret key」

sk-xxxx の形式で発行されます

※ OpenAI の料金体系に従います。

3-2. Brave News API キー（ニュース取得に使用）

https://search.brave.com/api/

ログイン後「Create API Key」

brv-xxxx の形式

※ ニュース機能を使わない場合は不要です。

3-3. Yahoo Finance（株価取得）

Tickwise は Yahoo Finance v7（非公式） を利用します。

API キー不要

通信環境によって取得に失敗する場合があります

正確性・遅延・欠損は保証されません


# 4. tickwise.env の作成（sample を使う方法・推奨）

Tickwise では設定ファイルとして
tickwise.env を 1 つだけ使用します。

GitHub には、すべての設定項目を含んだ雛形として
tickwise.env.sample が用意されています。

👉 この sample をコピーして使うのが最も安全で確実です。

4-1. tickwise.env.sample をコピーする

リポジトリ直下にある tickwise.env.sample をコピーし、
ファイル名を tickwise.env（拡張子なし） に変更します。

tickwise.env.sample  →  tickwise.env


※ .env や .txt にならないよう注意してください。

4-2. セットアップ時に「必ず編集する項目」

最初に編集が必要なのは API キーだけ です。

#===== API_KEY =====
OPENAI_API_KEY=sk-xxxxx
BRAVE_API_KEY=brv-xxxxx


OpenAI：LLM 要約を使う場合に必要

Brave：ニュース取得を使う場合に必要

※ どちらも使わない場合は空欄のままでも動作します
（--no-llm / --no-news などで制御可能）

4-3. セットアップ段階では「触らなくてよい項目」

以下の項目は 初期値のままで問題ありません。

テクニカル閾値
BUY_RSI=30.0
SELL_RSI=70.0
MACD_DIFF_LOW=2.0
MACD_DIFF_MID=10.0

テクニカル指標の有効 / 無効
EMA=True
SMA=True
FIBONACCI=True
STOCHASTICS=True
ADX=True
ROC=True
BOLLINGER=True
VWAP=True
ICHIMOKU=True

Weight（配点）
WEIGHT_BASIC=2.0
WEIGHT_EMA=1.0
...


これらは 戦略設計用の項目です。
セットアップ確認や初回実行では触る必要はありません。

4-4. OpenAI / ニュース設定について
OPENAI_MODEL=gpt-5
NO_NEWS=false
NO_OPENAI=false


これらは CLI オプションで上書き可能

セットアップ時点では sample のままで OK

--no-llm / --no-news を使えば env 側を触らずに制御可能

4-5. ログ関連設定（後回しでOK）
SAVE_TECHNICAL_LOG=false
LOG_FORMAT=json
LOG_DIR=log
CSV_APPEND=false


ログを保存したいときに調整する項目

セットアップ確認では false のままで問題なし

※ LOG_DIR は 実行フォルダ依存になるため、
必要に応じて 環境変数で上書きするのが推奨です。

4-6. エイリアス CSV について
ALIAS_CSV=data_j.csv


ティッカーに会社名を対応付けるための CSV

未設定 / ファイルなしでも Tickwise は動作します

初期セットアップでは 深追い不要

4-7. まとめ：セットアップ時の最小作業

セットアップ時にやることは、実質これだけです：

tickwise.env.sample をコピーして tickwise.env を作る

API キー（必要なものだけ）を貼り付ける

他の項目は 一切触らない

戦略調整・weight 設計・ログ制御は
動作確認後に行うものです。

補足：設定の優先順位（再掲）

Tickwise は以下の順で設定を解決します。

環境変数（主にパス・ログ位置）

tickwise.env

デフォルト値

API キーは tickwise.env で管理することを推奨します。

# 5. 初回動作チェック（セットアップ確認用テスト）

このセクションは、Tickwise 本体が正常に起動し、
テクニカル指標の計算まで到達できるかを確認するためのテストです。

通常の分析用途ではありません。

5-1. テスト用の最小コマンド

以下は セットアップ確認専用のテストコマンド です。

tickwise.exe -t 7203.T --no-llm --no-news


このテストでは：

株価データのみ取得（Yahoo Finance / API キー不要）

ニュース取得なし

LLM 要約なし

という構成で、
外部依存を最小限にした状態で動作確認を行います。

5-2. 確認できること

Tickwise が起動する

コマンドライン引数が解釈される

株価データが取得できる

テクニカル指標の計算が最後まで実行される

5-3. 成功時の目安

以下のような出力が表示されれば、
セットアップは概ね問題ありません。

ティッカー：7203.T

株価終値データ

RSI / MACD / ボリンジャーバンドなどの指標値

5-4. 注意

このコマンドは：

セットアップ確認用

トラブルシューティング用

として用意しています。

通常の分析では使用し続けるものではありません。

# 6. 応用：実行フォルダに依存しない設定（推奨）

ショートカット起動、タスク実行、CI などでは、
実行フォルダ依存でファイル参照が失敗することがあります。

その場合、パス情報のみを環境変数で指定すると安定します。

6-1. 環境変数で指定するのに適した項目

銘柄 CSV ファイルのパス

ログ出力ディレクトリ

Tickwise のベースディレクトリ

API キーは 環境変数に設定せず、tickwise.env に置くことを推奨します。

6-2. Windows（PowerShell）
$env:TICKWISE_BASE_DIR="C:\tickwise"
$env:TSE_LIST_FILE="C:\tickwise\data\tse_list.csv"
$env:TICKWISE_LOG_DIR="C:\tickwise\logs"



6-3. Windows（cmd.exe）
set TICKWISE_BASE_DIR=C:\tickwise
set TSE_LIST_FILE=C:\tickwise\data\tse_list.csv
set TICKWISE_LOG_DIR=C:\tickwise\logs




6-4. macOS / Linux（bash / zsh）
export TICKWISE_BASE_DIR="$HOME/tickwise"
export TSE_LIST_FILE="$HOME/tickwise/data/tse_list.csv"
export TICKWISE_LOG_DIR="$HOME/tickwise/logs"


6-5. 設定の優先順位

環境変数

tickwise.env

デフォルト値

# 7. トラブルシューティング（抜粋）

CSV が文字化けする
→ CSV UTF-8 で保存されているか確認

tickwise.env が読まれない
→ ファイル名・改行・空白を確認

企業ネットワークで株価取得できない
→ Web フィルタ／プロキシの影響の可能性あり