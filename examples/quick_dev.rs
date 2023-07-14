//! Note: http rest endpoints: https://sqa.stackexchange.com/questions/47097/free-sites-for-testing-post-rest-api-calls
//!       Using: https://jsonplaceholder.typicode.com/

use anyhow::Result;

const END_POINT: &str = "https://jsonplaceholder.typicode.com";

#[tokio::main]
async fn main() -> Result<()> {
	let hc = httpc_test::new_client(END_POINT)?;

	let req = hc.do_get("/posts/1").await?;
	req.print().await?;

	let req = hc.do_get("/todos/1").await?;
	req.print().await?;

	Ok(())
}
