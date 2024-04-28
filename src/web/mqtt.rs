use esp_idf_svc::mqtt::client::{EspMqttClient, EspMqttConnection, MqttClientConfiguration};
use esp_idf_sys::EspError;

pub fn init_client(
    url: &str,
    client_id: &str,
) -> Result<(EspMqttClient<'static>, EspMqttConnection), EspError> {
    let (mqtt_client, mqtt_conn) = EspMqttClient::new(
        &url,
        &MqttClientConfiguration {
            client_id: Some(&client_id),
            ..Default::default()
        },
    )?;

    Ok((mqtt_client, mqtt_conn))
}
