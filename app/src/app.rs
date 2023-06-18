#![doc = include_str!("../README.md")]

#[cfg(test)]
mod app_test;
mod app_types;
mod ce_converter;

use app_types::*;
use cloudevents::binding::warp::{filter, reply::from_event};
use cloudevents::{AttributesReader, AttributesWriter, Event};
use log::{debug, error, info};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use warp::http::StatusCode;
use warp::{Filter, Reply};

use crate::ce_converter::InternalEvent;

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
    filter_post_root()
        .or(filter_get_root_env())
        .or(filter_get_root_health())
}

/// Filter: GET /health/*
///
/// Respond to various health probes.
fn filter_get_root_health() -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Copy
{
    warp::get()
        .and(warp::path!("health" / "started"))
        .map(get_root_health_started)
        .or(warp::get()
            .and(warp::path!("health" / "ready"))
            .map(get_root_health_ready))
        .or(warp::get()
            .and(warp::path!("health" / "live"))
            .map(get_root_health_live))
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
fn filter_post_root() -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Copy {
    warp::post()
        .and(warp::path::end())
        .and(filter::to_event())
        .map(post_root)
}

/// Handle GET request for server health (started).
fn get_root_health_started() -> impl Reply {
    debug!("Returning health (started) status.");
    // Do things to determine if service has started...
    let health: Result<(), ()> = Result::Ok(());

    match health {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}

/// Handle GET request for server health (ready).
fn get_root_health_ready() -> impl Reply {
    debug!("Returning health (ready) status.");
    // Do things to determine if service is ready for requests...
    let health: Result<(), ()> = Result::Ok(());

    match health {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}

/// Handle GET request for server health (live).
fn get_root_health_live() -> impl Reply {
    debug!("Returning health (live) status.");
    // Do things to determine if service is live/healthy...
    let health: Result<(), ()> = Result::Ok(());

    match health {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

/// Handle GET request for environment variables.
fn get_root_env() -> Box<dyn Reply> {
    let env_vars: HashMap<String, String> = std::env::vars().collect();
    if let Ok(value) = serde_json::to_value(&env_vars) {
        if let Ok(json) = serde_json::to_string(&value) {
            info!("Returning env var json: {}", json);
            return Box::new(warp::reply::json(&env_vars));
        }
    }

    Box::new(StatusCode::INTERNAL_SERVER_ERROR)
}

/// Handle POST request to echo a CloudEvent.
fn post_root(event: Event) -> Box<dyn Reply> {
    let json_value = match Value::try_from(InternalEvent(event.clone())) {
        Ok(val) => {
            info!("RECEIVED EVENT: {}", val.to_string());
            val
        }
        Err(err) => {
            error!("{err}");
            return Box::new(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let message: CustomMessage = match serde_json::from_value(json_value) {
        Ok(msg) => {
            info!("TO INTERNAL: {msg:?}");
            msg
        }
        Err(err) => {
            error!("{err}");
            return Box::new(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // You can do things with your internal type here.

    // Then turn it back into an event, or make some new event, or whatever.
    let mut output_event = match Event::try_from(message) {
        Ok(event) => event,
        Err(err) => {
            error!("{err}");
            return Box::new(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // modify your output event if you wish
    output_event.set_type(format!("{}.echoed", output_event.ty().to_owned()));

    info!("SENDING EVENT: {:?}", output_event);
    Box::new(from_event(output_event))
}
