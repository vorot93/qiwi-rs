use {
    bigdecimal::*,
    chrono::prelude::*,
    derive_more::{Display, FromStr},
    phonenumber::PhoneNumber,
    serde::{Deserialize, Serialize, Serializer},
    serde_json::Value,
    std::{collections::HashMap, fmt::Debug, net::IpAddr},
};

#[derive(Clone, Debug, Display)]
#[display(fmt = "{}{}", self.0.code().value(), self.0.national())]
pub struct QiwiUser(pub(crate) PhoneNumber);

impl Serialize for QiwiUser {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Clone, Debug, Display)]
#[display(fmt = "{}", self.0.info().number())]
pub struct QiwiCurrency(pub(crate) penny::Currency);

impl Serialize for QiwiCurrency {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MobilePinInfo {
    pub mobile_pin_used: bool,
    pub last_mobile_pin_change: String,
    pub next_mobile_pin_change: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PassInfo {
    pub password_used: bool,
    pub last_pass_change: String,
    pub next_pass_change: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PinInfo {
    pub pin_used: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum IdentificationLevel {
    Anonymous,
    Simple,
    Verified,
    Full,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentificationInfo {
    pub bank_alias: String,
    pub identification_level: IdentificationLevel,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub default_pay_currency: u64,
    pub default_pay_source: u64,
    pub email: String,
    pub first_txn_id: u64,
    pub language: String,
    pub operator: String,
    pub phone_hash: String,
    pub promo_enabled: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractInfo {
    pub blocked: bool,
    pub contract_id: u64,
    pub creation_date: DateTime<Utc>,
    pub features: Vec<Value>,
    pub identification_info: Vec<IdentificationLevel>,
    pub user_info: UserInfo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthInfo {
    pub person_id: u64,
    pub registration_date: DateTime<Utc>,
    pub bound_email: Option<String>,
    pub ip: IpAddr,
    pub last_login_date: Option<DateTime<Utc>>,
    pub mobile_pin_info: MobilePinInfo,
    pub pass_info: PassInfo,
    pub pin_info: PinInfo,
    pub contract_info: Option<ContractInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileInfo {
    pub auth_info: AuthInfo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentType {
    In,
    Out,
    QiwiCard,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentStatus {
    Waiting,
    Success,
    Error,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentSumData {
    pub amount: BigDecimal,
    pub currency: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderData {
    pub id: u64,
    pub short_name: String,
    pub long_name: String,
    pub logo_url: String,
    pub description: String,
    pub keys: String,
    pub site_url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentHistoryEntry {
    pub txn_id: u64,
    pub person_id: u64,
    pub date: DateTime<Utc>,
    pub error_code: u64,
    pub error: String,
    #[serde(rename = "type")]
    pub payment_type: PaymentType,
    pub status: PaymentStatus,
    pub status_text: String,
    pub trm_txn_id: String,
    pub account: String,
    pub sum: PaymentSumData,
    pub commission: PaymentSumData,
    pub total: PaymentSumData,
    pub provider: ProviderData,
    pub comment: String,
    pub currency_rate: BigDecimal,
    pub extras: HashMap<String, Value>,
    pub cheque_ready: bool,
    pub bank_document_available: bool,
    pub bank_document_ready: bool,
    pub repeat_payment_enabled: bool,
    pub favorite_payment_enabled: bool,
    pub regular_payment_enabled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentHistoryData {
    pub data: Vec<PaymentHistoryEntry>,
    pub next_txn_id: Option<u64>,
    pub next_txn_date: Option<String>,
}

#[derive(Clone, Copy, Debug, Display, FromStr, Serialize, Deserialize)]
pub struct ProviderId(pub(crate) u64);

impl ProviderId {
    pub const QIWI: Self = Self(99);
    pub const VISA_RU: Self = Self(1963);
    pub const VISA_CIS: Self = Self(1960);
    pub const MASTERCARD_RU: Self = Self(21013);
    pub const MASTERCARD_CIS: Self = Self(21012);
    pub const MIR: Self = Self(31652);
    pub const TINKOFF: Self = Self(466);
    pub const ALFABANK: Self = Self(464);
    pub const PROMSVYAZBANK: Self = Self(821);
    pub const RUSSIAN_STANDARD: Self = Self(815);
    pub const OTHER_BANK: Self = Self(1717);
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommissionRange {
    pub bound: BigDecimal,
    pub rate: BigDecimal,
    pub min: BigDecimal,
    pub max: BigDecimal,
    pub fixed: BigDecimal,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommissionLimit {
    pub currency: u16,
    pub min: BigDecimal,
    pub max: BigDecimal,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommissionInfo {
    pub ranges: Vec<CommissionRange>,
    pub limits: Vec<CommissionLimit>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CommissionInfoWrapper {
    pub commission: CommissionInfo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CommissionQuoteData {
    pub amount: BigDecimal,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CommissionQuote {
    pub qw_commission: CommissionQuoteData,
}

#[derive(Clone, Debug)]
pub enum TransferDirection {
    Qiwi {
        to_phone: PhoneNumber,
        to_currency: penny::Currency,
    },
    Cellular {
        carrier: u64,
        to_phone: PhoneNumber,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferState {
    pub code: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferTransactionData {
    pub id: String,
    pub state: TransferState,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferData {
    pub transaction: TransferTransactionData,
}
