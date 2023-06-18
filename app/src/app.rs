use cloudevents::binding::warp::{filter::to_event, reply::from_event};
use cloudevents::AttributesWriter;
use cloudevents::{AttributesReader, Event};
use env_logger;
use log::info;
use serde_json;
use std::collections::HashMap;
use std::env;
use uuid::Uuid;
use warp::{Filter, Reply};

#[tokio::main]
/// Program entry point.
async fn main() {
    env_logger::init();

    let hostname = env::var("HOSTNAME").unwrap_or("[hostname]".to_owned());

    // Use compile-time Cargo environment variables to set log string.
    info!(
        "Application {} version {} running on {}.",
        option_env!("CARGO_PKG_NAME").unwrap_or("[name]"),
        option_env!("CARGO_PKG_VERSION").unwrap_or("[version]"),
        hostname
    );

    // Use run-time container environment variables to set log string.
    info!(
        "Release {} revision {} of chart {} version {} in namespace {}.",
        env::var("HELM_RELEASE_NAME").unwrap_or("[name]".to_owned()),
        env::var("HELM_RELEASE_REVISION").unwrap_or("[version]".to_owned()),
        env::var("HELM_CHART_NAME").unwrap_or("[name]".to_owned()),
        env::var("HELM_CHART_VERSION").unwrap_or("[version]".to_owned()),
        env::var("HELM_RELEASE_NAMESPACE").unwrap_or("[namespace]".to_owned()),
    );

    // Start server.
    warp::serve(composed_filters())
        .run(([0, 0, 0, 0], 8080))
        .await;
}

/// Filter: Compose all of the app's filters with or().
fn composed_filters() -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Copy {
    filter_root_post()
        .or(filter_get_root_env())
        .or(filter_get_any())
}

/// Filter: GET /
///
/// Respond with 200 status and empty body.
fn filter_get_any() -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Copy {
    warp::get().and(warp::any()).map(warp::reply)
}

/// Filter: GET /env
///
/// Respond with 200 status and JSON body of environment variables.
fn filter_get_root_env() -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Copy {
    warp::get().and(warp::path("env")).map(get_root_env)
}

/// Filter: POST /
///
/// Accepts only CloudEvents.
/// Respond with 200 status echoed CloudEvent.
fn filter_root_post() -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Copy {
    warp::post()
        .and(warp::path::end())
        .and(to_event())
        .map(|event| post_root(event))
}

/// Handle GET request for environment variables.
fn get_root_env() -> impl Reply {
    let env_vars_map: HashMap<String, String> = std::env::vars().collect();
    let env_vars_json_obj: serde_json::Value = serde_json::to_value(&env_vars_map).unwrap();
    let env_vars_json_str: String = serde_json::to_string(&env_vars_json_obj).unwrap();
    info!("Returning env var json: {}", env_vars_json_str);
    warp::reply::json(&env_vars_map)
}

/// Handle POST request to echo a CloudEvent.
fn post_root(event: Event) -> impl Reply {
    info!("GOT: {}", event);


    let mut echoed_event = event.clone();
    echoed_event.set_id(Uuid::new_v4().to_string());
    echoed_event.set_type(format!("{}.echoed", event.ty().to_owned()));

    info!("SENDING: {}", echoed_event);
    from_event(echoed_event)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_empty_200() {
        let res = warp::test::request()
            .method("GET")
            .reply(&composed_filters())
            .await;

        assert_eq!(res.status(), 200);
        assert!(res.body().is_empty());
    }

    #[tokio::test]
    async fn test_env() {
        let res = warp::test::request()
            .method("GET")
            .path("/env")
            .reply(&composed_filters())
            .await;

        assert_eq!(res.status(), 200);

        let json_str: &str = std::str::from_utf8(res.body()).unwrap();
        let json_val: serde_json::Value = serde_json::from_str(json_str).unwrap();
        assert!(json_val.is_object())
    }
}
