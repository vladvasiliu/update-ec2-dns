use anyhow::Context;
use aws_config::SdkConfig;
use aws_sdk_dynamodb::model::AttributeValue;
use serde::{Deserialize, Serialize};
use serde_dynamo::from_item;

pub async fn fetch_from_db(
    config: &SdkConfig,
    table_name: &str,
    instance_id: &str,
) -> crate::Result<Option<Ec2Info>> {
    let client = aws_sdk_dynamodb::Client::new(config);
    let key_value = AttributeValue::S(instance_id.into());

    let response = client
        .get_item()
        .table_name(table_name)
        .key("instance-id", key_value)
        .consistent_read(true)
        .send()
        .await?;

    let info = response
        .item
        .map(|i| from_item(i).context("Deserialization of the db object failed"))
        .transpose()?;

    Ok(info)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Ec2Info {
    pub instance_id: String,
    pub record_name: String,
    pub zone_id: String,
}
