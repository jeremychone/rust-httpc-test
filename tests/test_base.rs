//! Note: http rest endpoints: https://sqa.stackexchange.com/questions/47097/free-sites-for-testing-post-rest-api-calls
//!       Using: https://jsonplaceholder.typicode.com/
//!
//! Some is better than none: At this point very basic tests.

use anyhow::Result;
use serde_json::json;

const TYPICODE_END_POINT: &str = "https://jsonplaceholder.typicode.com";

#[tokio::test]
async fn test_base_typicode_get() -> Result<()> {
	// -- Setup
	let hc = httpc_test::new_client(TYPICODE_END_POINT)?;

	// -- Exec
	let res = hc.do_get("/posts/1").await?;
	let status = res.status();
	let res = res.json_body()?.to_string();

	// -- Check
	assert_eq!(status, 200);
	assert!(res.contains(r#""body":"quia et suscipit\nsuscipit"#));
	assert!(res.contains(r#""id":1"#));
	assert!(res.contains(r#""userId":1"#));

	Ok(())
}

#[tokio::test]
async fn test_base_typicode_post() -> Result<()> {
	// -- Setup
	let hc = httpc_test::new_client(TYPICODE_END_POINT)?;

	// -- Exec
	let res = hc
		.do_post(
			"/posts",
			json!(
					{
			  "userId": 1,
			  "title": "test_base_typicode_post - title-01",
			  "body": "test_base_typicode_post - body-01"
			}
				),
		)
		.await?;
	let status = res.status();
	let res = res.json_body()?;

	// -- Check
	assert_eq!(status, 201); // success, resource created.
	assert_eq!(
		res.to_string(),
		r#"{"body":"test_base_typicode_post - body-01","id":101,"title":"test_base_typicode_post - title-01","userId":1}"#
	);

	Ok(())
}
