use crate::core::metrics::METRIC_ACTIVE_USERS;
use crate::services::USER_SERVICE;
use crate::{
    core::ic_cdk::api::{
        canister_balance, data_certificate, print, set_certified_data, time, trap,
    },
    SERVICE_NAME,
};
use ic_cdk_macros::query;
use ic_http_certification::{HttpRequest, HttpResponse};
use orbit_essentials::http::{certify_assets, serve_asset};
use orbit_essentials::metrics::with_metrics_registry;

#[query(decoding_quota = 10000)]
fn http_request(req: HttpRequest) -> HttpResponse {
    // If no data certificate is available (in an update call),
    // then we can refresh the metrics (note that this does not invalidate
    // the certificate since any state changes in an update call
    // to a query method are discarded at the end).
    if data_certificate().is_none() {
        if let Err(err) = refresh_metrics() {
            print(format!("Failed to refresh metrics: {err}"));
        }
    }
    let res = serve_asset(&req, data_certificate());
    match res {
        Ok(response) => response,
        Err(err) => trap(err),
    }
}

// Certification
fn refresh_metrics() -> Result<Vec<u8>, String> {
    // Trigger active users metric update.
    METRIC_ACTIVE_USERS.with(|metric| metric.borrow_mut().refresh(time()));

    // Add dynamic metrics, dropped after the request since query calls don't save state changes.
    with_metrics_registry(SERVICE_NAME, |registry| {
        registry
            .gauge_mut(
                "canister_cycles_balance",
                "cycles balance available to the canister",
            )
            .set(canister_balance() as f64);
    });
    with_metrics_registry(SERVICE_NAME, |registry| {
        registry
            .gauge_mut(
                "metrics_timestamp",
                "UNIX timestamp in nanoseconds when the metrics were exported",
            )
            .set(time() as f64);
    });
    let metrics_contents =
        with_metrics_registry(SERVICE_NAME, |registry| registry.export_metrics());
    certify_assets(vec![
        (
            "/metrics".to_string(),
            metrics_contents.unwrap_or_else(|e| e.to_string().as_bytes().to_vec()),
        ),
        ("/metrics/sd".to_string(), metrics_service_discovery()),
    ])
}

pub fn certify_metrics() {
    match refresh_metrics() {
        Ok(certified_data) => {
            set_certified_data(&certified_data);
        }
        Err(err) => {
            print(format!("Failed to refresh metrics: {err}"));
        }
    }
}

/// Returns all deployed station hosts for Prometheus service discovery.
///
/// As defined by https://prometheus.io/docs/prometheus/latest/configuration/configuration/#http_sd_config
fn metrics_service_discovery() -> Vec<u8> {
    let station_hosts = USER_SERVICE
        .get_all_deployed_stations()
        .iter()
        .map(|station| format!("{}.raw.icp0.io", station.to_text()))
        .collect::<Vec<String>>();

    format!(
        r#"[{{"targets": ["{}"],"labels": {{"__metrics_path__":"/metrics","dapp":"orbit"}}}}]"#,
        station_hosts.join("\", \"")
    )
    .as_bytes()
    .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{models::user_model_utils::mock_user, repositories::USER_REPOSITORY};
    use candid::Principal;
    use orbit_essentials::repository::Repository;

    #[test]
    fn test_service_discovery() {
        let mut user = mock_user();
        user.deployed_stations = vec![Principal::from_slice(&[0; 29])];
        let station_host = format!("{}.raw.icp0.io", user.deployed_stations[0].to_text());

        USER_REPOSITORY.insert(user.to_key(), user.clone());

        let response = metrics_service_discovery();

        assert_eq!(
            response,
            format!(
                r#"[{{"targets": ["{}"],"labels": {{"__metrics_path__":"/metrics","dapp":"orbit"}}}}]"#,
                station_host
            )
            .as_bytes()
            .to_owned()
        );
    }
}
