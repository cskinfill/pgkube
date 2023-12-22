use anyhow::Result;
use garde::Validate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use std::{time::Duration, error::Error};
use tracing::*;

use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::{
    api::{Api, DeleteParams, ListParams, Patch, PatchParams, PostParams, ResourceExt},
    core::crd::CustomResourceExt,
    Client, CustomResource,
};

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, Validate, JsonSchema)]
#[kube(group = "service.pagerduty.cs.dev", version = "v1alpha1", kind = "Integration", namespaced)]
#[kube(status = "IntegrationStatus")]
// #[kube(scale = r#"{"specReplicasPath":".spec.replicas", "statusReplicasPath":".status.replicas"}"#)]
#[kube(printcolumn = r#"{"name":"Team", "jsonPath": ".spec.metadata.team", "type": "string"}"#)]
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

    let docs: Api<Integration> = Api::default_namespaced(client);
    let d = Integration::new("guide", IntegrationSpec{service: "ABCDE".into(), secret: "testme".into()});
    println!("doc: {:?}", d);
    println!("crd: {:?}", serde_yaml::to_string(&Integration::crd()));
    println!("Hello, world!");

    Ok(())
}
