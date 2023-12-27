use anyhow::Result;
use garde::Validate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use futures::{StreamExt, TryStreamExt};

use std::{time::Duration, error::Error};
use tracing::*;

use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::{
    api::{Api, DeleteParams, ListParams, Patch, PatchParams, PostParams, ResourceExt},
    runtime::{reflector, watcher, WatchStreamExt},
    core::crd::CustomResourceExt,
    Client, CustomResource,
};

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, Validate, JsonSchema)]
#[kube(group = "service.pagerduty.cs.dev", version = "v1alpha1", kind = "Integration", namespaced)]
#[kube(status = "IntegrationStatus")]
// #[kube(scale = r#"{"specReplicasPath":".spec.replicas", "statusReplicasPath":".status.replicas"}"#)]
// #[kube(printcolumn = r#"{"name":"Team", "jsonPath": ".spec.metadata.team", "type": "string"}"#)]
pub struct IntegrationSpec {
    #[schemars(length(min = 3))]
    #[garde(length(min = 3))]
    service: String,
    #[garde(skip)]
    secret: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
pub struct IntegrationStatus {
    integration_key: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let client = Client::try_default().await?;

    let integrations: Api<Integration> = Api::default_namespaced(client);
    // let d = Integration::new("guide", IntegrationSpec{service: "ABCDE".into(), secret: "testme".into()});
    // info!("doc: {:?}", d);
    // info!("crd: {:?}", serde_yaml::to_string(&Integration::crd()));
    debug!("Hello, world!");

    // docs.create(&PostParams { dry_run: false, field_manager: None }, &d).await?;
    let (reader, writer) = reflector::store::<Integration>();
    let wc = watcher::Config::default().any_semantic();
    let mut stream = watcher(integrations, wc)
        .default_backoff()
        .reflect(writer)
        .applied_objects()
        .boxed();

    tokio::spawn(async move {
        reader.wait_until_ready().await.unwrap();
        loop {
            // Periodically read our state
            // while this runs you can kubectl apply -f crd-baz.yaml or crd-qux.yaml and see it works
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            let crds = reader.state().iter().map(|r| r.name_any()).collect::<Vec<_>>();
            info!("Current crds: {:?}", crds);
        }
    });
    while let Some(event) = stream.try_next().await? {
        info!("saw {}", event.name_any());
    }

    Ok(())
}
