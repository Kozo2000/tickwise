# Tickwise Strategy Guide  
（戦略設計・レシピ・検証ガイド）

---

## 1. このドキュメントの目的

このドキュメントは、Tickwise を **どのような考え方で使うべきか** を説明します。

- tickwise.env とコマンドオプションの役割
- 戦略（レシピ）の設計方法
- 分析結果の検証・再利用の方法

> ここでは「何が指定できるか」ではなく  
> **「なぜそう指定するのか」**を扱います。  
> 各オプションの完全な仕様は command-reference.md を参照してください。

---

## 2. Tickwise における「戦略」とは何か

### 2.1 戦略 ≠ 単一指標

Tickwise における戦略とは、

- テクニカル指標
- 閾値
- weight（重み）
- 投資スタンス
- ニュースの扱い方

を **一貫した意図で束ねたもの**です。
複数の要素をどう評価し、どう判断に使うかが戦略です。

---

### 2.2 戦略は「固定ロジック」ではなく「レシピ」

Tickwise は正解の戦略を内蔵しません。

代わりに、

- 指標
- 閾値
- 重み
- スタンス

を組み合わせ、  
**自分の相場観を表現するための道具**を提供します。

---

## 3. 初期値と設定の優先順位

Tickwise では、同じ設定項目に対して  
複数の指定方法が存在します。

評価される優先順位は以下の通りです。

コマンドオプション

↓

tickwise.env

↓

デフォルト値（コード内）

---

### 3.1 コマンドオプション

- 実行時に最優先で評価される
- 一時的な調整・検証向け
- 毎回変わる条件に適している

例：

--buy-rsi 20

--stance buyer

--no-news

---

### 3.2 tickwise.env

- コマンドオプションが指定されていない場合に使用される
- 戦略の「基本思想」を固定する用途
- 繰り返し使う前提の設定

---

### 3.3 デフォルト値

- コマンドオプションも tickwise.env も指定されていない場合に使用
- 動作保証のための最低限の値
- 戦略としての意味は薄い

> デフォルト値は「おすすめ設定」ではありません。  
> 未指定時の安全弁です。

---

## 4. レシピにおける前提（重要）

以下のレシピ例では、  
**tickwise.env は最小構成（ベースライン）**になっていることを前提とします。

- 指標は原則 OFF
- ニュースは OFF
- LLM は OFF
- weight はデフォルト

tickwise.env は  
**戦略を上書きしない安全な下地**として使い、  
戦略の指定はコマンドオプションで明示します。

---

### ベースライン例（考え方）

- 「何もしない」env
- コマンドオプションで意図をすべて読める状態

---

## 5. 戦略レシピ集（実行可能例）


### レシピ実行時の注意（重要）

#### 各種レシピを試す際、 `tickwise.env` に有効なパラメータが残っていると、  その設定がレシピに影響し、**意図した結果が得られない**場合があります。

#### レシピを正しく検証するため,事前に `tickwise.env` を最小構成（ネガティブ設定）にしてから  コマンドオプションで戦略を指定することを推奨します。
---
### レシピ検証用：ネガティブ設定 tickwise.env

### ===== レシピ検証用ベースライン =====
### 各種レシピを試す前に使用する、最小構成の設定です。

#===== API_KEY（必要な場合のみ設定） =====

OPENAI_API_KEY=

BRAVE_API_KEY=

#===== テクニカル閾値（デフォルト） =====

BUY_RSI=30.0

SELL_RSI=70.0

MACD_DIFF_LOW=2.0

MACD_DIFF_MID=10.0

MACD_MINUS_OK=false

#===== 拡張テクニカル分析（すべて無効） =====

EMA=false

SMA=false

FIBONACCI=false

STOCHASTICS=false

ADX=false

ROC=false

BOLLINGER=false

VWAP=false

ICHIMOKU=false

#===== weight（主張しない） =====

WEIGHT_BASIC=1.0

WEIGHT_EMA=1.0

WEIGHT_SMA=1.0

WEIGHT_BOLLINGER=1.0

WEIGHT_ROC=1.0

WEIGHT_ADX=1.0

WEIGHT_STOCHASTICS=1.0

WEIGHT_FIBONACCI=1.0

WEIGHT_VWAP=1.0

WEIGHT_ICHIMOKU=1.0

#===== 投資スタンス =====

STANCE=holder

#===== ニュース・LLM（無効） =====

NO_NEWS=true

SHOW_NEWS=false

NEWS_FILTER=false

NO_LLM=true

#===== ログ設定（必要最低限） =====

SAVE_TECHNICAL_LOG=false

LOG_FORMAT=csv

LOG_DIR=log

CSV_APPEND=false

LOG_FLAT=false

#===== その他 =====

DEBUG_PROMPT=false

ALIAS_CSV=data_j.csv

---

## 5.1 逆張り（Reversal / Mean Reversion）
### 目的

売られ過ぎ局面での反発候補を検討する。

### コマンド例

tickwise -t 9984.T --stance buyer --bollinger --buy-rsi 22 --macd-minus-ok --macd-diff-low 1 --bb-bandwidth-squeeze-pct 10 --weight-basic 2.5 --weight-bollinger 2.0 --weight-adx 0.5

### 数値の考え方（目安）

#### RSI（buy-rsi）

30 前後
- 一般的に「売られ過ぎ」と見なされやすい水準。

30以上に上げる場合

- → 反発初動よりも「戻り途中」まで含めたい
- → シグナルは増えるが、逆張り色は弱くなる

30以下に下げる場合

- → 強い売られ過ぎだけを拾う
- → シグナルは減るが、極端な局面に限定される

#### MACD Diff

0 付近
- 下落の勢いが弱まりつつある状態。

許容値を大きくする場合
- → 下方向の勢いが残る段階も含めたい
- → 早めに拾えるが失敗も増えやすい

許容値を小さくする場合
- → 下げ止まりがより明確になるまで待つ
- → シグナルは遅れるが確度は上がる

#### ADX

25 前後
- トレンドの有無を切り分ける目安。

25以上を許容する場合
- → トレンドが残る局面も逆張り対象に含める
- → 反発幅は大きくなる可能性があるが逆行リスクあり

25未満に制限する場合
- → トレンドが弱い局面に限定
- → 逆張り向きだが機会は減る

## 5.2 順張り（Trend Follow）
### 目的

すでに出ているトレンドの継続に乗る。

### コマンド例

tickwise -t 7203.T --stance buyer --ema --sma --adx --buy-rsi 40 --weight-ema 2.0 --weight-sma 2.0 --weight-adx 2.5

### 数値の考え方（目安）

#### RSI（buy-rsi）

40 前後
-押し目として解釈されやすい水準。

40以上に上げる場合
- → 強い局面だけに限定
- → シグナルは減るが、トレンド色が濃くなる

40以下に下げる場合
- → 浅い押し目も拾う
- → シグナルは増えるが、ダマシも増えやすい

#### ADX

25 前後
- トレンド成立の目安。

25以上に上げる場合
- → 明確なトレンドのみを対象
- → 判断は遅れるが安定しやすい

25以下に下げる場合
- → トレンド初動を含める
- → 早く入れるが不安定になりやすい

## 5.3 ブレイク狙い（Momentum）
### 目的

「動いたものだけ」を対象にする。

### コマンド例
tickwise -t AAPL --stance holder --roc --adx --weight-roc 2.5 --weight-adx 2.0

### 数値の考え方（目安）

#### ROC

0 付近
- 加速の有無を切り分ける境界。

0より大きくなるほど
- → 加速を重視
- → 動き出しを拾いやすいが反動も大きい

0に近づけるほど
- → 加速が明確なものだけに限定
- → 機会は減るが判断は明確

#### ADX

25 前後
- 動いているかどうかの目安。

高くする場合
- → 本格的に動いている銘柄だけ
- → 追随型になる

低くする場合
- → 動き始めも含める
- → 初動狙いになるがノイズも増える

## 5.7 レンジ相場（Range Trading）
### 目的

方向感のない相場で行き過ぎだけを拾う。

### コマンド例

tickwise -t 9433.T --stance holder --stochastics --bollinger --bb-bandwidth-squeeze-pct 12 --weight-stochastics 2.0 --weight-bollinger 1.8 --weight-adx 0.5

### 数値の考え方（目安）

#### ADX

20 前後
-レンジかどうかの判断材料。

20以上を許容する場合
- → レンジ崩れ初期も含める
- → 反転が大きくなる可能性あり

20未満に制限する場合
- → 明確なレンジだけに限定
- → 小さな反発を拾う用途向き

#### Bandwidth

- 指定値前後
- 値動きが収縮している状態。

値を大きくする場合
- → ある程度動き始めても対象
- → レンジ色は弱くなる

値を小さくする場合
- → かなり狭いレンジだけを対象
- → 機会は減るが典型的なレンジになる

## 5.4 価格帯重視（VWAP / Ichimoku）
### 目的

「高いか安いか」を価格帯で整理する。

### コマンド例

tickwise -t 9984.T --stance holder --vwap --ichimoku --weight-vwap 2.0 --weight-ichimoku 2.0

### 数値の考え方（目安）

#### VWAP

現在値 ≒ VWAP
- 価格帯の中心付近。

現在値がVWAPより上
- → 割高側に寄っていると考えやすい

現在値がVWAPより下
- → 割安側に寄っていると考えやすい

#### 一目均衡表（転換線・基準線）

線より上
- → 上側の帯に位置

線より下
- → 下側の帯に位置

位置関係のみを確認し、方向の決め打ちは行わない。


### 5.5 複合レシピの考え方（上級）
- Tickwise のレシピは 排他的ではありません。


例：
- スクイーズ（5.4）↓
- 逆張り（5.1）↓
- ニュース確認（5.8）

という 段階的利用が可能。

#### 重要な考え方
- 1回の実行で完結させない
- レシピは「判断工程」
- Tickwise は 工程を可視化する道具

---

# 6. weight 設計の考え方

### 6.1 weight は「信頼度」
weight は強さではなく、
その戦略でどの指標をどれだけ信じるかを表します。

### 6.2 よくある失敗
すべて同じ weight

すべて高い weight

結果として、

何も主張しない戦略になります。

---

# 7. 戦略を壊さないための注意点
tickwise.env を頻繁に変更しない

コマンドオプションを盛りすぎない

便利そうな指標を全部有効にしない

LLM の出力を最終判断にしない

Tickwise は
判断を代行するツールではありません。

--- 

# 8. 戦略を検証・拡張するための機能
### 8.1 --debug-prompt：プロンプト確認

tickwise -t 7203.T \
  --debug-prompt
LLM に渡すプロンプト全文を出力

API は呼ばれない
他の LLM に手動投入する用途に使える

### 8.2 LLM を使わない運用
tickwise -t 7203.T \
  --no-llm
テクニカル分析のみ実行

LLMの見解が不要。検証・バッチ用途など。

### 8.3 ログ出力（CSV / JSON）
CSV：表計算・比較向け
JSON：プログラム連携向け

8.3.1 JSONログの活用例（パイプ連携）
JSON を標準出力に流すには --stdout-log が必要です。

必要な項目だけ抽出する

tickwise -t 9984.T \
  --no-llm \
  --save-technical-log \
  --stdout-log \
  --log-format json \
  --silent \
| jq .

しきい値判定（例：RSI が 30 以下だけ表示）

tickwise -t 9984.T \
  --no-llm \
  --save-technical-log \
  --stdout-log \
  --log-format json \
  --silent \
| jq 'select(.rsi <= 30)'

複数銘柄を回して JSON Lines として蓄積する

for t in 7203.T 9984.T 9433.T; do
  tickwise -t "$t" \
    --no-llm \
    --save-technical-log \
    --stdout-log \
    --log-format json \
    --silent
done >> tickwise-log.jsonl

---


# 9. まとめ
- 戦略は tickwise.env に隠さない
- 戦略はコマンドオプションで読める形にする
- env は思想、オプションは意図
- ログを残し、戦略を検証する

## Tickwise は
## 判断を出すツールではなく、
## 判断を説明できる状態を作るツールです。