# tiktok-rs

Sign TikTok API endpoints by simulating a mobile device.

Aim of this project is to provide an easy to use `Signer` module that will let you simulate a device
which capable of signing API endpoints with one function call.

## Prerequesite

To use this library you need to make sure to install [ChromeDriver](https://chromedriver.chromium.org/).

WebDriver is an open source tool for automated testing of webapps across many browsers. It provides capabilities for navigating to web pages, user input, JavaScript execution, and more. ChromeDriver is a standalone server that implements the W3C WebDriver standard. ChromeDriver is available for Chrome on Android and Chrome on Desktop (Mac, Linux, Windows and ChromeOS).

The easiest way to install ChromeDriver is via brew

```bash
brew install --cask chromedriver
```

To use ChromeDriver you need a Chrome browser installed on your machine, ChromeDriver assumes the Chrome browser default path when launched.

To launch ChromeDriver simply run

```bash
chromedriver
```

If your Chrome browser is installed in a custom path please refer to the ChromeDriver docs.

## Usage

Make sure you are running the ChromeDriver, and then the following code can be called.

```rust
let signer = Signer::new().await?;
let signed_api_call = signer.sign("tiktok_api_url").await?;
```

## Tests

To run the tests successfully you need to launch your chromedriver after that a simple `cargo test` should start the tests.
