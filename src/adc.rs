use embassy_rp::adc::{AdcPin, Async, Channel, Config, InterruptHandler};
use embassy_rp::bind_interrupts;

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => InterruptHandler;
});

pub struct Adc<'a> {
    adc: embassy_rp::adc::Adc<'a, Async>,
    p26: Option<Channel<'a>>,
    p27: Option<Channel<'a>>,
    p28: Option<Channel<'a>>,
    temp_sensor: Option<Channel<'a>>,
}

pub enum Pin {
    P26,
    P27,
    P28,
    TempSensor,
}

impl<'a> Adc<'a> {
    pub fn new(pin: embassy_rp::peripherals::ADC) -> Self {
        let adc = embassy_rp::adc::Adc::new(pin, Irqs, Config::default());
        Adc {
            adc,
            p26: None,
            p27: None,
            p28: None,
            temp_sensor: None,
        }
    }

    pub fn init_p26(&mut self, pin: impl embassy_rp::Peripheral<P = impl AdcPin> + 'a) {
        if self.p26.is_none() {
            self.p26 = Some(Channel::new_pin(pin, embassy_rp::gpio::Pull::None));
        }
    }

    pub fn init_p27(&mut self, pin: impl embassy_rp::Peripheral<P = impl AdcPin> + 'a) {
        if self.p27.is_none() {
            self.p27 = Some(Channel::new_pin(pin, embassy_rp::gpio::Pull::None));
        }
    }
    pub fn init_p28(&mut self, pin: impl embassy_rp::Peripheral<P = impl AdcPin> + 'a) {
        if self.p28.is_none() {
            self.p28 = Some(Channel::new_pin(pin, embassy_rp::gpio::Pull::None));
        }
    }
    pub fn init_temp_sensor(&mut self, pin: impl embassy_rp::Peripheral<P = impl AdcPin> + 'a) {
        if self.temp_sensor.is_none() {
            self.temp_sensor = Some(Channel::new_pin(pin, embassy_rp::gpio::Pull::None));
        }
    }

    pub async fn read(&mut self, pin: Pin) -> u16 {
        match pin {
            Pin::P26 => {
                if self.p26.is_some() {
                    self.adc.read(self.p26.as_mut().unwrap()).await.unwrap()
                } else {
                    0
                }
            }
            Pin::P27 => {
                if self.p27.is_some() {
                    self.adc.read(self.p27.as_mut().unwrap()).await.unwrap()
                } else {
                    0
                }
            }
            Pin::P28 => {
                if self.p28.is_some() {
                    self.adc.read(self.p28.as_mut().unwrap()).await.unwrap()
                } else {
                    0
                }
            }
            Pin::TempSensor => {
                if self.temp_sensor.is_some() {
                    self.adc.read(self.temp_sensor.as_mut().unwrap()).await.unwrap()
                } else {
                    0
                }
            }
        }
    }
}
