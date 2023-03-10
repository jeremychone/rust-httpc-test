use crate::cookie::Cookie;
use crate::{Response, Result};
use http::Method;
use reqwest_cookie_store::CookieStoreMutex;
use serde_json::Value;
use std::sync::Arc;

pub struct Client {
	base_url: Option<String>,
	cookie_store: Arc<CookieStoreMutex>,
	client: reqwest::Client,
}

pub fn new_client(base_url: impl Into<BaseUrl>) -> Result<Client> {
	let base_url = base_url.into().into();
	let cookie_store = Arc::new(CookieStoreMutex::default());
	let client = reqwest::Client::builder().cookie_provider(cookie_store.clone()).build()?;

	Ok(Client {
		base_url,
		cookie_store,
		client,
	})
}

impl Client {
	pub async fn do_get(&self, url: &str) -> Result<Response> {
		let url = self.compose_url(url);
		let reqwest_res = self.client.get(&url).send().await?;
		// capture the cookies at the time of the

		self.capture_response(Method::GET, url, reqwest_res).await
	}

	pub async fn do_delete(&self, url: &str) -> Result<Response> {
		let url = self.compose_url(url);
		let reqwest_res = self.client.delete(&url).send().await?;
		// capture the cookies at the time of the

		self.capture_response(Method::DELETE, url, reqwest_res).await
	}

	pub async fn do_post_json(&self, url: &str, body: Value) -> Result<Response> {
		self.do_push_json(Method::POST, url, body).await
	}

	pub async fn do_put_json(&self, url: &str, body: Value) -> Result<Response> {
		self.do_push_json(Method::PUT, url, body).await
	}

	pub async fn do_patch_json(&self, url: &str, body: Value) -> Result<Response> {
		self.do_push_json(Method::PATCH, url, body).await
	}

	// region:    --- Client Privates
	/// Internal implementation for POST, PUT, PATCH
	async fn do_push_json(&self, method: Method, url: &str, body: Value) -> Result<Response> {
		let url = self.compose_url(url);
		let reqwest_res = self.client.post(&url).json(&body).send().await?;

		self.capture_response(method, url, reqwest_res).await
	}

	#[allow(clippy::await_holding_lock)] // ok for testing lib
	async fn capture_response(
		&self,
		request_method: Method,
		url: String,
		reqwest_res: reqwest::Response,
	) -> Result<Response> {
		// Note: For now, we will unwrap/panic if fail.
		//       Might handle this differently in the future.
		let cookie_store = self.cookie_store.lock().unwrap();

		// Cookies from the client store
		let client_cookies: Vec<Cookie> = cookie_store
			.iter_any()
			.map(|c| Cookie {
				name: c.name().to_string(),
				value: c.value().to_string(),
			})
			.collect();

		Response::from_reqwest_response(request_method, url, client_cookies, reqwest_res).await
	}

	fn compose_url(&self, url: &str) -> String {
		match &self.base_url {
			Some(base_url) => format!("{base_url}{url}"),
			None => url.to_string(),
		}
	}
	// endregion: --- Client Privates
}

// region:    --- BaseUrl
pub struct BaseUrl(Option<String>);

impl From<&str> for BaseUrl {
	fn from(val: &str) -> Self {
		BaseUrl(Some(val.to_string()))
	}
}
impl From<String> for BaseUrl {
	fn from(val: String) -> Self {
		BaseUrl(Some(val))
	}
}
impl From<&String> for BaseUrl {
	fn from(val: &String) -> Self {
		BaseUrl(Some(val.to_string()))
	}
}
impl From<BaseUrl> for Option<String> {
	fn from(val: BaseUrl) -> Self {
		val.0
	}
}
impl From<Option<String>> for BaseUrl {
	fn from(val: Option<String>) -> Self {
		BaseUrl(val)
	}
}
// endregion: --- BaseUrl
