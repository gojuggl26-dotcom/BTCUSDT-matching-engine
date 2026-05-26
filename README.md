# BTC/USDT Matching Engine (Trial)
## Phase 1 
22:34 25/05/2026 
<br>基本的な機能のみを搭載したマッチングエンジンの試作
<br>Prototype of a matching engine equipped with only basic functions

Benchmark
<br>キャンセルオーダーのレイテンシが目立つ.
<br>スウィープもレイテンシ削減の余地あり

##  ベンチマーク結果 (Performance Benchmarks)

本マッチングエンジンは、Rustのマイクロベンチマークフレームワーク Criterion を使用し、WSL2環境下で性能測定を行いました。
###  測定結果サマリー

| 処理内容 (Benchmark) | 平均処理時間 (Mean Time) | 信頼区間 (95% Confidence Interval) | 推定スループット (Estimated Throughput) |
| :--- | :--- | :--- | :--- |
| full_match <br> (完全合致注文の約定) | 131.30 ns | 127.11 ns 〜 135.89 ns | 約 761 万件 / 秒 |
| cancel <br> (注文のキャンセル処理) | 146.57 ns | 142.35 ns 〜 150.82 ns | 約 682 万件 / 秒 |
| limit_resting <br> (指値注文の板乗り/Resting) | 174.56 ns | 164.23 ns 〜 186.66 ns | 約 572 万件 / 秒 |
| sweep_10_levels <br> (10価格レベルにまたがる板の全消費) | 963.30 ns | 937.02 ns 〜 987.78 ns | 約 103 万件 / 秒 |

---

| 処理内容 (Benchmark) | 通常時 平均 (Normal Mean) | 高負荷時 平均 (Deep Mean) | 処理遅延の増加率 |
| :--- | :--- | :--- | :--- |
| full_match (完全合致の約定) | 125.45 ns | 183.28 ns | +46.1% |
| cancel (注文のキャンセル) | 153.77 ns | 153.90 ns | +0.08%  |
| sweep_10_levels (10価格帯の全消費) | 798.77 ns | 917.08 ns | +14.8% |
| limit_resting (指値注文の板乗り) | 141.29 ns | 570.80 ns | +304.0% |

---

| 項目 | 内容 |
| :--- | :--- |
| 深い板の規模 | 買い10万件 + 売り10万件 = 計〜50MB（L3キャッシュを超える） |
| 価格配置 | 深い板は 4M〜4.1M（買い）と 6M〜6.1M（売り）、計測対象は 5M 付近のスプレッド内 |
| iter_custom | full_match/sweep/cancel はセットアップ（補充）時間を計測から完全に除外 |
| limit_resting_deep | 1000価格サイクルで BTreeMap の新規ノード挿入・既存ノード追記を混在させる |

### 結果分析
* limit-resting項目が大幅に遅延している。Btree-mapのキャッシュミス？の可能性。連続行列の使用を検討。
* Cancel耐性あり。
* 板が深いと根からの探索でキャッシュミスを起こす。

###  動作環境 (Environment)
* OS / Environment: WSL2 (Ubuntu) on Windows
* Hardware: AMD/Intel CPU (Laptop: ASUS Zenbook series)
* Runner: cargo bench (Criterion.rs backend)
* NUMAノードが一つのみ

### AI使用について
#### AIを以下の用途で使用しました。
* 各ファイルのボイラープレート記述
* engine_benches.rsファイルの記述
* 
### 実行環境２

| 項目 | 設定値 / 環境 | 最適化の目的 |
| :--- | :--- | :--- |
| 項目 | 設定値 / 環境 | 最適化の目的 |
| 実行環境 (Execution Environment) | 各自の環境に依存 | 測定ベースの統一 |
| CPUコア固定 | taskset -c 2 (Core 2 に固定) | OSスケジューラによるスレッド移動・遅延の排除 |
| NUMAノード固定 | numactl --cpunodebind=0 --membind=0 | リモートメモリへのアクセス（NUMAミス）の完全防御 |
| ビルドプロファイル | release (opt-level = 3, LTO有効) | コンパイラによるインライン化と極限最適化 |

---

| 処理（ホットパス） | 通常時の平均遅延 (`ns`) | 深い板 (`_deep`) での平均遅延 (`ns`) | 差分 (`ns`) | アルゴリズム性能評価 |
| :--- | :---: | :---: | :---: | :--- |
| `limit_resting` (指値板乗り) | 138.41 ns | 572.79 ns | +434.38 ns | 板が厚くなっても 0.5μs 帯。構成上許容範囲内。 |
| `full_match` (完全約定) | 126.80 ns | 156.83 ns | +30.03 ns | 驚異的な安定性。 データ量に影響されない $O(1)$ を維持。 |
| `cancel` (注文取消) | 133.04 ns | 123.57 ns | -9.47 ns | メモリ最適化（アリーナ/Slab等）が完璧に機能。 |
| `sweep_10_levels` (10価格連続約定) | 765.50 ns | 896.44 ns | +130.94 ns | 連続ループ処理が入っても 1μs 未満を死守。 |


