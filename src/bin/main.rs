use std::{sync::Arc, time::Duration};

use futures::{StreamExt};

use tracing::*;

use kube::{
    api::{Api, ResourceExt},
    runtime::{Controller, controller::Action},
    Client
};

#[derive(thiserror::Error, Debug)]
pub enum Error {}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[tokio::main]
async fn main() -> Result<(), kube::Error> {
    tracing_subscriber::fmt::init();
    let client = Client::try_default().await?;

    let integrations: Api<pgkube::Integration> = Api::default_namespaced(client);
    debug!("Hello, world!");

    Controller::new(integrations.clone(), Default::default())
        .run(reconcile, error_policy, Arc::new(()))
        .for_each(|_| futures::future::ready(()))
        .await;

    Ok(())
}

async fn reconcile(obj: Arc<pgkube::Integration>, _ctx: Arc<()>) -> Result<Action> {
    info!("reconcile request: {}", obj.name_any());
    Ok(Action::requeue(Duration::from_secs(3600)))
}

fn error_policy(_object: Arc<pgkube::Integration>, _err: &Error, _ctx: Arc<()>) -> Action {
    Action::requeue(Duration::from_secs(5))
}