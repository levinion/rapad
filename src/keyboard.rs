use core::sync::atomic::{AtomicBool, Ordering};

use defmt::*;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_usb::class::hid::{HidReader, HidReaderWriter, HidWriter, ReportId, RequestHandler, State};
use embassy_usb::control::OutResponse;
use embassy_usb::{Builder, Config, Handler, UsbDevice};
use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor};
use {defmt_rtt as _, panic_probe as _};

use crate::keycode;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

pub struct KeyboardBuilder<'a> {
    device_descriptor: [u8; 256],
    config_descriptor: [u8; 256],
    bos_descriptor: [u8; 256],
    msos_descriptor: [u8; 256],
    device_handler: MyDeviceHandler,
    request_handler: MyRequestHandler,
    state: State<'a>,
    usb_config: Config<'a>,
}

impl<'a> Default for KeyboardBuilder<'a> {
    fn default() -> Self {
        let mut config = Config::new(0x16c8, 0x27dd);
        config.manufacturer = Some("mp");
        config.product = Some("mpv1");
        config.serial_number = Some("mp");
        config.max_power = 100;
        config.max_packet_size_0 = 64;

        Self {
            device_descriptor: [0; 256],
            config_descriptor: [0; 256],
            bos_descriptor: [0; 256],
            msos_descriptor: [0; 256],
            device_handler: MyDeviceHandler::new(),
            request_handler: MyRequestHandler {},
            state: State::new(),
            usb_config: config,
        }
    }
}

impl<'a> KeyboardBuilder<'a> {
    pub fn build(&'a mut self, usb: USB) -> (Keyboard<'a>, UsbDevice<'a, Driver<'a, USB>>) {
        Keyboard::setup(usb, self)
    }

    pub fn manufacturer(&mut self, manufacturer: &'a str) {
        self.usb_config.manufacturer = Some(manufacturer);
    }

    pub fn product(&mut self, product: &'a str) {
        self.usb_config.product = Some(product);
    }

    pub fn serial_number(&mut self, serial_number: &'a str) {
        self.usb_config.serial_number = Some(serial_number);
    }
}

pub struct Keyboard<'a> {
    #[allow(unused)]
    hid_reader: HidReader<'a, Driver<'a, USB>, 1>,
    hid_writer: HidWriter<'a, Driver<'a, USB>, 8>,
    report: KeyboardReport,
}

impl<'a> Keyboard<'a> {
    fn setup(usb: USB, kb: &'a mut KeyboardBuilder<'a>) -> (Self, UsbDevice<'a, Driver<'a, USB>>) {
        // Create the driver, from the HAL.
        let driver = Driver::new(usb, Irqs);

        let mut builder = Builder::new(
            driver,
            kb.usb_config,
            &mut kb.device_descriptor,
            &mut kb.config_descriptor,
            &mut kb.bos_descriptor,
            &mut kb.msos_descriptor,
        );

        builder.handler(&mut kb.device_handler);

        // Create classes on the builder.
        let config = embassy_usb::class::hid::Config {
            report_descriptor: KeyboardReport::desc(),
            request_handler: Some(&kb.request_handler),
            poll_ms: 60,
            max_packet_size: 64,
        };
        let hid = HidReaderWriter::<_, 1, 8>::new(&mut builder, &mut kb.state, config);

        let (hid_reader, hid_writer) = hid.split();

        // Build the builder.
        let usb = builder.build();

        let report = KeyboardReport {
            modifier: 0,
            reserved: 0,
            leds: 0,
            keycodes: [0, 0, 0, 0, 0, 0],
        };

        (
            Self {
                hid_reader,
                hid_writer,
                report,
            },
            usb,
        )
    }

    async fn send_report(&mut self) {
        self.hid_writer
            .write_serialize(&self.report)
            .await
            .unwrap_or_else(|err| warn!("report error: {}", err));
    }

    // send if keycode not already included and there is some place in report.
    pub async fn press(&mut self, keycode: keycode::Keycode) {
        for code in self.report.keycodes.iter_mut() {
            if *code == keycode as u8 {
                break;
            }
            if *code == 0 {
                *code = keycode as u8;
                break;
            }
        }
        self.send_report().await;
    }

    // release key in report if there exists, or do nothing.
    pub async fn release(&mut self, keycode: keycode::Keycode) {
        for code in self.report.keycodes.iter_mut() {
            if *code == keycode as u8 {
                *code = 0;
                break;
            }
        }
        self.send_report().await;
    }

    // release all keys.
    pub async fn release_all(&mut self) {
        self.report.keycodes.fill(0);
        self.send_report().await;
    }
}

struct MyRequestHandler {}

impl RequestHandler for MyRequestHandler {
    fn get_report(&self, id: ReportId, _buf: &mut [u8]) -> Option<usize> {
        info!("Get report for {:?}", id);
        None
    }

    fn set_report(&self, id: ReportId, data: &[u8]) -> OutResponse {
        info!("Set report for {:?}: {=[u8]}", id, data);
        OutResponse::Accepted
    }

    fn set_idle_ms(&self, id: Option<ReportId>, dur: u32) {
        info!("Set idle rate for {:?} to {:?}", id, dur);
    }

    fn get_idle_ms(&self, id: Option<ReportId>) -> Option<u32> {
        info!("Get idle rate for {:?}", id);
        None
    }
}

struct MyDeviceHandler {
    configured: AtomicBool,
}

impl MyDeviceHandler {
    fn new() -> Self {
        MyDeviceHandler {
            configured: AtomicBool::new(false),
        }
    }
}

impl Handler for MyDeviceHandler {
    fn enabled(&mut self, enabled: bool) {
        self.configured.store(false, Ordering::Relaxed);
        if enabled {
            info!("Device enabled");
        } else {
            info!("Device disabled");
        }
    }

    fn reset(&mut self) {
        self.configured.store(false, Ordering::Relaxed);
        info!("Bus reset, the Vbus current limit is 100mA");
    }

    fn addressed(&mut self, addr: u8) {
        self.configured.store(false, Ordering::Relaxed);
        info!("USB address set to: {}", addr);
    }

    fn configured(&mut self, configured: bool) {
        self.configured.store(configured, Ordering::Relaxed);
        if configured {
            info!("Device configured, it may now draw up to the configured current limit from Vbus.")
        } else {
            info!("Device is no longer configured, the Vbus current limit is 100mA.");
        }
    }
}
