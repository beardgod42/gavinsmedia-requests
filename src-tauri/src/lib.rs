use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const SEERR_URL: &str = "http://overseerr.gavinsmedia.com";

#[derive(Serialize, Deserialize)]
struct ApiResponse {
    ok: bool,
    status: u16,
    data: Value,
    cookie: Option<String>,
}

#[tauri::command]
async fn login_local(email: String, password: String) -> Result<ApiResponse, String> {
    let client = Client::new();
    let url = format!("{}/api/v1/auth/local", SEERR_URL);
    let res = client.post(&url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&serde_json::json!({ "email": email, "password": password }))
        .send().await.map_err(|e| e.to_string())?;
    let status = res.status().as_u16();
    let ok = res.status().is_success();
    let cookie = res.headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(';').next().unwrap_or("").to_string());
    let data: Value = res.json().await.unwrap_or(Value::Null);
    Ok(ApiResponse { ok, status, data, cookie })
}

#[tauri::command]
async fn api_request(method: String, endpoint: String, body: Option<Value>, token: Option<String>) -> Result<ApiResponse, String> {
    let client = Client::new();
    let url = format!("{}/api/v1{}", SEERR_URL, endpoint);
    let mut req = match method.as_str() {
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        _ => client.get(&url),
    };
    req = req.header("Content-Type", "application/json");
    req = req.header("Accept", "application/json");
    if let Some(t) = token { req = req.header("Cookie", t); }
    if let Some(b) = body { req = req.json(&b); }
    match req.send().await {
        Ok(res) => {
            let status = res.status().as_u16();
            let ok = res.status().is_success();
            let data: Value = res.json().await.unwrap_or(Value::Null);
            Ok(ApiResponse { ok, status, data, cookie: None })
        }
        Err(e) => Err(e.to_string()),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![api_request, login_local])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
