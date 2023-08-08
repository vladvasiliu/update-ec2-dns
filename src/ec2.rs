use anyhow::{anyhow, Result};
use aws_config::SdkConfig;
use aws_sdk_ec2::types::InstanceStateName;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::net::{Ipv4Addr, Ipv6Addr};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", remote = "InstanceStateName")]
enum Ec2State {
    Pending,
    Running,
    Stopping,
    Stopped,
    ShuttingDown,
    Terminated,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Ec2StateChangeNotificationDetail {
    pub instance_id: String,
    #[serde(with = "Ec2State")]
    pub state: InstanceStateName,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Ec2StateChangeNotification {
    pub id: Uuid,
    pub detail_type: Option<String>,
    pub source: String,
    pub account: String,
    pub time: DateTime<Utc>,
    pub region: String,
    pub resources: Vec<String>,
    pub detail: Ec2StateChangeNotificationDetail,
}

#[derive(Debug)]
pub struct Ec2Description {
    pub instance_id: String,
    pub public_ip_address: Option<Ipv4Addr>,
    pub ipv6_address: Option<Ipv6Addr>,
    pub state: InstanceStateName,
}

pub async fn describe_instance(config: &SdkConfig, instance_id: &str) -> Result<Ec2Description> {
    let ec2_client = aws_sdk_ec2::Client::new(config);

    let response = ec2_client
        .describe_instances()
        .instance_ids(instance_id)
        .send()
        .await?;

    let instance = response
        .reservations()
        .unwrap_or_default()
        .first()
        .ok_or_else(|| anyhow!("No instance with id {} found", instance_id))?
        .instances()
        .unwrap_or_default()
        .first()
        .ok_or_else(|| anyhow!("No instance with id {} found", instance_id))?;

    let public_ip_address = match instance.public_ip_address() {
        None => None,
        Some(ip) => Some(ip.parse::<Ipv4Addr>()?),
    };

    let ipv6_address = match instance.ipv6_address() {
        None => None,
        Some(ip) => Some(ip.parse::<Ipv6Addr>()?),
    };

    let instance_id = instance
        .instance_id()
        .ok_or_else(|| anyhow!("Invalid description retrieved. Missing instance id"))?
        .to_string();

    let state = instance
        .state()
        .and_then(|s| s.name.clone())
        .ok_or_else(|| anyhow!("Missing state in instance description"))?;

    let result = Ec2Description {
        instance_id,
        public_ip_address,
        ipv6_address,
        state,
    };

    Ok(result)
}
