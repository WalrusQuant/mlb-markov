use reqwest::Client;

pub const BASE_URL: &str = "https://statsapi.mlb.com/api/v1";
const USER_AGENT: &str = "mlb-markov/0.1 (github.com/adamwickwire/mlb-markov)";

pub fn http_client() -> Result<Client, reqwest::Error> {
    Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(30))
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()
}
