mod services;
mod util;

pub use services::EmailService;
pub use services::ses::SESService;
pub use util::config::Config;
pub use util::content::EmailContent;
pub use util::error::ServiceResult;
pub use util::types::{EmailConfig, SendEmailConfig, SendEmailPayload};

#[cfg(test)]
mod tests {}
