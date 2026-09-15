use crate::util::types::{EmailConfig, EmailConfigs};

#[derive(Default)]
#[non_exhaustive]
pub struct ConfigBuilder {
    email_from: Option<String>,
    config_set: Option<String>,
    templates: Option<EmailConfigs>,
}

impl ConfigBuilder {
    pub fn email_from(self, from_email: impl Into<String>) -> Self {
        Self {
            email_from: Some(from_email.into()),
            ..self
        }
    }

    pub fn config_set(self, config_set: impl Into<String>) -> Self {
        Self {
            config_set: Some(config_set.into()),
            ..self
        }
    }

    pub fn templates(self, templates: EmailConfigs) -> Self {
        Self {
            templates: Some(templates),
            ..self
        }
    }

    pub fn template(self, key: &str, template: EmailConfig) -> Self {
        let mut templates = self.templates.unwrap_or_default();
        templates.insert(key.into(), template);

        Self {
            templates: Some(templates),
            ..self
        }
    }

    pub async fn build(self) -> Config {
        Config::new(self).await
    }
}

pub struct Config {
    pub(crate) client: aws_sdk_sesv2::Client,
    pub(crate) config_set: Option<String>,
    pub(crate) email_from: String,
    pub(crate) templates: EmailConfigs,
}

impl Config {
    async fn new(builder: ConfigBuilder) -> Config {
        let config = aws_config::load_from_env().await;

        Self {
            client: aws_sdk_sesv2::Client::new(&config),
            templates: builder.templates.unwrap_or_default(),
            email_from: builder.email_from.expect("email_from is required"),
            config_set: builder.config_set,
        }
    }

    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}
