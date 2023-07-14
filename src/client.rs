use crate::cookie::{from_tower_cookie_deref, Cookie};
use crate::{Response, Result};
use http::Method;
use reqwest_cookie_store::CookieStoreMutex;
use serde::de::DeserializeOwned;
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
	// region:    --- http calls returning httpc-test Response
	pub async fn do_get(&self, url: &str) -> Result<Response> {
		let url = self.compose_url(url);
		let reqwest_res = self.client.get(&url).send().await?;
		self.capture_response(Method::GET, url, reqwest_res).await
	}

	pub async fn do_delete(&self, url: &str) -> Result<Response> {
		let url = self.compose_url(url);
		let reqwest_res = self.client.delete(&url).send().await?;
		self.capture_response(Method::DELETE, url, reqwest_res).await
	}

	pub async fn do_post(&self, url: &str, content: impl Into<PostContent>) -> Result<Response> {
		self.do_push(Method::POST, url, content.into()).await
	}

	pub async fn do_put(&self, url: &str, content: impl Into<PostContent>) -> Result<Response> {
		self.do_push(Method::PUT, url, content.into()).await
	}

	pub async fn do_patch(&self, url: &str, content: impl Into<PostContent>) -> Result<Response> {
		self.do_push(Method::PATCH, url, content.into()).await
	}
	// endregion: --- http calls returning httpc-test Response

	// region:    --- http calls returning typed Deserialized body
	pub async fn get<T>(&self, url: &str) -> Result<T>
	where
		T: DeserializeOwned,
	{
		self.do_get(url).await.and_then(|res| res.json_body_as::<T>())
	}

	pub async fn delete<T>(&self, url: &str) -> Result<T>
	where
		T: DeserializeOwned,
	{
		self.do_delete(url).await.and_then(|res| res.json_body_as::<T>())
	}

	pub async fn post<T>(&self, url: &str, content: impl Into<PostContent>) -> Result<T>
	where
		T: DeserializeOwned,
	{
		self.do_post(url, content).await.and_then(|res| res.json_body_as::<T>())
	}

	pub async fn put<T>(&self, url: &str, content: impl Into<PostContent>) -> Result<T>
	where
		T: DeserializeOwned,
	{
		self.do_put(url, content).await.and_then(|res| res.json_body_as::<T>())
	}

	pub async fn patch<T>(&self, url: &str, content: impl Into<PostContent>) -> Result<T>
	where
		T: DeserializeOwned,
	{
		self.do_patch(url, content).await.and_then(|res| res.json_body_as::<T>())
	}
	// endregion: --- http calls returning typed Deserialized body

	// region:    --- Cookie
	pub fn cookie(&self, name: &str) -> Option<Cookie> {
		let cookie_store = self.cookie_store.lock().unwrap();
		let cookie = cookie_store
			.iter_any()
			.find(|c| c.name() == name)
			.map(|c| from_tower_cookie_deref(c));

		cookie
	}

	pub fn cookie_value(&self, name: &str) -> Option<String> {
		self.cookie(name).map(|c| c.value)
	}
	// endregion: --- Cookie

	// region:    --- Client Privates

	/// Internal implementation for POST, PUT, PATCH
	async fn do_push(&self, method: Method, url: &str, content: PostContent) -> Result<Response> {
		let url = self.compose_url(url);
		let reqwest_res = match content {
			PostContent::Json(value) => self.client.post(&url).json(&value).send().await?,
			PostContent::Text { content_type, body } => {
				self.client
					.post(&url)
					.body(body)
					.header("content-type", content_type)
					.send()
					.await?
			}
		};

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
		let client_cookies: Vec<Cookie> = cookie_store.iter_any().map(|c| from_tower_cookie_deref(c)).collect();

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

// region:    --- Post Body
pub enum PostContent {
	Json(Value),
	Text { body: String, content_type: &'static str },
}
impl From<Value> for PostContent {
	fn from(val: Value) -> Self {
		PostContent::Json(val)
	}
}
impl From<String> for PostContent {
	fn from(val: String) -> Self {
		PostContent::Text {
			content_type: "text/plain",
			body: val,
		}
	}
}
impl From<&String> for PostContent {
	fn from(val: &String) -> Self {
		PostContent::Text {
			content_type: "text/plain",
			body: val.to_string(),
		}
	}
}

impl From<&str> for PostContent {
	fn from(val: &str) -> Self {
		PostContent::Text {
			content_type: "text/plain",
			body: val.to_string(),
		}
	}
}

impl From<(String, &'static str)> for PostContent {
	fn from((body, content_type): (String, &'static str)) -> Self {
		PostContent::Text { body, content_type }
	}
}

impl From<(&str, &'static str)> for PostContent {
	fn from((body, content_type): (&str, &'static str)) -> Self {
		PostContent::Text {
			body: body.to_string(),
			content_type,
		}
	}
}

// endregion: --- Post Body

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
