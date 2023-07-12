use crate::cookie::Cookie;
use crate::Result;
use http::HeaderMap;
use reqwest::{Method, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::{to_string_pretty, Value};

pub struct Response {
	request_method: Method,
	request_url: String,

	status: StatusCode,
	header_map: HeaderMap,

	client_cookies: Vec<Cookie>,

	/// Cookies from the response
	cookies: Vec<Cookie>,
	body: Body,
}

enum Body {
	Json(Value),
	Text(String),
	Other,
}

impl Response {
	pub(crate) async fn from_reqwest_response(
		request_method: Method,
		request_url: String,
		client_cookies: Vec<Cookie>,
		mut res: reqwest::Response,
	) -> Result<Response> {
		let status = res.status();

		// Cookies from response
		let cookies: Vec<Cookie> = res.cookies().map(Cookie::from).collect();

		// Move the headers into a new HeaderMap
		let headers = res.headers_mut().drain().filter_map(|(n, v)| n.map(|n| (n, v)));
		let header_map = HeaderMap::from_iter(headers);

		// Capture the body
		let ct = header_map.get("content-type").and_then(|v| v.to_str().ok());
		let body = if let Some(ct) = ct {
			if ct.starts_with("application/json") {
				Body::Json(res.json::<Value>().await?)
			} else if ct.starts_with("text/") {
				Body::Text(res.text().await?)
			} else {
				Body::Other
			}
		} else {
			Body::Other
		};

		Ok(Response {
			client_cookies,
			request_method,
			request_url,
			status,
			header_map,
			cookies,
			body,
		})
	}
}

impl Response {
	// region:    --- Print Methods
	pub async fn print(&self) -> Result<()> {
		self.inner_print(true).await
	}

	pub async fn print_no_body(&self) -> Result<()> {
		self.inner_print(false).await
	}

	/// NOTE: For now, does not need to be async, but keeping the option of using async for later.
	async fn inner_print(&self, body: bool) -> Result<()> {
		println!();
		println!("=== Response for {} {}", self.request_method, self.request_url);
		println!("=> {:<15}: {}", "Status", self.status);
		// Print the response headers.
		println!("=> {:<15}:", "Headers");
		for (n, v) in self.header_map.iter() {
			println!("   {n}: {v:?}");
		}

		// Print the cookie_store
		if !self.cookies.is_empty() {
			println!("=> {:<15}:", "Response Cookies");
			for c in self.cookies.iter() {
				println!("   {}: {}", c.name, c.value);
			}
		}

		// Print the cookie_store
		if !self.client_cookies.is_empty() {
			println!("=> {:<15}:", "Client Cookies");
			for c in self.client_cookies.iter() {
				println!("   {}: {}", c.name, c.value);
			}
		}

		if body {
			// Print the body (json pretty print if json type)
			println!("=> {:<15}:", "Response Body");
			match &self.body {
				Body::Json(val) => println!("{}", to_string_pretty(val)?),
				Body::Text(val) => println!("{val}"),
				_ => (),
			}
		}

		println!("===\n");
		Ok(())
	}
	// endregion: --- Print Methods

	// region:    --- Headers
	pub fn header_all(&self, name: &str) -> Vec<String> {
		self.header_map
			.get_all(name)
			.iter()
			.filter_map(|v| v.to_str().map(|v| v.to_string()).ok())
			.collect()
	}

	pub fn header(&self, name: &str) -> Option<String> {
		self.header_map.get(name).and_then(|v| v.to_str().map(|v| v.to_string()).ok())
	}
	// endregion: --- Headers

	// region:    --- Response Cookie
	/// Return the cookie that has been set for this http response.
	pub fn res_cookie(&self, name: &str) -> Option<&Cookie> {
		self.cookies.iter().find(|c| c.name == name)
	}

	/// Return the cookie value that has been set for this http response.
	pub fn res_cookie_value(&self, name: &str) -> Option<String> {
		self.cookies.iter().find(|c| c.name == name).map(|c| c.value.clone())
	}
	// endregion: --- Response Cookie

	// region:    --- Client Cookies
	/// Return the client httpc-test Cookie for a given name.
	/// Note: The response.client_cookies are the captured client cookies
	///       at the time of the response.
	pub fn client_cookie(&self, name: &str) -> Option<&Cookie> {
		self.client_cookies.iter().find(|c| c.name == name)
	}

	/// Return the client cookie value as String for a given name.
	/// Note: The response.client_cookies are the captured client cookies
	///       at the time of the response.
	pub fn client_cookie_value(&self, name: &str) -> Option<String> {
		self.client_cookies.iter().find(|c| c.name == name).map(|c| c.value.clone())
	}
	// endregion: --- Client Cookies

	// region:    --- Body
	pub fn json_body(&self) -> Result<Value> {
		match &self.body {
			Body::Json(val) => Ok(val.clone()),
			_ => Err(crate::Error::Static("No json body")),
		}
	}

	pub fn text_body(&self) -> Result<String> {
		match &self.body {
			Body::Text(val) => Ok(val.clone()),
			_ => Err(crate::Error::Static("No text body")),
		}
	}

	pub fn json_body_as<T>(&self) -> Result<T>
	where
		T: DeserializeOwned,
	{
		self.json_body()
			.and_then(|val| serde_json::from_value::<T>(val).map_err(|e| crate::Error::Generic(e.to_string())))
	}
	// endregion: --- Body
}
