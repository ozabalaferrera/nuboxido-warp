#[test_with::executable(docker)]
mod docker {
    use cloudevents::binding::reqwest::{RequestBuilderExt, ResponseExt};
    use cloudevents::{AttributesReader, EventBuilder, EventBuilderV10};
    use lazy_static::lazy_static;
    use more_asserts as ma;
    use reqwest::StatusCode;
    use serde_json::json;
    use std::time::{Duration, Instant};
    use testcontainers::clients::{self, Cli};
    use testcontainers::core::WaitFor;
    use testcontainers::images::generic::GenericImage;
    use uuid;
    use warp::Future;

    lazy_static! {
        static ref CLIENT: Cli = clients::Cli::default();
        static ref APP_IMAGE: GenericImage = GenericImage::new(
            option_env!("CARGO_PKG_NAME").unwrap_or("app").to_owned(),
            option_env!("CARGO_PKG_VERSION")
                .unwrap_or("latest")
                .to_owned(),
        )
        .with_exposed_port(8080)
        .with_wait_for(WaitFor::Duration {
            length: Duration::from_secs(5)
        });
    }

    async fn time_async<F, O>(f: F) -> (O, Duration)
    where
        F: Future<Output = O>,
    {
        let start = Instant::now();
        let out = f.await;
        let duration = start.elapsed();
        (out, duration)
    }

    #[tokio::test]
    async fn get_health_ready() {
        let container = CLIENT.run(APP_IMAGE.clone());
        let container_port = container.get_host_port_ipv4(8080);
        let (response_res, duration) = time_async(reqwest::get(&format!(
            "http://127.0.0.1:{container_port}/health/ready"
        )))
        .await;
        let response = response_res.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(response
            .text()
            .await
            .expect("Missing response body.")
            .is_empty());

        // Get health check status in less than 1 s (includes docker overhead).
        ma::assert_le!(duration.as_secs_f64(), 1.0)
    }

    #[tokio::test]
    async fn get_env() {
        let container = CLIENT.run(APP_IMAGE.clone());
        let container_port = container.get_host_port_ipv4(8080);
        let (response_res, duration) = time_async(reqwest::get(&format!(
            "http://127.0.0.1:{container_port}/env"
        )))
        .await;
        let response = response_res.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let json_str = response.text().await.expect("Missing response body.");
        let json_val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert!(json_val.is_object());

        // Get environment variables in less than 1 s (includes docker overhead).
        ma::assert_le!(duration.as_secs_f64(), 1.0)
    }

    #[tokio::test]
    async fn post_event() {
        let container = CLIENT.run(APP_IMAGE.clone());
        let container_port = container.get_host_port_ipv4(8080);

        let client = reqwest::Client::new();

        // Prepare the event to send
        let event_type = "com.acme.events.something";
        let event_data = json!({"body":"hello world", "volume": 10});
        let event_to_send = EventBuilderV10::new()
            .id(uuid::Uuid::new_v4().to_string())
            .ty(event_type)
            .source("com.acme.apps.ingress")
            .data("application/json", event_data.clone())
            .extension("something", "not-null")
            .build()
            .expect("Unable to build cloudevent.");

        println!("SENDING: {:?}", event_to_send);

        // Send request
        let response = client
            .post(format!("http://127.0.0.1:{container_port}/"))
            .event(event_to_send)
            .expect("Unable to write request from cloudevent.")
            .send()
            .await
            .expect("Unable to send request with cloudevent.");

        println!("RESPONSE: {:?}", response);

        // Parse response as event
        let received_event = response
            .into_event()
            .await
            .expect("Unable to build cloudevent from response.");

        println!("CONV: {:?}", received_event);

        assert_eq!(received_event.ty(), format!("{event_type}.echoed"));
    }
}
