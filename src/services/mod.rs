use crate::util::error::ServiceResult;
use crate::util::types::SendEmailPayload;

use async_trait::async_trait;

pub(crate) mod ses;

#[async_trait]
pub trait EmailService {
    async fn send_email(&self, payload: SendEmailPayload) -> ServiceResult<()>;
    async fn send_emails(&self, payload: Vec<SendEmailPayload>) -> Vec<ServiceResult<()>>;
}
