🔍 Why tickwise uses Rust — author’s note

Context: solo project now, designed to scale to a team later.

Design intent — why we narrowed to C++ & Rust

Separate UI and engine. We built the engine first so the UI can evolve independently.

Native performance. The engine must be fast with zero-overhead abstractions and no VM tax.

No GC pauses. We prefer deterministic latency and explicit lifetime control over runtime garbage collection.

Given these constraints, C++ and Rust were the natural shortlist. Both can meet the performance and control requirements; the remaining question was maintenance and security over time.

We seriously considered C++. It remains a first-class option with a deep ecosystem and outstanding performance, and in capable hands it ships robust systems every day.
Our decision for Rust isn’t about superiority—it’s about fit for a solo workflow today and smooth collaboration tomorrow.

What mattered in this project

Fail early at compile time. Ownership/borrowing and the borrow checker turn aliasing and lifetime mistakes into compile errors, not late tickets. That let a single maintainer stay focused on domain logic instead of chasing memory hazards afterwards.

Type-safe plumbing with minimal glue. clap (CLI), reqwest (HTTP), and serde (JSON) mapped cleanly to tickwise, keeping behavior predictable without heavy scaffolding.

Reproducible releases. Cargo + lockfiles made deterministic builds and hash verification straightforward.

Built to welcome a team later

Modular structure (crates/modules) and typed domain models keep boundaries clear.

Deterministic toolchain with rustfmt/clippy reduces review friction.

Test-first posture that CI can extend with fuzzing and concurrency checks.

Room for FFI if we ever need to bind existing C/C++ libraries.

C++ would also have been viable with disciplined idioms, sanitizers, and thorough reviews. Given a solo maintainer and short release cycles, leaning on Rust’s compile-time guarantees (the borrow checker) aligned better with our risk and cadence.

Bottom line: tickwise uses Rust so one person can ship safely today and a team can build on it tomorrow—not because C++ can’t, but because this stack fits our constraints.

🔍 なぜ tickwise は Rust を選んだか（作者メモ）

※ 現在は個人開発、将来のチーム開発を見据えた設計です。

設計方針 ― C++ と Rust に絞った理由

UI とエンジンの分離。 まずは エンジンを先行開発し、UIは独立に進化できるようにしたい。

ネイティブ性能の確保。 エンジンは高速で、ゼロコスト抽象・非VMが前提。

GC を避ける。 実行時のガベージコレクションではなく、予測可能なレイテンシと明示的なライフタイム管理を重視。

この前提から、候補は自然に C++ と Rust に絞られました。性能と制御の要件はどちらでも満たせるため、残る論点は長期のメンテナンス性と安全性でした。

C++ も真剣に検討しました。エコシステムと性能は第一級で、熟練の現場では高信頼なプロダクトが日々生まれています。
今回 Rust を選んだのは優劣ではなく、個人開発の運用と将来の協調開発に最も合致したためです。

本プロジェクトで効いた点

コンパイル時に失敗してくれる。 所有権／借用と borrow checker により、エイリアシングやライフタイムの不整合を実行前にエラー化。個人開発でも後追いのメモリ不具合に時間を取られにくい。

型安全で配線が少ない。 clap（CLI）・reqwest（HTTP）・serde（JSON）が要件に素直に合い、重いボイラープレートが不要。

再現ビルドが容易。 Cargo＋lockfile でハッシュ検証やツールチェーン固定が簡単。

将来のチーム化に備えた作り

モジュール分割と型付きドメインモデルで責務境界を明確化。

決まったツールチェーン＋rustfmt/clippyでレビュー摩擦を低減。

テスト前提で、将来は CI に fuzzing／並行性チェックを追加しやすい。

FFI 余地を残し、必要なら C/C++ 資産とも連携可能。

C++ でも、作法・サニタイザ・レビューを徹底すれば成立します。
ただ、個人開発 × 短いリリースサイクルという前提では、borrow checker によるコンパイル時保証に寄りかかる方が、リスクと手戻りを抑えられると判断しました。

結論： tickwise は 今日ひとりで安全に出せて、明日チームで拡張しやすいことを軸に Rust を選択しました。これは“C++を否定するため”ではなく、本プロジェクトの制約に最も適した選択です。