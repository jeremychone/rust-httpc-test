use crate::cookie::Cookie;
use crate::Result;
use http::HeaderMap;
use reqwest::{Method, StatusCode};
use serde_json::{to_string_pretty, Value};

pub struct Response {
	request_method: Method,
	request_url: String,

	status: StatusCode,
	headers: HeaderMap,

	client_cookies: Vec<Cookie>,

	/// Cookies from the response
	cookies: Vec<Cookie>,
	body: Body,
}

impl From<&reqwest::cookie::Cookie<'_>> for Cookie {
	fn from(val: &reqwest::cookie::Cookie) -> Self {
		Cookie {
			name: val.name().to_string(),
			value: val.value().to_string(),
		}
	}
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
		let cookies: Vec<Cookie> = res
			.cookies()
			.map(|c| Cookie {
				name: c.name().to_string(),
				value: c.value().to_string(),
			})
			.collect();

		// Move the headers into a new HeaderMap
		let headers = res.headers_mut().drain().filter_map(|(n, v)| n.map(|n| (n, v)));
		let headers = HeaderMap::from_iter(headers);

		// Capture the body
		let ct = headers.get("content-type").and_then(|v| v.to_str().ok());
		let body = if let Some(ct) = ct {
			if ct == "application/json" {
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
			headers,
			cookies,
			body,
		})
	}
}

impl Response {
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
		for (n, v) in self.headers.iter() {
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
}
