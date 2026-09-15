use super::EmailService;
use crate::util::config::Config;
use crate::util::types::{SendEmailConfig, SendEmailPayload};

use shared::error::AppError;

use crate::util::error::{Error, ServiceError, ServiceResult};
use async_trait::async_trait;
use aws_sdk_sesv2::error::{ProvideErrorMetadata, SdkError};
use aws_sdk_sesv2::operation::send_email::{SendEmailError, SendEmailOutput};
use std::sync::Arc;
use tokio::task::JoinSet;

pub struct SESService {
    config: Arc<Config>,
}

impl SESService {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }
}

#[async_trait]
impl EmailService for SESService {
    async fn send_email(&self, payload: SendEmailPayload) -> ServiceResult<()> {
        let mut responses = self.send_emails(vec![payload]).await;

        responses.pop().unwrap_or_else(|| {
            Err(AppError::HttpMessage(
                500,
                "Failed to send email".to_string(),
            ))
        })
    }

    async fn send_emails(&self, payloads: Vec<SendEmailPayload>) -> Vec<ServiceResult<()>> {
        let config = &self.config.clone();

        let mut emails_sent: JoinSet<(usize, Result<SendEmailOutput, SdkError<SendEmailError>>)> =
            JoinSet::new();

        let mut responses = Vec::with_capacity(payloads.len());
        responses.resize_with(payloads.len(), || {
            Err(AppError::HttpMessage(500, "not sent".into()))
        });

        for (i, payload) in payloads.iter().enumerate() {
            let email_template = match payload.config {
                SendEmailConfig::Raw(ref raw_email) => &raw_email,
                SendEmailConfig::Template(template_key) => {
                    let email_template = config.templates.get(template_key);
                    let Some(email_template) = email_template else {
                        responses[i] = Err(Error::from(ServiceError::EmailTemplateMissing).into());

                        continue;
                    };

                    email_template
                }
            };

            let email_content = match payload.content.build(config, email_template) {
                Ok(content) => content,
                Err(e) => {
                    responses[i] = Err(e);
                    continue;
                }
            };

            let from_email = email_template
                .email_from
                .as_ref()
                .unwrap_or(&config.email_from);

            let email_sent = config.client.send_email().content(email_content);

            let email_sent = if let Some(config_set) = &config.config_set {
                email_sent.configuration_set_name(config_set)
            } else {
                email_sent
            };

            let email_sent = email_sent.from_email_address(from_email).send();

            emails_sent.spawn(async move {
                let res = email_sent.await;
                (i, res)
            });
        }

        while let Some(res) = emails_sent.join_next().await {
            match res {
                Ok((id, res)) => {
                    responses[id] = res.map(|_| ()).map_err(|e| Error::from(e).into());
                }
                Err(_) => {
                    continue;
                }
            };
        }

        responses
    }
}
