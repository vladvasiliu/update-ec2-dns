mod db;
mod ec2;
mod route53;

use crate::ec2::{describe_instance, Ec2StateChangeNotification};
use anyhow::{anyhow, Context};
use aws_sdk_ec2::model::InstanceStateName;
use aws_sdk_route53::model::{ChangeAction, RrType};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use tracing::{instrument, warn, Span};

static TABLE_NAME: &str = "update-ec2-dns";

type Result<T> = std::result::Result<T, Error>;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}

#[instrument(fields(instance_id), skip(event))]
async fn function_handler(event: LambdaEvent<Ec2StateChangeNotification>) -> Result<()> {
    // Extract some useful information from the request
    let action = match event.payload.detail.state {
        InstanceStateName::Running => ChangeAction::Create,
        InstanceStateName::Stopping | InstanceStateName::ShuttingDown => ChangeAction::Delete,
        _ => return Ok(()),
    };

    let config = aws_config::load_from_env().await;

    let instance_id = event.payload.detail.instance_id;
    Span::current().record("instance_id", &instance_id);

    let instance_info = db::fetch_from_db(&config, TABLE_NAME, &instance_id)
        .await?
        .ok_or_else(|| anyhow!("Instance was not found in the database. Ignoring."))?;

    let description = describe_instance(&config, &instance_id)
        .await
        .context("Failed to describe instance")?;

    if event.payload.detail.state != description.state {
        warn!(
            "Instance is not in expected state. Expected: {}. Actual: {}",
            event.payload.detail.state.as_str(),
            description.state.as_str()
        );
        return Ok(());
    }

    let mut changes = Vec::with_capacity(2);
    if let Some(v6) = description.ipv6_address {
        let change = route53::get_change(
            action.clone(),
            &instance_info.record_name,
            RrType::Aaaa,
            &v6.to_string(),
        );
        changes.push(change);
    }
    if let Some(v4) = description.public_ip_address {
        let change = route53::get_change(
            action,
            &instance_info.record_name,
            RrType::A,
            &v4.to_string(),
        );
        changes.push(change);
    }
    if changes.is_empty() {
        warn!("No IPv4 or IPv6 found");
        return Ok(());
    }

    let route53_client = route53::Route53Client::new(&config, &instance_info.zone_id);
    route53_client
        .change_record_set(changes, "update-ec2-dns")
        .await?;

    Ok(())
}
