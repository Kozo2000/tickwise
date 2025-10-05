Tickwise Windows セットアップガイド
──────────────────────

前提条件
・Windows 10 / 11（64bit）
・インターネット接続
・Excel（東証データのCSV変換に使用）

──────────────────────

ファイルの準備

tickwise.exe と tickwise.env.sample をダウンロード。

任意のフォルダ（例：C:\Users\<ユーザー名>\Documents\tickwise）に保存。

フォルダ例：
C:\Users\<ユーザー名>\Documents\tickwise\
├─ tickwise.exe
└─ tickwise.env.sample

──────────────────────
2. 東証の銘柄コードファイルを入手
以下のURLから東証公式の銘柄一覧（XLS）をダウンロード：
https://www.jpx.co.jp/markets/statistics-equities/misc/tvdivq0000001vg2-att/data_j.xls

──────────────────────
3. XLS → CSV に変換（Excel）

data_j.xls を Excel で開く。

「名前を付けて保存」→ ファイルの種類を「CSV（カンマ区切り）」に選択。

ファイル名を tse_codes.csv として保存。

文字コードは UTF-8 にしておくと安全。

tickwise.exe と同じフォルダに置くと管理が簡単。

──────────────────────
4. Brave APIキーを取得

Brave Search API のページへアクセス：
https://api.search.brave.com/

Sign Up / Log In でアカウント作成。

Subscriptions → API Keys → Add API Key。

生成されたキーをコピーして控える。

参考ドキュメント（英語）：
https://api.search.brave.com/app/documentation

──────────────────────
5. OpenAI APIキーを取得

OpenAI公式ページへアクセス：
https://platform.openai.com/

ログイン後、右上メニュー「View API keys」へ。

「Create new secret key」をクリック。

生成されたキーをコピーして控える。

参考ドキュメント（英語）：
https://platform.openai.com/docs/guides/manage-api-keys

──────────────────────
6. tickwise.env を作成

tickwise.env.sample をコピーして tickwise.env にリネーム。

メモ帳などで開いて APIキーとCSVのパスを記入。

例：
OPENAI_API_KEY=sk-xxxxxxxxxxxxxxxxxxxx
BRAVE_API_KEY=brv-xxxxxxxxxxxxxxxxxxxx

保存先は tickwise.exe と同じフォルダ。

──────────────────────
7. 動作確認
PowerShell または コマンドプロンプトで以下を実行：

cd C:\Apps\tickwise
.\tickwise.exe --help

APIなしでテスト起動：
.\tickwise.exe --ticker 9434.T --no-llm

API有効で起動：
.\tickwise.exe --ticker 9434.T

──────────────────────
8. トラブルシューティング

・tickwise.env が読まれない
→ exeと同じフォルダにあるか確認。拡張子が .txt になっていないか注意。

・CSVが文字化けする
→ Excelで再保存し、文字コードを UTF-8 に指定。

・APIキーエラー
→ .env のキー名と余分な空白や全角記号をチェック。

・通信エラー（社内プロキシ環境など）
→ HTTP_PROXY / HTTPS_PROXY / NO_PROXY を設定。

──────────────────────
9. 出荷版情報（参考）

・バイナリハッシュ（SHA-256）
62C05ED9B23580BE24081C1D03540621B277DC18BBCC0CCB843FFE0B0920A1AB

・誤検知対応
Google：申請済（Safe Browsing）
Ikarus：申請済（fp@ikarus.at
）

──────────────────────
10. 最終フォルダ構成

C:\Users\<ユーザー名>\Documents\tickwise\
 ├─ tickwise.exe
 ├─ tickwise.env
 ├─ tse_codes.csv
 └─ log\

──────────────────────
11. 注意事項

・tickwise.env に含まれるAPIキーは第三者と共有しない。
・GitHubは非公開のままで運用可能。

──────────────────────