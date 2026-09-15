use crate::util::content::EmailContentBuilder;
use std::collections::HashMap;

pub struct EmailConfig {
    pub email_from: Option<String>,
    pub subject: Option<String>,
}

pub(crate) type EmailConfigs = HashMap<String, EmailConfig>;

pub struct SendEmailPayload {
    pub config: SendEmailConfig,
    pub content: EmailContentBuilder,
}

pub enum SendEmailConfig {
    Template(&'static str),
    Raw(EmailConfig),
}
