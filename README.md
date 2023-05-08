
Minimalistic HTTP Client Test Utilities.

- Built on top of [reqwest](https://crates.io/crates/reqwest).
- Optimized for testing convenience, not for performance. 
- Do not use for production/application code, just for testing.
- For production code (apps, services, ...) use the underlying [reqwest](https://crates.io/crates/reqwest) library and its utilities.

# Thanks

- Thanks to [@defic](https://github.com/defic) for the type client `get/post/put/patch/delete` and the response `body...` apis.


# Example

```rs
use anyhow::Result;
use serde_json::{json, Value};

#[tokio::test]
async fn test_simple_base() -> httpc_test::Result<()> {
	// Create a new httpc test client with a base URL (will be prefixed for all calls)
	// The client will have a cookie_store.
	let hc = httpc_test::new_client("http://localhost:8080")?;


	//// do_get, do_post, do_put, do_patch, do_delete return a httpc_test::Response

	// Simple do_get
	let res = hc.do_get("/hello").await?; // httpc_test::Response 
	// Pretty print the result (status, headers, response cookies, client cookies, body)
	res.print().await?;

	let auth_token = res.res_cookie_value("auth-token"); // Option<String>
	let content_type = res.header("content_type"); // Option<&String>

	// Another do_get
	let res = hc.do_get("/context.rs").await?;
	// Pretty print but do not print the body 
	res.print_no_body().await?;


	//// get, post, put, patch, delete return a DeserializeOwned

	// a get (return a Deserialized)
	let json_value = hc.get::<Value>("/api/tickets").await?;

	// Another post (with the cookie store updated from the login request above )
	let res = hc
		.do_post(
			"/api/tickets",
			json!({
				"subject": "ticket 01"
			}),
		)
		.await?;
	res.print().await?;


	// Post with text content and specific content type
	let res = hc
		.do_post(
			"/api/tickets",
			(r#"{
				"subject": "ticket bb"
			}
			"#, 
			"application/json"),
		)
		.await?;
	res.print().await?;

	// Same woth do_patch, do_put.


	Ok(())
}
```

<br /><br />
[This GitHub repo](https://github.com/jeremychone/rust-httpc-test)