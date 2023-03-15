#[derive(Debug)]
pub enum TikTokRsErr {
    InitFailure,
    CloseFailure,
    PrepareFailure,
    UnknownDevice,
    ScriptEvalFailed,
    ScriptLoadFailed,
    EvaluationFailure,
    InvalidUrlFormat,
    CipherEncryptFailure,
    NavigationParseError,
}
