mod common;

use common::setup_test_db;
use serde_json::Value;

#[tokio::test]
async fn test_json_output_format() {
    let db = setup_test_db().await;

    // Add todos
    doob::commands::add::execute(&db, vec!["Test 1".to_string()], Some(1), None, None, Some("urgent".to_string())).await.unwrap();

    // Get todos
    let todos = doob::commands::list::execute(&db, None, None, None).await.unwrap();

    // Format as JSON
    let json = doob::output::json::format_todos(&todos);

    // Parse to verify structure
    let parsed: Value = serde_json::from_str(&json).expect("Invalid JSON");
    assert_eq!(parsed["count"], 1);
    assert!(parsed["todos"].is_array());
    assert_eq!(parsed["todos"][0]["content"], "Test 1");
    assert_eq!(parsed["todos"][0]["priority"], 1);
    assert_eq!(parsed["todos"][0]["tags"][0], "urgent");
}
