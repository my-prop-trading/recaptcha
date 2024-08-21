use my_telemetry::MyTelemetryContext;

pub async fn verify(
    secret_key: String,
    response: String,
    remote_ip: String,
    my_telemetry: &MyTelemetryContext,
) -> Option<RecaptchaResponse> {
    let url = "https://www.google.com/recaptcha/api/siteverify";

    let mut ctx = my_telemetry.start_event_tracking("Recaptcha");

    let response = tokio::spawn(async move {
        let params = [
            ("secret", secret_key),
            ("response", response),
            ("remoteip", remote_ip), // Uncomment and add your remoteip if you want
                                     // ("remoteip", "your-remoteip-here"),
        ];

        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();

        let response = client.post(url).form(&params).send().await.unwrap();
        let recaptcha_response = response.bytes().await.unwrap().to_vec();

        println!(
            "Recaptcha response: {:?}",
            std::str::from_utf8(recaptcha_response.as_slice()).unwrap()
        );

        let recaptcha_response: RecaptchaResponse = serde_json::from_slice(&recaptcha_response)
            .expect("Failed to deserialize recaptcha response");
        recaptcha_response
    })
    .await;

    if response.is_err() {
        ctx.set_fail_result(format!("Recaptcha request is panicked"));
        return None;
    }

    let response = response.unwrap();
    response.into()
}

#[derive(serde::Deserialize, Debug)]
pub struct RecaptchaResponse {
    success: bool,
    score: Option<f64>,
    //  action: Option<String>,
    //  challenge_ts: Option<String>,
    //  hostname: Option<String>,
    #[serde(default)]
    error_codes: Vec<String>,
}

impl RecaptchaResponse {
    pub fn get_result(&self, up_to_score: f64) -> Result<(), Vec<String>> {
        if self.success {
            if let Some(score) = self.score {
                if up_to_score < score {
                    return Ok(());
                }

                return Err(vec!["Recaptcha score is too low".to_string()]);
            }
            Err(vec!["Recaptcha score is not present".to_string()])
        } else {
            Err(self.error_codes.clone())
        }
    }
}
