//! Built-in PII recognizers.

pub mod email;
pub mod url;
pub mod phone;
pub mod credit_card;
pub mod ip_address;

pub use email::EmailRecognizer;
pub use url::UrlRecognizer;
pub use phone::PhoneRecognizer;
pub use credit_card::CreditCardRecognizer;
pub use ip_address::IpAddressRecognizer;
