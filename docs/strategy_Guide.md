# Tickwise Strategy Guide  
（戦略設計・レシピ・検証ガイド）

---

## 1. このドキュメントの目的

このドキュメントは、Tickwise を **どのような考え方で使うべきか** を説明します。

- 戦略（レシピ）の設計方法
- 分析結果の検証・再利用の方法
# Tickwise Strategy Guide (v1.1)
**— 鋼のロジックを、10のレシピで使い倒す —**

---

## ２. 設定の優先順位と `-I` オプション

Tickwise v1.1 では、検証の効率化のために以下の優先順位を徹底しています。

1. **コマンドオプション**（最優先）
2. **tickwise.env**（ファイル設定）
3. **デフォルト値**（コード内）

### 💡 レシピ検証の極意：`-I` オプション
`.env` の中身をいちいち書き換える必要はありません。**`-I` (Ignore) を付与すれば、すべての指標設定がリセットされ、コマンドラインで指定した「レシピ」のみが純粋に実行されます。** 以下のレシピはすべて `-I` 併用を前提としています。

---

## 3. 実戦汎用レシピ 10選

### ① 【順張り】王道のトレンドフォロー
上昇トレンドの波に乗り、勢いが衰えていないかを確認する最も標準的な設定です。
```bash
tickwise -t <TICKER> -I --stance buyer --ema --sma --adx --buy-rsi 40 --weight-ema 2.0 --weight-sma 2.0 --weight-adx 2.5
```
- **論理**: 複数の移動平均のクロスを主軸にし、ADXでトレンドの「本気度」を判定。RSIは押し目を拾うために少し高めの40に設定。

### ② 【順張り】一目均衡表・雲の上の戦い
中長期のトレンドが「厚いサポート」に支えられているかを重視します。
```bash
tickwise -t <TICKER> -I --stance holder --ichimoku --sma --weight-ichimoku 3.0 --weight-sma 1.0
```
- **論理**: 一目均衡表の転換線・基準線の関係を最重視（Weight 3.0）。雲の上にいる間の「安心感」を数値化します。

### ③ 【順張り】ボラティリティ・ブレイクアウト
ボリンジャーバンドが収束（スクイーズ）し、パワーを貯めた後の爆発を狙います。
```bash
tickwise -t <TICKER> -I --bollinger --bb-bandwidth-squeeze-pct 8 --weight-bollinger 3.0 --adx --weight-adx 2.0
```
- **論理**: バンド幅が8%以下の「嵐の前の静けさ」を検知。放たれた瞬間のADXの立ち上がりを評価します。

### ④ 【順張り】短期決戦・モメンタム重視
価格の変化率（ROC）を主役に据え、短期間で一気に駆け上がる銘柄を追います。
```bash
tickwise -t <TICKER> -I --roc --weight-roc 3.0 --ema --weight-ema 1.0 --news-freshness pd
```
- **論理**: 10日間のROCが急騰していることを重視。ニュースは `pd`（過去24時間）に絞り、今の熱量を測ります。

### ⑤ 【逆張り】売られ過ぎ反発（基本形）
教科書的な「売られ過ぎ」からのリバウンドを論理的に拾います。
```bash
tickwise -t <TICKER> -I --stance buyer --buy-rsi 30 --macd-minus-ok --macd-diff-low 1.0 --weight-basic 2.5
```
- **論理**: RSI 30以下を狙いつつ、`macd-minus-ok` でマイナス圏での反転を許容。`diff-low` を絞ることで下げ止まりを確認します。

### ⑥ 【逆張り】パニック売り・底値拾い
市場が総悲観になり、セリングクライマックスに達した瞬間を狙う過激な設定です。
```bash
tickwise -t <TICKER> -I --stance buyer --buy-rsi 20 --bollinger --weight-bollinger 2.5 --macd-minus-ok --weight-basic 1.5
```
- **論理**: RSI 20という極限状態を設定。ボリンジャーバンドの下限（-2σ）を突き抜けた後の戻りを目指します。

### ⑦ 【逆張り】VWAP回帰（平均回帰戦略）
適正価格（VWAP）から大きく乖離した株価が、磁石のように引き戻される性質を利用します。
```bash
tickwise -t <TICKER> -I --vwap --weight-vwap 3.0 --buy-rsi 35 --stance buyer
```
- **論理**: VWAPと現在値の乖離をスコアの最大根拠にします。行き過ぎたゴムが戻る瞬間を捉えます。

### ⑧ 【防御】ホルダーの警戒モード
保有株の「逃げどき」を探る、ディフェンス重視の設定です。
```bash
tickwise -t <TICKER> -I --stance holder --sell-rsi 65 --macd-diff-mid 5 --bollinger --weight-bollinger 2.0
```
- **論理**: RSIの売り閾値を少し下げて早めに警告。バンド上限での過熱感を厳しめに評価させます。

### ⑨ 【全方位】AIセカンドオピニオン
全56指標をフル稼働させ、AIに「指標間の不整合」を炙り出させます。
```bash
tickwise -t <TICKER> -I --ema --sma --bollinger --fibonacci --adx --ichimoku --vwap --llm
```
- **論理**: 人間では見きれない多角的データをAIにぶつけ、総合的な「穴」を指摘させるためのレーダーチャート設定。

### ⑩ 【情報戦】材料・テーマ深掘り
テクニカルはベースに留め、ニュースの「中身」をAIに徹底的に読み込ませます。
```bash
tickwise -t <TICKER> -I --llm --news -q "業績予想 修正 増配" --weight-news 3.0 --news-count 20
```
- **論理**: カスタムクエリ `-q` で材料を絞り込み、AIの総評を「情報先行」の投資判断に活用します。

---

## 4. 最後に
> **ここに挙げたのは使い方の一例です。自由にカスタマイズしていただき、自分に合った分析パラメータ作りにチャンレンジしてみてください。**

**Stock Technical 'AI' Analysis Tool — Created & Designed by Kozo2000**