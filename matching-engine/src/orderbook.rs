use std::collections::VecDeque;
use rustc_hash::FxHashMap;
use crate::order::{Order, Side};
use crate::types::{OrderId, Price};

pub const PRICE_RANGE: usize = 200_000;

#[derive(Default)]
pub struct PriceLevel {
    pub orders: VecDeque<Order>,
    pub total_qty: u64,
}

pub struct OrderBook {
    pub(crate) base_tick: i64,
    pub(crate) levels: Vec<Option<PriceLevel>>,
    pub(crate) best_bid_idx: Option<usize>,
    pub(crate) best_ask_idx: Option<usize>,
    pub(crate) occupied: Vec<u64>,
    pub(crate) order_index: FxHashMap<OrderId, (Side, usize)>,
    active_bids: usize,
    active_asks: usize,
}

impl OrderBook {
    pub fn new(reference_price: Price) -> Self {
        let base_tick = reference_price.0 - (PRICE_RANGE as i64 / 2);
        let mut levels = Vec::with_capacity(PRICE_RANGE);
        levels.resize_with(PRICE_RANGE, || None);
        let occupied_words = (PRICE_RANGE + 63) / 64;
        Self {
            base_tick,
            levels,
            best_bid_idx: None,
            best_ask_idx: None,
            occupied: vec![0u64; occupied_words],
            order_index: FxHashMap::with_capacity_and_hasher(65_536, Default::default()),
            active_bids: 0,
            active_asks: 0,
        }
    }

    #[inline]
    pub fn best_bid(&self) -> Option<Price> {
        self.best_bid_idx.map(|i| Price(self.base_tick + i as i64))
    }

    #[inline]
    pub fn best_ask(&self) -> Option<Price> {
        self.best_ask_idx.map(|i| Price(self.base_tick + i as i64))
    }

    pub fn add(&mut self, order: Order) -> Result<(), &'static str> {
        let idx = self.price_to_idx(order.price).ok_or("price out of range")?;
        let side = order.side;
        let qty = order.quantity.0;
        let order_id = order.id;

        let level = self.levels[idx].get_or_insert_with(PriceLevel::default);
        let was_empty = level.orders.is_empty();
        level.orders.push_back(order);
        level.total_qty += qty;

        if was_empty {
            self.occupied[idx / 64] |= 1u64 << (idx % 64);
        }

        self.order_index.insert(order_id, (side, idx));

        match side {
            Side::Buy  => self.active_bids += 1,
            Side::Sell => self.active_asks += 1,
        }

        match side {
            Side::Buy => {
                if self.best_bid_idx.map_or(true, |b| idx > b) {
                    self.best_bid_idx = Some(idx);
                }
            }
            Side::Sell => {
                if self.best_ask_idx.map_or(true, |a| idx < a) {
                    self.best_ask_idx = Some(idx);
                }
            }
        }
        Ok(())
    }

    #[inline]
    pub fn price_to_idx(&self, price: Price) -> Option<usize> {
        let offset = price.0 - self.base_tick;
        if offset < 0 || offset >= PRICE_RANGE as i64 {
            None
        } else {
            Some(offset as usize)
        }
    }

    pub fn cancel(&mut self, order_id: OrderId) -> Option<Order> {
        let (side, idx) = self.order_index.remove(&order_id)?;
        let level = self.levels[idx].as_mut()?;
        let pos = level.orders.iter().position(|o| o.id == order_id)?;
        let removed = level.orders.remove(pos)?;
        level.total_qty -= removed.quantity.0;

        match (side, level.orders.is_empty()) {
            (Side::Buy,  true) => self.active_bids -= 1,
            (Side::Sell, true) => self.active_asks -= 1,
            _                  => {}
        }

        if level.orders.is_empty() {
            self.levels[idx] = None;
            self.occupied[idx / 64] &= !(1u64 << (idx % 64));
            self.update_best_after_removal(side, idx);
        }
        Some(removed)
    }

        pub(crate) fn update_best_after_removal(&mut self, side: Side, idx: usize) {
        match side {
            Side::Buy if self.best_bid_idx == Some(idx) => {
                // 他に買い注文がゼロ → スキャン不要、即 None
                self.best_bid_idx = if self.active_bids == 0 {
                    None
                } else {
                    self.scan_prev_occupied(idx)
                };
            }
            Side::Sell if self.best_ask_idx == Some(idx) => {
                self.best_ask_idx = if self.active_asks == 0 {
                    None
                } else {
                    self.scan_next_occupied(idx)
                };
            }
            _ => {}
        }
    }
    // ビットマップで idx より低い最大占有インデックスを検索（best_bid 更新用）
    fn scan_prev_occupied(&self, idx: usize) -> Option<usize> {
        if idx == 0 { return None; }
        let target = idx - 1;
        let word_idx = target / 64;
        let bit_pos = target % 64;
        let mask = if bit_pos == 63 { u64::MAX } else { (1u64 << (bit_pos + 1)) - 1 };
        let word = self.occupied[word_idx] & mask;
        if word != 0 {
            return Some(word_idx * 64 + (63 - word.leading_zeros() as usize));
        }
        for w in (0..word_idx).rev() {
            let word = self.occupied[w];
            if word != 0 {
                return Some(w * 64 + (63 - word.leading_zeros() as usize));
            }
        }
        None
    }

    // ビットマップで idx より高い最小占有インデックスを検索（best_ask 更新用）
    fn scan_next_occupied(&self, idx: usize) -> Option<usize> {
        let start = idx + 1;
        if start >= PRICE_RANGE { return None; }
        let word_idx = start / 64;
        let bit_pos = start % 64;
        let mask = !((1u64 << bit_pos) - 1);
        let word = self.occupied[word_idx] & mask;
        if word != 0 {
            return Some(word_idx * 64 + word.trailing_zeros() as usize);
        }
        for w in (word_idx + 1)..self.occupied.len() {
            let word = self.occupied[w];
            if word != 0 {
                return Some(w * 64 + word.trailing_zeros() as usize);
            }
        }
        None
    }
}

impl Default for OrderBook {
    fn default() -> Self {
        Self::new(Price(5_000_000))
    }
}
