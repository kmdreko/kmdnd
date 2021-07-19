use awc::Client;
use kmdnd_server::{CampaignBody, CreateCampaignBody};

#[actix_rt::test]
async fn create_campaign() {
    let _ = std::thread::spawn(|| kmdnd_server::run(false));

    let body = CreateCampaignBody {
        name: "The Green Bean Brigade".into(),
    };
    let client = Client::default();
    let campaign: CampaignBody = client
        .post("http://localhost:8080/campaigns")
        .send_json(&body)
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(campaign.name, "The Green Bean Brigade".to_string());
}
