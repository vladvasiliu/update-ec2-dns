use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Ec2State {
    Pending,
    Running,
    Stopping,
    Stopped,
    ShuttingDown,
    Terminated,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Ec2StateChangeNotificationDetail {
    instance_id: String,
    state: Ec2State,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Ec2StateChangeNotification {
    id: Uuid,
    detail_type: Option<String>,
    source: String,
    account: String,
    time: DateTime<Utc>,
    region: String,
    resources: Vec<String>,
    detail: Ec2StateChangeNotificationDetail,
}
