# email

Library crate for sending transactional email through Amazon SES. Provides an
`EmailService` trait (currently one implementation, `SESService`) plus builders for
configuring the service and composing a message.

## What it does

- **`EmailService::send_email`** / **`send_emails`**: sends one or many emails. Batches
  are sent concurrently (one SES `SendEmail` call per message, spawned on a `JoinSet`),
  not sequentially.
- **Named templates**: register `EmailConfig { email_from, subject }` values under a key
  at config time, then send by referencing the key (`SendEmailConfig::Template`) instead
  of repeating `from`/`subject` per call. `SendEmailConfig::Raw` supplies one inline
  instead.
- **Message building**: `EmailContent::builder()` composes the actual HTML body and
  attachments (raw MIME, built with `lettre` and sent via SES's `SendRawEmail`-style raw
  content API).

## Wiring it into an app

```rust
use email::{
    Config, EmailConfig, EmailContent, EmailService, SESService, SendEmailConfig, SendEmailPayload,
};
use std::sync::Arc;

let config = Config::builder()
    .email_from("no-reply@example.com")
    .config_set("my-ses-configuration-set") // optional
    .template(
        "welcome",
        EmailConfig { email_from: None, subject: Some("Welcome!".into()) },
    )
    .build()
    .await;

let email_service = SESService::new(Arc::new(config));

let payload = SendEmailPayload {
    config: SendEmailConfig::Template("welcome"),
    content: EmailContent::builder()
        .to("user@example.com")
        .html("<p>Hello!</p>"),
};

email_service.send_email(payload).await?;
```

## Config

Built via `Config::builder()...build().await` (uses `aws_config::load_from_env()` for
the SES client, so it needs AWS credentials/region available the usual way — env vars,
instance profile, etc.):

| Method          | Required | Description                                                          |
| --------------- | -------- | ---------------------------------------------------------------------|
| `.email_from()` | yes      | Default `From` address, used when a template/call doesn't override it. Panics if omitted. |
| `.config_set()` | no       | SES configuration set name to attach to every send.                  |
| `.template()`   | no       | Register one named `EmailConfig { email_from, subject }`, keyed by string. Can be called repeatedly. |
| `.templates()`  | no       | Register a whole template map at once, replacing any set so far.     |

## Known gaps

- **`EmailContentBuilder::cc(...)` is accepted but not sent.** The value is stored but
  `build()` never adds it to the outgoing message — CC recipients are silently dropped
  today.
- **SES/SDK send failures all collapse to a generic 500** (`"Unable to send email"`,
  `util/error.rs`) — the specific SES error (throttling, invalid address, etc.) isn't
  distinguished or surfaced.
- Attachments are always sent as `text/plain` regardless of the file's actual type.
