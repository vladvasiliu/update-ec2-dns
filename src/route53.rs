use aws_config::SdkConfig;

pub struct Route53Client {
    client: aws_sdk_route53::Client,
}

impl Route53Client {
    pub fn new(config: &SdkConfig) -> Self {
        Self {
            client: aws_sdk_route53::Client::new(config),
        }
    }
}
