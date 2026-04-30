use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct CaptchaFeature {
    pub enabled: bool,
    pub key: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct Feature {
    pub enabled: bool,
    pub url: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct VoiceNode {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub public_url: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct VoiceFeature {
    pub enabled: bool,
    pub nodes: Vec<VoiceNode>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct StoatFeatures {
    pub captcha: CaptchaFeature,
    pub email: bool,
    pub invite_only: bool,
    pub autumn: Feature,
    pub january: Feature,
    pub livekit: VoiceFeature,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct BuildInformation {
    pub commit_sha: String,
    pub commit_timestamp: String,
    pub semver: String,
    pub origin_url: String,
    pub timestamp: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct StoatConfig {
    pub revolt: String,
    pub features: StoatFeatures,
    pub ws: String,
    pub app: String,
    pub vapid: String,
    pub build: BuildInformation,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct AutumnResponse {
    pub id: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct RatelimitFailure {
    pub retry_after: u128,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "lowercase")]
pub enum Tag {
    Attachments,
    Avatars,
    Backgrounds,
    Icons,
    Banners,
    Emojis,
}

impl Tag {
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    pub fn to_string(self) -> String {
        self.as_str().to_string()
    }
}

impl Into<&'static str> for Tag {
    fn into(self) -> &'static str {
        match self {
            Tag::Attachments => "attachments",
            Tag::Avatars => "avatars",
            Tag::Backgrounds => "backgrounds",
            Tag::Icons => "icons",
            Tag::Banners => "banners",
            Tag::Emojis => "emojis",
        }
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DataLogin {
    Email {
        /// Email
        email: String,
        /// Password
        password: String,
        /// Friendly name used for the session
        friendly_name: Option<String>,
    },
    MFA {
        /// Unvalidated or authorised MFA ticket
        ///
        /// Used to resolve the correct account
        mfa_ticket: String,
        /// Valid MFA response
        ///
        /// This will take precedence over the `password` field where applicable
        mfa_response: Option<MFAResponse>,
        /// Friendly name used for the session
        friendly_name: Option<String>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "result")]
pub enum ResponseLogin {
    Success(Session),
    MFA {
        ticket: String,
        allowed_methods: Vec<MFAMethod>,
    },
    Disabled {
        user_id: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MFAResponse {
    Password { password: String },
    Recovery { recovery_code: String },
    Totp { totp_code: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    #[serde(rename = "_id")]
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MFAMethod {
    Password,
    Recovery,
    Totp,
}
