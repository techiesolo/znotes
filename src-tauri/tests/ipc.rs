use serde_json::{Value, json};
use tauri::ipc::{CallbackFn, InvokeBody, InvokeResponseBody};
use tauri::test::MockRuntime;
use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder};
use tauri::webview::InvokeRequest;
use tauri::{App, WebviewWindow};

fn app() -> (App<MockRuntime>, WebviewWindow<MockRuntime>) {
    let app = znotes_lib::configure(mock_builder())
        .build(tauri::generate_context!())
        .expect("build mock app");
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .expect("build webview");
    (app, webview)
}

fn local_origin() -> &'static str {
    if cfg!(windows) {
        "http://tauri.localhost"
    } else {
        "tauri://localhost"
    }
}

fn invoke(webview: &WebviewWindow<MockRuntime>, cmd: &str, args: Value) -> Result<Value, Value> {
    get_ipc_response(
        webview,
        InvokeRequest {
            cmd: cmd.into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: local_origin().parse().unwrap(),
            body: InvokeBody::Json(args),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        },
    )
    .map(|body| match body {
        InvokeResponseBody::Json(text) => serde_json::from_str(&text).unwrap(),
        InvokeResponseBody::Raw(bytes) => Value::String(String::from_utf8_lossy(&bytes).into()),
    })
}

#[test]
fn locked_app_reports_status_and_rejects_data_commands() {
    let (_app, webview) = app();
    let status = invoke(&webview, "vault_status", json!({})).unwrap();
    assert_eq!(status, json!({ "locked": true, "dir": null }));

    let err = invoke(&webview, "folder_list", json!({})).unwrap_err();
    assert_eq!(err["code"], "locked");
    assert_eq!(err["message"], "vault is locked");
}

#[test]
fn full_round_trip_through_ipc() {
    let (_app, webview) = app();
    let dir = tempfile::tempdir().unwrap();
    let dir_str = dir.path().display().to_string();

    let wrong_open = invoke(
        &webview,
        "vault_open",
        json!({ "dir": dir_str, "password": "pw" }),
    )
    .unwrap_err();
    assert_eq!(wrong_open["code"], "not_a_vault");

    let info = invoke(
        &webview,
        "vault_create",
        json!({ "dir": dir_str, "password": "pw" }),
    )
    .unwrap();
    assert_eq!(info["dir"], dir_str);

    let folder = invoke(
        &webview,
        "folder_create",
        json!({ "parentId": null, "name": "Work" }),
    )
    .unwrap();
    let folder_id = folder["id"].as_i64().unwrap();

    let note = invoke(
        &webview,
        "note_create",
        json!({ "folderId": folder_id, "title": "Hello" }),
    )
    .unwrap();
    let note_id = note["id"].as_i64().unwrap();
    assert_eq!(note["title"], "Hello");

    let updated = invoke(
        &webview,
        "note_update",
        json!({ "id": note_id, "title": "Hello", "body": "tiếng việt" }),
    )
    .unwrap();
    assert_eq!(updated["body"], "tiếng việt");

    let hits = invoke(&webview, "search", json!({ "query": "tieng", "limit": 10 })).unwrap();
    assert_eq!(hits.as_array().unwrap().len(), 1);
    assert_eq!(hits[0]["id"], note_id);

    let listed = invoke(&webview, "note_list", json!({ "folderId": folder_id })).unwrap();
    assert_eq!(listed.as_array().unwrap().len(), 1);

    let missing = invoke(&webview, "note_get", json!({ "id": 9999 })).unwrap_err();
    assert_eq!(missing["code"], "not_found");

    invoke(&webview, "vault_lock", json!({})).unwrap();
    let status = invoke(&webview, "vault_status", json!({})).unwrap();
    assert_eq!(status["locked"], true);
    let err = invoke(&webview, "note_get", json!({ "id": note_id })).unwrap_err();
    assert_eq!(err["code"], "locked");

    let reopened = invoke(
        &webview,
        "vault_open",
        json!({ "dir": dir_str, "password": "pw" }),
    )
    .unwrap();
    assert_eq!(reopened["dir"], dir_str);
    let note_again = invoke(&webview, "note_get", json!({ "id": note_id })).unwrap();
    assert_eq!(note_again["body"], "tiếng việt");
}
