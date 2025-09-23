## 運用メモ（ブランチ/PR/マージ）

- 既定ブランチ: `main`
- 禁止: `main` への直接 push / force-push（常に PR 経由）
- ブランチ命名:
  - 機能追加: `feat/<短い名前>`
  - バグ修正: `fix/<短い名前>`
  - 雑務/整備: `chore/<短い名前>`

### フロー
1) 作業ブランチ作成 `git switch -c feat/<name>`
2) 小さくコミット（`type: 要約` 例: `feat: 5段階ゲージ導入`）
3) プッシュ `git push -u origin feat/<name>`
4) **PR を作成**（タイトル=要約、本文=目的/影響/確認手順）
5) **チェック（必須）**
   - `cargo fmt -- --check`（整形OK）
   - `cargo clippy -D warnings`（警告ゼロ）
   - `cargo build --release`（ビルド成功）
   - 画面出力の体裁/色/幅に破綻なし（必要ならスクショ）
6) **マージ方法**: 緑ボタンの ▾ → **Squash and merge** のみ  
   スカッシュメッセージ: `type: 要約 (#PR番号)`
7) マージ後
   - リモート元ブランチは自動削除（設定済）
   - ローカル: `git branch -d <branch> && git fetch -p`
   - 最新化: `git switch main && git pull`

### レビュー基準（ソロ運用の自己チェック）
- 仕様と出力例に一致（日本語ラベル/ゲージ幅/色）
- 変更は**単一責務・最小差分**（周辺最適化・リネーム禁止）
- 0除算ガード等の退行なし（Buyer/Seller=50%、Holder=0%）
