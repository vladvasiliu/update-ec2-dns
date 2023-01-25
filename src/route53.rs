use anyhow::Result;
use aws_config::SdkConfig;
pub use aws_sdk_route53::model::ChangeAction;
use aws_sdk_route53::model::{Change, ChangeBatch, ResourceRecord, ResourceRecordSet, RrType};

use tracing::info;

pub struct Route53Client {
    client: aws_sdk_route53::Client,
    zone_id: String,
}

impl Route53Client {
    pub fn new(config: &SdkConfig, zone_id: &str) -> Self {
        Self {
            client: aws_sdk_route53::Client::new(config),
            zone_id: zone_id.to_string(),
        }
    }

    pub async fn change_record_set(&self, changes: Vec<Change>, comment: &str) -> Result<()> {
        let change_batch = ChangeBatch::builder()
            .comment(comment)
            .set_changes(Some(changes))
            .build();

        let response = self
            .client
            .change_resource_record_sets()
            .hosted_zone_id(&self.zone_id)
            .change_batch(change_batch)
            .send()
            .await?;
        if let Some(change_info) = response.change_info() {
            let status = change_info
                .status()
                .map_or_else(|| "UNKNOWN", |s| s.as_str());
            info!(
                change.id = change_info.id,
                change.status = status,
                "Changed record sets"
            );
        } else {
            info!("Changed record sets");
        }

        Ok(())
    }
}

pub fn get_change(
    action: ChangeAction,
    record_name: &str,
    record_type: RrType,
    record_value: &str,
) -> Change {
    let rr = ResourceRecord::builder().value(record_value).build();
    let rrset = ResourceRecordSet::builder()
        .name(record_name)
        .r#type(record_type)
        .resource_records(rr)
        .ttl(100)
        .build();
    Change::builder()
        .resource_record_set(rrset)
        .action(action)
        .build()
}
