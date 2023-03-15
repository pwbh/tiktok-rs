mod error;
mod navigator;
mod utils;

use base64::{engine::general_purpose, Engine as _};
use fantoccini::{wd::Capabilities, Client, ClientBuilder};
use navigator::Navigator;
use serde_json::{json, Value};
use std::{fs, path::Path};

use self::error::TikTokRsErr;

const TIKTOK_URL: &str = "https://www.tiktok.com/";
const CHROME_DRIVER_DEFAULT_URL: &str = "http://127.0.0.1:9515/";
const PASSWORD: &[u8; 16] = b"webapp1.0+202106";
const PRE_POST_FIX_LEN: usize = 1;

#[derive(Debug)]
pub struct Signer {
    client: Client,
}

struct SignResult {
    verify_fp: String,
    signature: String,
    signed_url: String,
    xttparams: String,
    bogus: String,
    navigator: Navigator,
}

impl Signer {
    pub async fn new() -> Result<Self, TikTokRsErr> {
        let mut capabilities = Capabilities::new();

        capabilities.insert(
            String::from("goog:chromeOptions"),
            json!({
                "args": ["--headless"],
                "mobileEmulation": json!({"deviceName": "iPhone 12 Pro"})
            }),
        );

        let client = match ClientBuilder::native()
            .capabilities(capabilities)
            .connect(CHROME_DRIVER_DEFAULT_URL)
            .await
        {
            Ok(c) => c,
            Err(e) => {
                println!("{}", e);
                return Err(TikTokRsErr::InitFailure);
            }
        };

        match client.goto(TIKTOK_URL).await {
            Ok(()) => (),
            Err(_) => return Err(TikTokRsErr::PrepareFailure),
        };

        eval_script(&client, "js/signature.src".as_ref()).await?;
        eval_script(&client, "js/webmssdk.src".as_ref()).await?;

        Ok(Self { client })
    }

    pub async fn sign(&self, url: &str) -> Result<SignResult, TikTokRsErr> {
        let mut parsed_url = match url::Url::parse(url) {
            Ok(u) => u,
            Err(_) => return Err(TikTokRsErr::InvalidUrlFormat),
        };

        let params = if let Some(query) = parsed_url.query() {
            query
        } else {
            return Err(TikTokRsErr::InvalidUrlFormat);
        };

        let verify_fp = utils::generate_verify_fp();
        let signature = self.signature(url).await?;
        let bogus = self.bogus(params).await?;

        println!("queries={}", params);

        let mut queries: Vec<(&str, &str)> = params
            .split("&")
            .map(|query| {
                let splited: Vec<&str> = query.split("=").collect();
                (splited[0], splited[1])
            })
            .collect();

        let mut is_bogus_set = false;
        let mut is_signature_set = false;

        for query in queries.iter_mut() {
            if query.0 == "_signature" {
                query.1 = &signature;
                is_signature_set = true;
            }

            if query.0 == "X-Bogus" {
                query.1 = &bogus;
                is_bogus_set = true;
            }
        }

        let queries: Vec<String> = queries
            .iter()
            .map(|query| format!("{}={}", query.0, query.1))
            .collect();

        parsed_url.query_pairs_mut().clear();

        let mut params = queries.join("&");

        if !is_bogus_set {
            params.push_str(format!("&X-Bogus={}", bogus).as_ref());
        }

        if !is_signature_set {
            params.push_str(format!("&_signature={}", signature).as_ref());
        }

        let signed_url = format!("{}{}", parsed_url.to_string(), params);

        let xttparams = Signer::xttparams(&params)?;

        let navigator = self.navigator().await?;

        Ok(SignResult {
            verify_fp,
            signature,
            signed_url,
            xttparams,
            bogus,
            navigator,
        })
    }

    fn xttparams(params: &str) -> Result<String, TikTokRsErr> {
        let params = format!("{}{}", params, "&is_encryption=1");
        let cipher = libaes::Cipher::new_128(PASSWORD);
        let encrypted = cipher.cbc_encrypt(&PASSWORD[..], params.as_bytes());
        Ok(general_purpose::STANDARD.encode(encrypted))
    }

    async fn navigator(&self) -> Result<Navigator, TikTokRsErr> {
        Navigator::new(&self.client).await
    }

    async fn signature(&self, url: &str) -> Result<String, TikTokRsErr> {
        let script_eval = r#"
            const [url] = arguments;
            return window.byted_acrawler.sign({ url });
        "#;

        match self
            .client
            .execute(script_eval, vec![serde_json::to_value(url).unwrap()])
            .await
        {
            Ok(value) => Ok(get_clean_string_from_value(value)),
            Err(e) => {
                println!("{}", e);
                return Err(TikTokRsErr::ScriptEvalFailed);
            }
        }
    }

    async fn bogus(&self, params: &str) -> Result<String, TikTokRsErr> {
        let script_eval = r#"
        const [params] = arguments;  
        return window._0x32d649(params);
    "#;

        match self
            .client
            .execute(script_eval, vec![serde_json::to_value(params).unwrap()])
            .await
        {
            Ok(value) => Ok(get_clean_string_from_value(value)),
            Err(e) => {
                println!("{}", e);
                return Err(TikTokRsErr::ScriptEvalFailed);
            }
        }
    }

    pub async fn close(self) -> Result<(), TikTokRsErr> {
        match self.client.close().await {
            Ok(()) => Ok(()),
            Err(_) => Err(TikTokRsErr::CloseFailure),
        }
    }
}

async fn eval_script(client: &Client, path: &Path) -> Result<(), TikTokRsErr> {
    let script = fs::read_to_string(path).unwrap();

    match client.execute(script.as_ref(), vec![]).await {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("{}", e);
            return Err(TikTokRsErr::ScriptEvalFailed);
        }
    }
}

fn get_clean_string_from_value(value: Value) -> String {
    let s = value.to_string();
    return s[PRE_POST_FIX_LEN..s.len() - PRE_POST_FIX_LEN].to_string();
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn boots_signer_and_closes_after_success() {
        let signer = Signer::new().await;
        assert!(signer.is_ok());
        let result = signer.unwrap().close().await;
        assert!(result.is_ok());
    }

    #[test]
    fn creates_valid_xttparams_header() {
        let test_params = "app_language=en&app_name=tiktok_web&battery_info=1&browser_language=en-US&browser_name=Mozilla&browser_online=trupe&browser_platform=MacIntel";
        let xttparams = Signer::xttparams(test_params).unwrap();
        assert_eq!(xttparams, "Uc+BC1uJ7kP5X5QLp1d2k267gp7z2V4F2eC+a9DmvNBBm6BzQNeaMCsDF30G28nJ3gFMYQTjZDC49E6YmTudEFuI+0K+5PADw0No1f5FsmfblUkSpoWL6/OEA6this7BkHt5CKFx68tg0vHIKhOaqrD0GRoBsQX6qH4Uf2Hf9bKG9+HTxYaBgRxYv2EtXXjrrx6QqyuE2eozik2REOOMwA==");
    }

    #[tokio::test]
    async fn signer_creates_signed_valid_tiktok_url() {
        let tiktok_api_url = "https://www.tiktok.com/api/recommend/item_list/?aid=1988&app_language=en&app_name=tiktok_web&battery_info=1&browser_language=en-US&browser_name=Mozilla&browser_online=true&browser_platform=MacIntel&browser_version=5.0%20%28Macintosh%3B%20Intel%20Mac%20OS%20X%2010_15_7%29%20AppleWebKit%2F537.36%20%28KHTML%2C%20like%20Gecko%29%20Chrome%2F110.0.0.0%20Safari%2F537.36&channel=tiktok_web&clientABVersions=70508271%2C70830115%2C70894022%2C70941820%2C70947390%2C70951149%2C70971375%2C70405643%2C70455309&cookie_enabled=true&count=30&device_id=7197304278209578498&device_platform=web_pc&focus_state=false&from_page=fyp&history_len=2&is_fullscreen=false&is_page_visible=true&language=en&os=mac&priority_region=&referer=&region=IL&screen_height=1440&screen_width=3440&tz_name=Asia%2FJerusalem&webcast_language=en&msToken=6lqYlGuSAUoCAvYEgdMn_vfeTuKBES8CU238g7keL3B4LkXC7RFU1h4KZSeuFIL89EjroMTeBM2cqup5sDa0-TlXizeV84YW5AirQ_dDEVru6GfzqXyHJhBl2zVWVkm4UvnEC2aq25e5t9E7&X-Bogus=DFSzswVY5QiANa8UShHLcbJ68sbN&_signature=_02B4Z6wo00001T0JBSwAAIDDOipluJXox8U9CQGAACyu33";
        let signer = Signer::new().await.unwrap();
        let signed_url = signer.sign(tiktok_api_url).await;
        assert!(signed_url.is_ok());
    }

    #[tokio::test]
    async fn navigator_should_return_device_details() {
        let signer = Signer::new().await.unwrap();
        let navigation = signer.navigator().await;
        assert!(navigation.is_ok());
    }
}
