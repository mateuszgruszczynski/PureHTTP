use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct HttpResponse {
    status: u16,
    status_text: String,
    headers: String,
    body: serde_json::Value,
}

#[tauri::command]
async fn execute_request(
    method: String,
    url: String,
    headers: String,
    body: Option<String>,
) -> Result<HttpResponse, String> {
    let client = reqwest::Client::new();
    
    let mut request_builder = match method.as_str() {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "PATCH" => client.patch(&url),
        "DELETE" => client.delete(&url),
        _ => return Err("Unsupported HTTP method".to_string()),
    };

    // Parse and add headers from plain text format
    if !headers.trim().is_empty() {
        for line in headers.lines() {
            let line = line.trim();
            if !line.is_empty() {
                if let Some((key, value)) = line.split_once(':') {
                    let key = key.trim();
                    let value = value.trim();
                    request_builder = request_builder.header(key, value);
                }
            }
        }
    }

    // Add body if present
    if let Some(body_content) = body {
        if !body_content.trim().is_empty() {
            request_builder = request_builder.body(body_content);
        }
    }

    match request_builder.send().await {
        Ok(response) => {
            let status = response.status().as_u16();
            let status_text = response.status().canonical_reason().unwrap_or("Unknown").to_string();
            
            // Format response headers
            let mut headers_text = String::new();
            for (name, value) in response.headers() {
                headers_text.push_str(&format!("{}: {}\n", name.as_str(), value.to_str().unwrap_or("")));
            }
            
            let text = response.text().await.map_err(|e| e.to_string())?;
            
            // Try to parse as JSON, fallback to string
            let body = match serde_json::from_str::<serde_json::Value>(&text) {
                Ok(json) => json,
                Err(_) => serde_json::Value::String(text),
            };
            
            Ok(HttpResponse { 
                status,
                status_text,
                headers: headers_text,
                body 
            })
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn save_request(app_handle: tauri::AppHandle, request: String) -> Result<(), String> {
    use tauri_plugin_dialog::DialogExt;
    use std::fs;
    
    let file_path = app_handle
        .dialog()
        .file()
        .add_filter("JSON files", &["json"])
        .add_filter("All files", &["*"])
        .set_file_name("request.json")
        .blocking_save_file();
    
    if let Some(path) = file_path {
        fs::write(path.as_path().unwrap(), request).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Save cancelled".to_string())
    }
}

#[tauri::command]
async fn load_request(app_handle: tauri::AppHandle) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;
    use std::fs;
    
    let file_path = app_handle
        .dialog()
        .file()
        .add_filter("JSON files", &["json"])
        .add_filter("All files", &["*"])
        .blocking_pick_file();
    
    if let Some(path) = file_path {
        fs::read_to_string(path.as_path().unwrap()).map_err(|e| e.to_string())
    } else {
        Err("Load cancelled".to_string())
    }
}

fn main() {
    // Set WebKit environment variable for Linux compatibility
    #[cfg(target_os = "linux")]
    std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            execute_request,
            save_request,
            load_request
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_request_get() {
        let method = "GET".to_string();
        let url = "https://petstore.swagger.io/v2/pet/3".to_string();
        let headers = "".to_string();
        let body = None;

        let result = execute_request(method, url, headers, body).await;
        
        match result {
            Ok(response) => {
                println!("Test successful! Status: {}", response.status);
                assert!(response.status == 200 || response.status == 404, 
                    "Expected 200 or 404, got {}", response.status);
            }
            Err(e) => {
                panic!("Request failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_save_and_load_request() {
        let test_request = r#"{
            "method": "GET",
            "url": "https://example.com",
            "headers": "{\"Content-Type\": \"application/json\"}",
            "body": ""
        }"#;

        // Test save
        let save_result = save_request(test_request.to_string()).await;
        assert!(save_result.is_ok(), "Save request failed: {:?}", save_result);

        // Test load
        let load_result = load_request().await;
        assert!(load_result.is_ok(), "Load request failed: {:?}", load_result);
        
        let loaded_data = load_result.unwrap();
        assert!(loaded_data.contains("example.com"), "Loaded data doesn't contain expected URL");
    }
}
