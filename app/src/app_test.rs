use super::*;
use more_asserts as ma;
use std::time::{Duration, Instant};
use warp::Future;

#[tokio::test]
async fn get_health_started() {
    let res = warp::test::request()
        .method("GET")
        .path("/health/started")
        .reply(&composed_filters())
        .await;

    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.body().is_empty());
}

#[tokio::test]
async fn get_health_ready() {
    let res = warp::test::request()
        .method("GET")
        .path("/health/ready")
        .reply(&composed_filters())
        .await;

    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.body().is_empty());
}

#[tokio::test]
async fn get_health_live() {
    let res = warp::test::request()
        .method("GET")
        .path("/health/live")
        .reply(&composed_filters())
        .await;

    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.body().is_empty());
}

#[tokio::test]
async fn get_env() {
    let res = warp::test::request()
        .method("GET")
        .path("/env")
        .reply(&composed_filters())
        .await;

    assert_eq!(res.status(), StatusCode::OK);

    let json_str: &str = std::str::from_utf8(res.body()).unwrap();
    let json_val: serde_json::Value = serde_json::from_str(json_str).unwrap();
    assert!(json_val.is_object())
}

#[tokio::test]
async fn post_root() {
    let event_type = "com.acme.events.something";
    let event_data = r#"{"body":"hello world", "volume": 10}"#;
    // todo: Is there a way/binding to make a warp Request from a CloudEvent?
    // Construct a Request by setting the body and required CloudEvent headers.
    let req = warp::test::request()
        .method("POST")
        .path("/")
        .header("ce-type", event_type)
        .header("ce-source", "com.acme.apps.ingress")
        .header("ce-id", "abcd")
        .header("ce-specversion", "1.0")
        .header("ce-time", "2023-07-02T00:00:00Z")
        .header("ce-datacontenttype", "application/json")
        .header("ce-something", "no nulls")
        .body(event_data);

    let res = req.reply(&composed_filters()).await;

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.headers().get("ce-type").unwrap().to_owned(),
        format!("{event_type}.echoed")
    );

    let received_event: Event =
        cloudevents::binding::http::to_event(res.headers(), res.body().to_vec()).unwrap();
    let returned_type = received_event.ty().to_owned();
    println!("{returned_type}: {:?}", received_event);
    let custom_msg: CustomMessage = received_event.try_into().unwrap();
    println!("{returned_type}: {:?}", custom_msg)
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
async fn time_get_env() {
    let (_res, duration) = time_async(
        warp::test::request()
            .method("GET")
            .path("/env")
            .reply(&composed_filters()),
    )
    .await;

    // Get environment variables in less than 2 ms.
    ma::assert_le!(duration.as_secs_f64(), 0.002)
}
