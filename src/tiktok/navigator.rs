use fantoccini::Client;
use serde::Deserialize;

use super::TikTokRsErr;

#[derive(Debug, Deserialize)]
pub struct Navigator {
    device_scale_factor: i32,
    user_agent: String,
    browser_language: String,
    browser_platform: String,
    browser_name: String,
    browser_version: String,
}

impl Navigator {
    pub async fn new(client: &Client) -> Result<Self, TikTokRsErr> {
        let script_eval = r#"
        return {
            device_scale_factor: window.devicePixelRatio,
            user_agent: window.navigator.userAgent,
            browser_language: window.navigator.language,
            browser_platform: window.navigator.platform,
            browser_name: window.navigator.appCodeName,
            browser_version: window.navigator.appVersion,
        }
    "#;

        match client.execute(script_eval, vec![]).await {
            Ok(value) => {
                if let Ok(result) = serde_json::from_value::<Navigator>(value) {
                    Ok(result)
                } else {
                    Err(TikTokRsErr::NavigationParseError)
                }
            }
            Err(e) => {
                println!("{}", e);
                return Err(TikTokRsErr::ScriptEvalFailed);
            }
        }
    }
}
