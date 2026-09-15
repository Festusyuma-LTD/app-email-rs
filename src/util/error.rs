use aws_sdk_sesv2::error::SdkError;
use aws_sdk_sesv2::operation::send_email::SendEmailError;
use shared::error::ServiceError as SharedError;

pub type ServiceResult<T> = Result<T, SharedError>;

pub enum ServiceError {
    EmailTemplateMissing,
}

pub struct Error {
    code: u16,
    message: String,
}

impl From<ServiceError> for Error {
    fn from(value: ServiceError) -> Self {
        match value {
            ServiceError::EmailTemplateMissing => Error {
                code: 400,
                message: "Email template missing".into(),
            },
        }
    }
}

impl From<SdkError<SendEmailError>> for Error {
    fn from(value: SdkError<SendEmailError>) -> Self {
        let (code, message) = match value {
            _ => (500, "Unable to send email".into()),
        };

        Error { code, message }
    }
}

impl Into<SharedError> for Error {
    fn into(self) -> SharedError {
        SharedError::HttpMessage(self.code, self.message)
    }
}
