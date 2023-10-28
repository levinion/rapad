use embassy_rp::adc::{Async, Channel};

use crate::keyboard::Keyboard;
use crate::keycode;

pub enum State {
    Release,
    Trigger,
}

pub struct Key<'a> {
    pub keycode: crate::keycode::Keycode,
    pub bottom_level: u16,   // 底部模拟值
    pub top_level: u16,      // 顶部模拟值
    pub trigger_stroke: f64, // 触发行程
    pub release_stroke: f64, // 释放行程
    pub dead_zone: f64,      // 死区
    channel: Option<embassy_rp::adc::Channel<'a>>,
    pub state: State,
    last_max_stroke: f64,
    last_min_stroke: f64,
}

impl<'a> Default for Key<'a> {
    fn default() -> Self {
        Self {
            keycode: keycode::Keycode::KeyA,
            bottom_level: 100,
            top_level: 0,
            trigger_stroke: 0.2,
            release_stroke: 0.2,
            dead_zone: 0.03,
            channel: None,
            state: State::Release,
            last_max_stroke: 0.0,
            last_min_stroke: 1.0,
        }
    }
}

impl<'a> Key<'a> {
    pub fn new(keycode: keycode::Keycode, channel: Channel<'a>) -> Self {
        Self {
            keycode,
            channel: Some(channel),
            ..Default::default()
        }
    }

    // 总行程
    fn sum_stroke(&self) -> u16 {
        self.top_level.abs_diff(self.bottom_level)
    }

    // 最大值
    fn max_level(&self) -> u16 {
        self.top_level.max(self.bottom_level)
    }

    // 当前行程百分比
    async fn current_stroke(&mut self, adc: &mut embassy_rp::adc::Adc<'a, Async>) -> f64 {
        let raw_level = adc.read(self.channel.as_mut().unwrap()).await.unwrap();
        (self.max_level() - raw_level) as f64 / self.sum_stroke() as f64
    }

    async fn press(&mut self, kb: &mut Keyboard<'a>) {
        self.state = State::Trigger;
        kb.press(self.keycode).await;
    }

    async fn release(&mut self, kb: &mut Keyboard<'a>) {
        self.state = State::Release;
        kb.release(self.keycode).await;
    }

    pub async fn process(&mut self, adc: &mut embassy_rp::adc::Adc<'a, Async>, kb: &mut Keyboard<'a>) {
        let current_stroke = self.current_stroke(adc).await;
        match self.state {
            State::Release => {
                // update min stroke
                if current_stroke < self.last_min_stroke {
                    self.last_min_stroke = current_stroke;
                }
                if current_stroke >= (self.last_min_stroke + self.trigger_stroke).min(1.0 - self.dead_zone) {
                    self.press(kb).await;
                    self.last_min_stroke = 1.0;
                }
            }
            State::Trigger => {
                // update max stroke
                if current_stroke > self.last_max_stroke {
                    self.last_max_stroke = current_stroke;
                }
                if current_stroke <= (self.last_max_stroke - self.release_stroke).max(0.0 + self.dead_zone) {
                    self.release(kb).await;
                    self.last_max_stroke = 0.0;
                }
            }
        }
    }
}
