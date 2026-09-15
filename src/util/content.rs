use crate::util::config::Config;
use crate::util::error::ServiceResult;
use crate::util::types::EmailConfig;
use aws_sdk_sesv2::primitives::Blob;
use aws_sdk_sesv2::types::{EmailContent as SESEmailContent, RawMessage};
use shared::error::ServiceError as SharedError;

pub struct EmailContent {
    to: String,
    cc: Vec<String>,
}

impl EmailContent {
    pub fn builder() -> EmailContentBuilder {
        EmailContentBuilder::default()
    }
}

#[derive(Default)]
pub struct EmailContentBuilder {
    to: Option<String>,
    cc: Vec<String>,
    html: Option<String>,
    attachments: Option<Vec<(String, Blob)>>,
}

impl EmailContentBuilder {
    pub fn to(mut self, to: impl Into<String>) -> Self {
        self.to = Some(to.into());
        self
    }

    pub fn cc(mut self, cc: impl Into<String>) -> Self {
        self.cc.push(cc.into());
        self
    }

    pub fn html(mut self, html: impl Into<String>) -> Self {
        self.html = Some(html.into());
        self
    }

    pub fn attachment(mut self, file_name: impl Into<String>, blob: Blob) -> Self {
        let mut attachments = self.attachments.unwrap_or_default();
        attachments.push((file_name.into(), blob));

        self.attachments = Some(attachments);

        self
    }

    pub fn build(
        &self,
        config: &Config,
        email_config: &EmailConfig,
    ) -> ServiceResult<SESEmailContent> {
        use lettre::message;

        let content = lettre::Message::builder();
        let email_from = email_config
            .email_from
            .as_ref()
            .unwrap_or(&config.email_from);

        let Some(ref subject) = email_config.subject else {
            return Err(SharedError::HttpMessage(400, "subject is required".into()));
        };

        let Some(email_to) = self.to.as_ref() else {
            return Err(SharedError::HttpMessage(400, "email to is required".into()));
        };

        let Some(html) = self.html.as_ref() else {
            return Err(SharedError::HttpMessage(
                400,
                "html content to is required".into(),
            ));
        };

        let content = content
            .from(email_from.parse().map_err(anyhow::Error::from)?)
            .to(format!("Recipient <{email_to}>")
                .parse()
                .map_err(anyhow::Error::from)?)
            .subject(subject);

        let mut content_body = message::MultiPart::mixed().singlepart(
            message::SinglePart::builder()
                .header(message::header::ContentType::TEXT_HTML)
                .body(String::from(html)),
        );

        if let Some(attachments) = &self.attachments {
            while let Some(attachment) = attachments.iter().next() {
                content_body =
                    content_body.singlepart(message::Attachment::new(attachment.0.clone()).body(
                        attachment.1.as_ref().to_vec(),
                        message::header::ContentType::TEXT_PLAIN,
                    ));
            }
        }

        let content = content
            .multipart(content_body)
            .map_err(anyhow::Error::from)?;

        let content = RawMessage::builder()
            .data(Blob::new(content.formatted()))
            .build()
            .map_err(anyhow::Error::from)?;

        Ok(SESEmailContent::builder().raw(content).build())
    }
}
