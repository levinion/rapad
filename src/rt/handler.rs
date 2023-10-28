use embassy_rp::adc::{Adc, Async, Channel, Config, InterruptHandler};
use embassy_rp::gpio::Pull;
use embassy_rp::{bind_interrupts, peripherals};

use super::key::Key;
use crate::keyboard::Keyboard;
use crate::keycode::Keycode;

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => InterruptHandler;
});

pub struct Handler<'a> {
    keys: [Key<'a>; 2],
    kb: Keyboard<'a>,
    adc: Adc<'a, Async>,
}

impl<'a> Handler<'a> {
    pub fn new(p26: peripherals::PIN_26, p27: peripherals::PIN_27, adc: peripherals::ADC, kb: Keyboard<'a>) -> Self {
        let keys = [
            Key::new(Keycode::KeyZ, Channel::new_pin(p26, Pull::None)),
            Key::new(Keycode::KeyX, Channel::new_pin(p27, Pull::None)),
        ];
        let adc = Adc::new(adc, Irqs, Config::default());
        Self { keys, kb, adc }
    }

    pub async fn process(&mut self) {
        for key in &mut self.keys {
            key.process(&mut self.adc, &mut self.kb).await;
        }
    }
}
