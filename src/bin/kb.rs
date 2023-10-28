#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_futures::join::join;
use embassy_rp::gpio::Level;
use embassy_rp_project::keyboard;
use embassy_rp_project::rt::handler::Handler;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut builder = keyboard::KeyboardBuilder::default();
    let (kb, mut usb) = builder.build(p.USB);
    let mut led = embassy_rp::gpio::Output::new(p.PIN_25, Level::Low);

    let mut handler = Handler::new(p.PIN_26, p.PIN_27, p.ADC, kb);

    let main_loop = async {
        led.toggle();
        handler.process().await;
    };

    join(usb.run(), main_loop).await;
}
