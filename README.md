# BTC/USDT Matching Engine (Trial)
## Phase 1 
22:34 25/05/2026 
<br>基本的な機能のみを搭載したマッチングエンジンの試作
<br>Prototype of a matching engine equipped with only basic functions

Benchmark
<br>キャンセルオーダーのレイテンシが目立つ.
<br>スウィープもレイテンシ削減の余地あり

|  操作	|  現在	|  目標 (Phase 2)	|  ボトルネック  |
|-------|-------|-----------------|---------------|
|limit resting |	132 ns|	< 80 ns |	BTreeMap挿入 |
|full match (1 level)	| 207 ns | < 120 ns  |	Trade Vec alloc |
| sweep 10 levels	| 1.83 µs |	< 600 ns	| BTreeMap per-level |
| cancel (100 orders)	| 10.2 µs | < 200 ns	| VecDeque 線形探索 |
| cancel (1000 orders) | 101 µs |	< 300 ns | VecDeque O(n) scan |

