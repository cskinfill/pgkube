use garde::Validate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use kube::CustomResource;


#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, Validate, JsonSchema)]
#[kube(group = "service.pagerduty.cs.dev", version = "v1alpha1", kind = "Integration", namespaced)]
#[kube(status = "IntegrationStatus")]
// #[kube(scale = r#"{"specReplicasPath":".spec.replicas", "statusReplicasPath":".status.replicas"}"#)]
#[kube(printcolumn = r#"{"name":"Service", "jsonPath": ".spec.service", "type": "string"}"#)]
pub struct IntegrationSpec {
    #[schemars(length(min = 3))]
    #[garde(length(min = 3))]
    service: String,
    #[schemars(length(min = 1, max = 64))]
    #[garde(length(min = 1, max = 64))]
    secret: Option<String>,
}

pub type URI = String;

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
pub struct IntegrationStatus {
    pub integration: Option<URI>,
    pub error: Option<String>,
}