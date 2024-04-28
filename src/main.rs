use anyhow::{bail, Result};
use core::time;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::mqtt::client::{Details::Complete, EventPayload::Received, QoS};
use esp_idf_svc::mqtt::client::{
    EspMqttClient, EspMqttConnection, EventPayload, LwtConfiguration, MqttClientConfiguration,
};
use log::{error, info, warn};
use rgb_led::WS2812RMT;
use std::time::Duration;

pub mod rgb_led;
pub mod web;

use esp_idf_sys::{self as _, EspError};

// Device configuration
#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,

    #[default("")]
    wifi_psk: &'static str,

    #[default("")]
    mqtt_url: &'static str,

    #[default("")]
    mqtt_id: &'static str,
}

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;

    // Start the LED off yellow
    let mut led = rgb_led::WS2812RMT::new(peripherals.pins.gpio2, peripherals.rmt.channel0)?;
    led.set_pixel(rgb_led::RGB8::new(50, 50, 0))?;

    // The constant `CONFIG` is auto-generated by `toml_config`.
    let app_config = CONFIG;

    // Connect to the Wi-Fi network
    let _wifi = match web::wifi::init(
        &app_config.wifi_ssid,
        &app_config.wifi_psk,
        peripherals.modem,
        sysloop,
    ) {
        Ok(inner) => inner,
        Err(err) => {
            // Red!
            led.set_pixel(rgb_led::RGB8::new(50, 0, 0))?;
            bail!("Could not connect to Wi-Fi network: {:?}", err)
        }
    };

    let mqtt_cfg = MqttClientConfiguration {
        client_id: Some("gca-control-device"),
        lwt: Some(LwtConfiguration {
            topic: "gca/control-device/status",
            payload: "offline".as_bytes() as &[u8],
            qos: QoS::AtLeastOnce,
            retain: true,
        }),
        keep_alive_interval: Some(time::Duration::from_secs(10)),
        ..Default::default()
    };

    info!("Connecting to MQTT broker");
    info!("MQTT cfg {:?}", mqtt_cfg);
    warn!("Broker URL: {}", app_config.mqtt_url);

    led.set_pixel(rgb_led::RGB8::new(0, 0, 50))?;

    info!("Connecting to MQTT broker");

    let (mut client, mut conn) = web::mqtt::init_client(&app_config.mqtt_url, &app_config.mqtt_id)?;

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        info!("Hello, world!");
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

pub fn mqtt_loop(
    client: &mut EspMqttClient<'_>,
    connection: &mut EspMqttConnection,
) -> Result<(), EspError> {
    info!("MQTT Loop");

    std::thread::scope(|s| {
        info!("About to start the MQTT client");

        std::thread::Builder::new()
            .stack_size(6000)
            .spawn_scoped(s, move || {
                info!("MQTT Listening for messages");
                while let Ok(event) = connection.next() {
                    process_message(&event.payload());
                }
                info!("Connection closed");
            })
            .unwrap();

        client.publish(
            "gca/control-device/status",
            QoS::AtLeastOnce,
            true,
            "online".as_bytes() as &[u8],
        )?;
        client.subscribe("gca/api-gateway/unlock", QoS::AtLeastOnce)?;

        // Just to give a chance of our connection to get even the first published message
        std::thread::sleep(Duration::from_millis(500));

        let thread_timeout = 2;
        loop {
            std::thread::sleep(Duration::from_secs(thread_timeout));
        }
    })
}

pub fn callback(topic: &str, message: &str) {
    info!("Received message on topic '{}': '{}'", topic, message);
}

// fn process_message(payload: &EventPayload<'_, EspError>, led: &mut rgb_led::WS2812RMT) {
fn process_message(payload: &EventPayload<'_, EspError>) {
    warn!("--------PROCESS MQTT MSG----------");
    warn!("{:?}", payload);
    warn!("--------PROCESS MQTT MSG----------");

    match payload {
        EventPayload::Received {
            id: _,
            data,
            topic,
            details: _,
        } => {
            let payload = std::str::from_utf8(data).unwrap();
            info!(
                "Received message on topic '{}': '{}'",
                topic.unwrap(),
                payload
            );

            if topic.unwrap() == "gca/api-gateway/unlock" && payload == "true" {
                warn!("Unlocking door");
                // let _ = led.set_pixel(rgb_led::RGB8::new(0, 50, 0));
                std::thread::sleep(std::time::Duration::from_secs(5));
                // let _ = led.set_pixel(rgb_led::RGB8::new(0, 0, 50));
            }
        }
        _ => error!("Could not set board LED"),
    }
}
