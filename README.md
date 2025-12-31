# tickwise
# 📝 Tickwise — 投資家向け・Rust 製 CLI テクニカル / ニュース分析ツール

Tickwise は、株価終値データとニュースを組み合わせて
「いま、この銘柄を買うべきか？」を 1 本の CLI で素早く判断できる分析ツールです。

Rust 製（高速・安全）

unwrap 禁止ポリシーで実装（雑なクラッシュ防止）

API キーは tickwise.env に完全分離

テクニカル指標＋ニュース評価を統合

オプションは 56 種類以上
→ 組み合わせは実質“無限”レベル（数百万通りの戦略プリセットが構築可能）

# 1. これは誰のためのツールか？
# 🎯 投資家向け

MACD / RSI / ボリンジャーバンド / 一目均衡表 / VWAP / ADX / ROC / フィボナッチ / ストキャス など
→ 多数指標を買い／売り／中立にスコア化

Brave News + OpenAI によるニュース要約

「買う確率」を 0〜100% のバーとして表示

# 👨‍💻 開発者向け

Rust（stable）＋ Clap CLI

unwrap / expect 不使用

Config と環境変数で設定を一元管理

CSV / JSON ログ出力を標準サポート

📚 言語学習・プロンプト設計の教材として

すべてのアウトプットが日本語で構造化

LLM に食わせると素材として再利用しやすい

# 2. 数百万通りの組み合わせで“自分の戦略”を作れる

Tickwise の本質は “戦略レシピを自由に構築できること”。

たとえば：

トレンドフォロー（順張り）

逆張りの強弱調整

ボリンジャー・スクイーズ戦略

サポレジ（フィボナッチ）重視型

EMA/SMA のクロス戦略

ニュース主導型センチメント補正

Weight（重み）を使った“独自配点システム”

56 のオプション × 多数の Weight
→ 実質的には数百万通りの戦略構成が可能。

以下はその中の “逆張り構成の 1 例” のみ紹介です。

#  使用例：逆張り強めの構成（ソフトバンク G）

ここでは、逆張りレシピの“わかりやすいサンプル”として
ソフトバンクグループ（9984.T）を例に説明します。

📝 市場・銘柄状況メモ（2025/12/30 時点）

大納会前で市場はやや調整ムード

ソフトバンク G は約 1 ヵ月下落しており、反発の兆し

RSI / MACD は下げ止まり気配

ちょうど AI 関連の買収ニュース（DigitalBridge）あり

1→4 の株式分割も予定されていた時期
→ 材料は強いが、株価は押し込まれている局面

🔧 このサンプルで使った“逆張り系キーフラグ”

macd-minus-ok

buy-rsi 20

macd-diff-low 1

bollinger

bb-bandwidth-squeeze-pct 10

stance buyer

show-news

# 🖥 実行コマンド（※コードブロックを使わずプレーン表示）

./tickwise -t 9984.T
--macd-minus-ok
--buy-rsi 20
--macd-diff-low 1
--bollinger
--bb-bandwidth-squeeze-pct 10
--stance buyer
--show-news

👉 フル出力はこちら
[softbank-reversal-20251230.pdf](samples/softbank-reversal-20251230.pdf)
# 3. 「逆張り設定とは？」（レシピ解説）

Tickwise には、逆張り時に便利な“3 つの起点”があります。

■ 1. macd-minus-ok

MACD がマイナス圏でも
シグナル上抜けなら「買いを許容」。

→ 典型的な“落ちるナイフ底打ち”検出ロジック。

■ 2. buy-rsi <閾値>

例：20
RSI ≤ 20 → 強い売られすぎ。

→ “投げ売りゾーン”の基準をユーザーが調整可能。

■ 3. macd-diff-low <値>

シグナルとの乖離が小さいときに
判定を中立へ補正して過剰反応を防止。

# 4. インストール

git clone https://github.com/xxx/tickwise.git

cd tickwise
cargo build --release

# 5. tickwise.env の例

OPENAI_API_KEY=sk-xxxxx
BRAVE_API_KEY=brv-xxxxx
NEWS_FILTER=true
NEWS_COUNT=20

# 6. 主なコマンドラインオプション（一覧）

👉 詳細版は docs/command-reference.md

# 7. レシピ（戦略プリセット集）

👉 docs/recipes/README.md

逆張りスイング

順張りトレンドフォロー

ニュース主導型

決算直前チェック

ボラブレイク戦略 など

# 8. セキュリティと非機能要件

API キーは tickwise.env に隔離

unwrap / expect 不使用

SBOM（CycloneDX）生成

VirusTotal 誤検知チェックの導線あり

CSV / JSON ログ

deterministic（再現性重視）設計

# 9. Git 運用メモ

main / feature ブランチ

リリース zip は Binary/ へ

VT 誤検知時の申請手順 → docs/vt-guide.md

Mac / Windows（MSVC）両対応

# 10. ライセンス

MIT License

# 11. お問い合わせ

Pull Request / Issue 歓迎