use base64::{engine::general_purpose, Engine as _};

use urlencoding::encode;

pub fn tts(payload: String) -> std::vec::Vec<u8> {
    let tts_text = if payload.is_empty() {
        "Che fai? Mi mandi una stringa vuota?"
    } else {
        payload.as_str()
    };
    let tts_lang = "it";
    let tts_slow = "false";
    let tts_tld = "it";
    let url = format!(
        "https://translate.google.{}/_/TranslateWebserverUi/data/batchexecute",
        tts_tld
    );

    let payload_raw = format!(
        "[[[\"jQ1olc\",\"[\\\"{}\\\",\\\"{}\\\",{},\\\"null\\\"]\",null,\"generic\"]]]",
        tts_text, tts_lang, tts_slow
    );

    let payload_urlencoded = encode(&payload_raw);
    let data_urlencoded = format!("f.req={}&", payload_urlencoded);
    // let data_json = [("f.req", payload_urlencoded)];
    // println!("{:?}", data_json);

    let response_urlencoded = ureq::post(&url)
        .set(
            "Content-Type",
            "application/x-www-form-urlencoded;charset=utf-8",
        )
        .send_string(&data_urlencoded);

    let response_text = response_urlencoded.unwrap().into_string().unwrap();
    let audio_str: Vec<String> = response_text.split("\\\"").map(|s| s.to_string()).collect();

    general_purpose::STANDARD
        .decode(audio_str[1].clone())
        .unwrap()
}
