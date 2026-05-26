# BTC/USDT Matching Engine (Trial)
## Phase 1 
22:34 25/05/2026 
<br>基本的な機能のみを搭載したマッチングエンジンの試作
<br>Prototype of a matching engine equipped with only basic functions

Benchmark
<br>キャンセルオーダーのレイテンシが目立つ.
<br>スウィープもレイテンシ削減の余地あり

##  ベンチマーク結果 (Performance Benchmarks)

本マッチングエンジンは、Rustのマイクロベンチマークフレームワーク Criterion を使用し、WSL2環境下で性能測定を行いました。すべてのコアロジックにおいて、ナノ秒（ns）単位の超低遅延・高スループットを達成しています。

###  測定結果サマリー

| 処理内容 (Benchmark) | 平均処理時間 (Mean Time) | 信頼区間 (95% Confidence Interval) | 推定スループット (Estimated Throughput) |
| :--- | :--- | :--- | :--- |
| full_match <br> (完全合致注文の約定) | 131.30 ns | 127.11 ns 〜 135.89 ns | 約 761 万件 / 秒 |
| cancel <br> (注文のキャンセル処理) | 146.57 ns | 142.35 ns 〜 150.82 ns | 約 682 万件 / 秒 |
| limit_resting <br> (指値注文の板乗り/Resting) | 174.56 ns | 164.23 ns 〜 186.66 ns | 約 572 万件 / 秒 |
| sweep_10_levels <br> (10価格レベルにまたがる板の全消費) | 963.30 ns | 937.02 ns 〜 987.78 ns | 約 103 万件 / 秒 |

---


###  動作環境 (Environment)
* OS / Environment: WSL2 (Ubuntu) on Windows
* Hardware: AMD/Intel CPU (Laptop: ASUS Zenbook series)
* Runner: cargo bench (Criterion.rs backend)

### AI使用について
#### AIを以下の用途で使用しました。
* 各ファイルのボイラープレート記述
* engine_benches.rsファイルの記述

