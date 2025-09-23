## 🔍 Why tickwise uses Rust (and not C++)

tickwise is a stock analysis CLI tool designed with security, robustness, and clarity in mind.  
Here's why Rust was chosen over C++ — from functionality to safety.

---

### 🧩 Feature Implementation

| Item                | Rust                            | C++                               |
|---------------------|----------------------------------|------------------------------------|
| CLI argument parsing | `clap` (simple, type-safe)      | `getopt`, Boost.ProgramOptions (verbose) |
| JSON handling       | `serde_json` (auto serialization) | RapidJSON, nlohmann/json (manual types) |
| API communication   | `reqwest` (async ready)          | `libcurl`, Boost.Beast (complex)  |
| String formatting   | `format!`, `println!` (easy)     | `std::ostringstream` (cumbersome) |

✅ **Rust wins in developer speed and correctness.**

---

### 🛡 Security and Safety

| Item                    | Rust                                  | C++                                    |
|-------------------------|----------------------------------------|-----------------------------------------|
| Bounds checking         | `Vec[i]` panics, `get(i)` returns `Option` | `vec[i]` may cause undefined behavior  |
| NULL pointer risks      | `Option<T>` enforces handling          | `nullptr` is common, often misused     |
| Memory management       | Ownership & borrow checking (compiler) | Manual (`new`, `delete`, `shared_ptr`) |
| Thread safety           | `Send` / `Sync` trait enforced         | Manual, prone to data races            |

✅ **Rust eliminates entire classes of memory vulnerabilities at compile time.**

---

### 🛠 Design and Maintainability

| Item                | Rust                              | C++                               |
|---------------------|------------------------------------|------------------------------------|
| Type handling       | Strong inference + boundary enforcement | Implicit conversions = bugs       |
| Error handling      | `Result`, `Option` enforced        | Exceptions can be ignored         |
| Ownership model     | Built-in, enforced by compiler     | No native ownership, must track manually |
| Predictable behavior| Panics or rejections by design     | Frequent undefined behavior        |

✅ **Rust enforces correctness by design.**

---

### 🧠 Developer Burden

| Item                | Rust                          | C++                            |
|---------------------|-------------------------------|--------------------------------|
| Code verbosity      | Slightly verbose in types      | Often shorter, but ambiguous   |
| Learning curve      | Steep at first, then smooth    | Familiar, but full of pitfalls |
| Debugging           | Compiler feedback is strong    | Silent failures are common     |

✅ **Rust: Harder to learn, easier to trust.**

---

### ✅ Conclusion

> If tickwise had been written in C++,  
> it might have compiled faster, but it wouldn't be **safe to ship**.

Rust makes tickwise:
- Safer by default
- Easier to extend and maintain
- Trustworthy for sensitive financial data

That’s why tickwise is built with **Rust**, not C++.

## 🔍 なぜ tickwise は Rust で作られたのか（そして C++ ではないのか）

tickwise は、**セキュリティ・堅牢性・明快な設計**を重視した株価分析CLIツールです。  
ここでは Rust を C++ より選んだ理由を、**機能性・安全性・保守性・開発効率**の観点から比較します。

---

### 🧩 機能実装のしやすさ

| 項目                 | Rust（ラスト）                      | C++（シープラスプラス）                  |
|----------------------|--------------------------------------|------------------------------------------|
| CLI引数の解析         | `clap`（簡単・型安全）               | `getopt` や Boost（記述量多い）           |
| JSON処理             | `serde_json`（直列化が簡単）         | RapidJSON や nlohmann/json（型定義が煩雑） |
| API通信              | `reqwest`（非同期対応）              | `libcurl`, Boost.Beast（学習コスト高）    |
| テキスト整形         | `format!`, `println!`（書きやすい）  | `std::ostringstream`（冗長・誤りやすい）   |

✅ **Rust の方が開発が早く、ミスも少ない。**

---

### 🛡 セキュリティと安全性

| 項目                 | Rust                                  | C++                                    |
|----------------------|----------------------------------------|-----------------------------------------|
| 境界外アクセス         | `Vec[i]` は panic、`get(i)` は Option型     | `vec[i]` は未定義動作の恐れ           |
| NULLポインタの扱い     | `Option<T>` で存在チェックを強制される     | `nullptr` は普通に使われ、誤用しやすい |
| メモリ管理             | 所有権・借用によるコンパイル時検査        | `new/delete`, `shared_ptr` が必要    |
| スレッドの安全性        | `Send` / `Sync` トレイトで静的に検査      | 自己責任。データ競合の可能性あり       |

✅ **Rustは、C++では起きるバグの“原因そのもの”を排除できる。**

---

### 🛠 設計と保守性

| 項目                 | Rust                             | C++                                     |
|----------------------|----------------------------------|------------------------------------------|
| 型の扱い              | 型推論＋厳格な制約（Option/Result） | 暗黙キャスト多く、設計ミスに繋がる        |
| エラー処理            | `Result`, `Option` で強制対応       | 例外（`throw`）は無視されがち             |
| 所有権・借用モデル     | ビルトインで安全性を担保             | 明示的に追跡しなければならない            |
| コードの予測可能性     | コンパイル時 or 明確な panic         | 未定義動作が多く、挙動が不明確になりやすい |

✅ **Rustは「壊れない設計」がしやすい。**

---

### 🧠 プログラマの負担

| 項目                | Rust                           | C++                                   |
|---------------------|----------------------------------|----------------------------------------|
| 記述量              | 型注釈がやや多くなることも        | STLを使えば短いが、挙動があいまい       |
| 学習コスト          | 最初は高いが、習得すれば安心        | 親しみやすいが、罠が多い                |
| デバッグ体験        | コンパイラが強力に指摘してくれる     | サイレントエラーが混ざりやすい          |

✅ **Rust：学習は大変だが、信頼できる。  
C++：学習は楽だが、運用が怖い。**

---

### ✅ 結論

> もし tickwise を C++ で書いていたら、  
> コンパイルは通っても「人に渡すのが怖い」ツールになっていたかもしれません。

Rustで作った tickwise は：

- **最初から安全**
- **拡張・保守が簡単**
- **金融データを扱っても信頼できる**

だから tickwise は **C++ ではなく Rust** で作られています。
