
Minimalistic HTTP Client Test Utilities.

- Built on top of [reqwest](https://crates.io/crates/reqwest)
- Still under development `0.0.x`
- Optimized for testing convenience, not for performance.
- Do not use for production code, just for unit or integration tests. 
- For production code (apps, services, ...) use the underlying [reqwest](https://crates.io/crates/reqwest) library and its utilities.


> WARNING - API might ching during `0.0.x`

```rs
use anyhow::Result;
use serde_json::json;

#[tokio::test]
async fn test_simple_base() -> httpc_test::Result<()> {
	// Create a new httpc test client with a base URL (will be prefixed for all calls)
	// The client will have a cookie_store.
	let hc = httpc_test::new_client("http://localhost:8080")?;

	// Simple get
	let res = hc.do_get("/hello").await?;
	// Pretty print the result (status, headers, response cookies, client cookies, body)
	res.print().await?;

	// Another get
	let res = hc.do_get("/context.rs").await?;
	// Pretty print but do not print the body 
	res.print_no_body().await?;

	// Do a json post
	// In this case, server might do a Set-Cookie for the auth-token, 
	// and the client cookie will be updated.
	let res = hc
		.do_post_json(
			"/api/login",
			json!({
				"username": "admin",
				"pwd": "welcome"
			}),
		)
		.await?;
	res.print().await?;

	// Another post (with the cookie store updated from the login request above )
	let res = hc
		.do_post_json(
			"/api/tickets",
			json!({
				"subject": "ticket 01"
			}),
		)
		.await?;
	res.print().await?;


	// Another post
	let res = hc.do_get("/api/tickets").await?;
	res.print().await?;

	Ok(())
}
```