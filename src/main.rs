mod db;
mod ec2;
mod route53;

use crate::ec2::{describe_instance, Ec2StateChangeNotification};
use aws_sdk_ec2::model::InstanceStateName;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use tracing::{info, warn};

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

async fn function_handler(event: LambdaEvent<Ec2StateChangeNotification>) -> Result<()> {
    // Extract some useful information from the request
    match event.payload.detail.state {
        InstanceStateName::Running | InstanceStateName::Stopping => (),
        _ => return Ok(()),
    };

    let config = aws_config::load_from_env().await;

    let instance_id = event.payload.detail.instance_id;

    let instance_info = match db::fetch_from_db(&config, TABLE_NAME, &instance_id).await? {
        None => {
            info!("Instance was not found in the database. Ignoring.");
            return Ok(());
        }
        Some(i) => i,
    };

    let description = describe_instance(&config, &instance_id).await?;

    if event.payload.detail.state != description.state {
        warn!(
            "Instance is not in expected state. Expected: {}. Actual: {}",
            event.payload.detail.state.as_str(),
            description.state.as_str()
        );
    }

    println!("{:?}", description);

    Ok(())
}
