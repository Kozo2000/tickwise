# tickwise
# 📝 Tickwise — 投資家向け・Rust 製 CLI テクニカル / ニュース　株価分析ツール

Tickwise は、株価終値データとニュースを組み合わせて
「いま、この銘柄を買うべきか？」を 1 本の CLI で素早く判断するための分析ツールです。

Rust 製（高速・安全）

unwrap 禁止ポリシー（雑なクラッシュ防止）

API キーは tickwise.env で完全分離

テクニカル指標＋ニュース評価を統合

オプションは 56 種類以上
→ 組み合わせは実質 “無限”（数百万通りの戦略プリセットが構築可能）

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

LLM に投入しやすい「定型フォーマットのテキスト」を生成

# 2. なぜ Tickwise が必要なのか（LLM × 投資の本質問題）

ChatGPT を含むほぼすべての LLM には、投資判断における致命的な弱点があります。

曖昧な質問文の書き方で回答が揺れる

その日のニュースの偏りでスタンスが変わる

MACD や RSI の“値そのもの”を自動計算しない

感情論・言い回し依存の回答になりやすい

つまり、
「ソフトバンクはそろそろ反発しますか？」
という曖昧質問を LLM に投げると、
“理由の薄い助言” や “矛盾した文章” が生成されやすい。

ここを矯正するために Tickwise が存在します。

# 3. Tickwise が解決する「曖昧性」の問題

投資判断には 3 つの揺れ（Variance）が必ず存在します：

質問文の揺れ

ニュースの揺れ

テクニカル指標の揺れ（LLM が計算しないことによる）

Tickwise はこれらを 定量化と標準化で“ゼロに寄せる” ために設計されています。

# 4. 素の LLM と Tickwise の決定的な違い
❌ 素の LLM に質問すると…
ソフトバンクはそろそろ反発しますか？
どの値で買えばいいですか？


主観・感情論に寄りやすい

ニュース偏重になり、テクニカルが無視される

数値がないため論理の土台が揺れる

✅ Tickwise → LLM の入力

Tickwise は LLM に 確定した事実（Fact）だけ を渡す。

RSI 50.18

MACD -102.18 / Signal -155.37

ボリンジャーバンド帯幅 18.3%

%b = 0.49

ニュースの一次性・重要度分類

総合スコア 4.0（買い 83%）

これにより LLM は
“感覚”ではなく 事実ベースの要約 を返す。

# 5. Tickwise の役割：LLM の“補助脳”としての定量化エンジン

Tickwise の本質は、
「LLM に考えさせるための入力を整えるツール」 です。

指標計算は 1 回だけ 実行（ブレない）

技術指標は 可能な限りライブラリ優先

サニタイズ（無効値の除去、境界値の保護）

ニュースは一次性・材料性で整理

解析結果を 定型フォーマット に整形

LLM が扱いやすい構造化テキストを生成

これにより LLM のブレ（Variance）が大幅に削減される



# 6. 数百万通りの組み合わせで“自分の戦略”を作れる

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
# 7. 「逆張り設定とは？」（レシピ解説）

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

# 8. インストール

✅ A. 一般ユーザー（推奨） — Windows バイナリを使うだけ

Binary/tickwise_win_v1.x.x.zip を解凍し、
中の tickwise.exe を実行すれば完了。

Rust のインストールは不要。

✅ B. 開発者向け（Rust でコンパイルする場合のみ）
git clone https://github.com/xxx/tickwise.git
cd tickwise
cargo build --release


target/release/tickwise が生成されます。

# 9. tickwise.env の例

OPENAI_API_KEY=sk-xxxxx

BRAVE_API_KEY=brv-xxxxx

NEWS_FILTER=true

NEWS_COUNT=20

# 10. 主なコマンドラインオプション（一覧）

👉 詳細版は docs/command-reference.md

# 11. レシピ（戦略プリセット集）

👉 docs/recipes/README.md

逆張りスイング

順張りトレンドフォロー

ニュース主導型

決算直前チェック

ボラブレイク戦略 など

# 12. セキュリティと非機能要件

✔ 安全な構造体アクセス（Config / Guard / Result の SoT 設計）

すべての設定・指標値は 単一の SoT（Config）から参照

ロジック内での多重定義・再計算を禁止

指標計算は 1 回のみ で固定化

ライブラリに存在する指標は 必ずライブラリを優先

不正値・欠損値などは 事前にサニタイズ

✔ Crash しない Rust 実装

unwrap / expect 禁止

Option / Result で全分岐処理

ロギングを強化（CSV / JSON）

✔ ソフトウェア供給網（SBOM）

CycloneDX 形式の SBOM を生成

依存パッケージの透明化

✔ VirusTotal（ウイルス対策ベンダー）

初回スキャンで誤検知 →
該当ベンダーへ連絡し誤検知認定取得済み（実施済み）

✔ 再現性のある出力（deterministic）

テクニカル指標は固定ロジック

LLM の揺れは Tickwise の定量データで最小化

# 13. Git 運用メモ

main / feature ブランチ

リリース zip は Binary/ へ

VT 誤検知時の申請手順 → docs/vt-guide.md

Mac / Windows（MSVC）両対応

# 14. ライセンス

MIT License

# 15. お問い合わせ

Pull Request / Issue 歓迎