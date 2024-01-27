use futures::{StreamExt, TryFutureExt, future::join, FutureExt};
use pgkube::Integration;
use serde_json::json;
use std::{collections::BTreeMap, sync::Arc, time::Duration};
use tracing::*;

use kube::{
    api::{Api, Patch, PatchParams, ResourceExt},
    core::ObjectMeta,
    runtime::{controller::Action, Controller},
    Client, Resource,
};

use k8s_openapi::{api::core::v1::Secret, Metadata};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Kube API Error")]
    KubeAPI(#[from] kube::Error),
    #[error("Unknown")]
    Unknown,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone)]
struct Ctx {
    client: Client
}


#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();
    let client = Client::try_default().await?;

    let integrations: Api<pgkube::Integration> = Api::all(client.clone());
    let secrets = Api::<Secret>::all(client.clone());
    let ctx = Ctx {
        client: client.clone()
    };
    debug!("Hello, world!");

    Controller::new(integrations.clone(), Default::default())
        .owns(secrets, Default::default())
        .run(reconcile, error_policy, Arc::new(ctx.clone()))
        .for_each(|_| futures::future::ready(()))
        .await;

    Ok(())
}

#[instrument(skip(ctx), fields(trace_id))]
async fn reconcile(obj: Arc<pgkube::Integration>, ctx: Arc<Ctx>) -> Result<Action> {

    let pgres = obj.status.clone()
    .map_or_else(|| create_pagerduty_integration(&obj), |_| get_pagerduty_integration(&obj))?;

    let secret = create_secret(obj.as_ref(), pgres.clone().key)
    .map(|secret| patch_secret(ctx.clone().client.clone(), secret))?;

    let status_update = patch_status(pgres.clone(), obj.clone(), ctx.clone().client.clone());
    
    match join(secret, status_update).await {
        (Ok(secret), Ok(status)) => {
            debug!("Updated secret {:?}: {} and status {:?}: {}",secret.namespace(), secret.name_any(), status.namespace(), status.name_any());
            Ok(Action::requeue(Duration::from_secs(60 * 60)))
        },
        (Err(e), Ok(status)) => {
            debug!("status {:?}: {}", status.namespace(), status.name_any());
            warn!("Error updating secret {}", e);
            Ok(Action::requeue(Duration::from_secs(30)))
        },
        (Ok(secret), Err(e)) => {
            debug!("Updated secret {:?}: {}", secret.namespace(), secret.name_any());
            warn!("Error updating status {:?}", e);
            Ok(Action::requeue(Duration::from_secs(30)))
        },
        (Err(e1), Err(e2)) => {
            warn!("Error updating secret {:?}", e1);
            warn!("Error updating status {:?}", e2);
            Ok(Action::requeue(Duration::from_secs(30)))
        },

    }

    
}

async fn patch_secret(client: Client, secret: Secret) -> Result<Secret, Error> {
    debug!("secret is {:?}",serde_yaml::to_string(&secret));
    Api::<Secret>::namespaced(client, secret.metadata().namespace.clone().unwrap().as_ref())
    .patch(
        &secret.name_any(),
        &PatchParams::apply("pgkubecontroller"),
        &Patch::Apply(&secret))
    .map_err(Error::KubeAPI)
    .await
}

async fn patch_status(
    integration: PGIntegration,
    obj: Arc<pgkube::Integration>,
    client: Client,
) -> Result<Integration, Error> {
    let status = json!({
        "status": pgkube::IntegrationStatus{ integration: Some(integration.url), ..Default::default()}
    });
    Api::<Integration>::namespaced(client, obj.namespace().unwrap().as_str())
    .patch_status(
        &obj.name_any(),
        &PatchParams::default(),
        &Patch::Merge(&status),
    )
    .await
    .map_err(Error::KubeAPI)
}

fn create_secret(integration: &Integration, key: String) -> Result<Secret,Error> {
    debug!(
        "create secret"
    );

    let owner_ref = integration.controller_owner_ref(&()).unwrap();

    Ok(Secret {
        metadata: ObjectMeta {
            name: integration.metadata.name.clone(),
            namespace: integration.metadata.namespace.clone(),
            owner_references: Some(vec![owner_ref]),
            ..ObjectMeta::default()
        },
        string_data: Some(BTreeMap::from([("key".to_owned(), key)])),
        ..Default::default()
    })
}

#[derive(Clone)]
struct PGIntegration {
    url: String,
    service: String,
    key: String,
}

#[instrument]
fn get_pagerduty_integration(integration: &Integration) -> Result<PGIntegration, Error> {
    info!(
        "get pg integration: {:?}",
        integration.name_any()
    );
    // returns mock API response
    Ok(PGIntegration {
        url: "https://api.pagerduty.com/services/PQL78HM/integrations/PE1U9CH".to_string(),
        service: "https://api.pagerduty.com/services/PQL78HM".to_string(),
        key: integration.name_any(),
    })
}

#[instrument]
fn create_pagerduty_integration(integration: &Integration) -> Result<PGIntegration, Error> {
    info!(
        "Create pg integration: {:?}",
        integration.name_any()
    );
    // returns mock API response
    Ok(PGIntegration {
        url: "https://api.pagerduty.com/services/PQL78HM/integrations/PABCDEF".to_string(),
        service: "https://api.pagerduty.com/services/PQZXYW".to_string(),
        key: integration.name_any(),
    })
}

fn error_policy(_object: Arc<pgkube::Integration>, _err: &Error, _ctx: Arc<Ctx>) -> Action {
    Action::requeue(Duration::from_secs(5))
}
