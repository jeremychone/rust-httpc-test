use crate::cookie::Cookie;
use crate::{Error, Result};
use reqwest::{Method, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::{to_string_pretty, Value};

#[allow(unused)]
#[cfg(feature = "color-output")]
use colored::*;
#[allow(unused)]
#[cfg(feature = "color-output")]
use colored_json::prelude::*;
use reqwest::header::HeaderMap;

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

#[allow(unused)]
#[cfg(feature = "color-output")]
fn get_status_color(status: &StatusCode) -> Color {
	match status.as_u16() {
		200..=299 => Color::Green,  // 2xx status codes are successful so we color them green
		300..=399 => Color::Blue,   // 3xx status codes are for redirection so we color them blue
		400..=499 => Color::Yellow, // 4xx status codes are client errors so we color them yellow
		500..=599 => Color::Red,    // 5xx status codes are server errors so we color them red
		_ => Color::White,          // Anything else we just color white
	}
}

#[allow(unused)]
#[cfg(feature = "color-output")]
fn get_method_background(method: &Method) -> Color {
	match *method {
		Method::GET => Color::TrueColor { r: 223, g: 231, b: 238 },
		Method::POST => Color::TrueColor { r: 220, g: 233, b: 228 },
		Method::PUT => Color::TrueColor { r: 238, g: 229, b: 218 },
		Method::DELETE => Color::TrueColor { r: 238, g: 219, b: 219 },
		_ => Color::White,
	}
}

#[allow(unused)]
#[cfg(feature = "color-output")]
fn get_method_color(method: &Method) -> Color {
	match *method {
		Method::GET => Color::TrueColor { r: 92, g: 166, b: 241 },
		Method::POST => Color::TrueColor { r: 59, g: 184, b: 127 },
		Method::PUT => Color::TrueColor { r: 239, g: 153, b: 46 },
		Method::DELETE => Color::TrueColor { r: 236, g: 59, b: 59 },
		_ => Color::White,
	}
}

#[allow(unused)]
#[cfg(feature = "color-output")]
fn split_and_color_url(url: &str) -> String {
	let url_struct = url::Url::parse(url).unwrap();
	let path = url_struct.path();
	format!("{}", path.purple())
}

#[allow(unused)]
#[cfg(feature = "color-output")]
fn format_method(method: &Method) -> String {
	format!(" {:<10}", method.to_string())
}

#[allow(unused)]
#[cfg(feature = "color-output")]
const INDENTATION: u8 = 12;

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
	#[allow(unused)]
	#[cfg(feature = "color-output")]
	async fn inner_print(&self, body: bool) -> Result<()> {
		let method_color = get_method_color(&self.request_method);
		let method_background = get_method_background(&self.request_method);
		let colored_url = split_and_color_url(&self.request_url);
		let status_color = get_status_color(&self.status);
		println!();
		println!(
			"{}: {}",
			format_method(&self.request_method)
				.bold()
				.color(method_color)
				.on_truecolor(50, 50, 50),
			colored_url
		);
		println!(
			" {:<9} : {} {}",
			"Status".blue(),
			self.status.as_str().bold().color(status_color).on_black(),
			self.status.canonical_reason().unwrap_or_default().color(status_color)
		);

		// Print the response headers.
		println!(" {:<9} :", "Headers".blue());

		for (n, v) in self.header_map.iter() {
			println!("    {}: {}", n.to_string().yellow(), v.to_str().unwrap_or_default());
		}

		// Print the cookie_store
		if !self.cookies.is_empty() {
			println!(" {}:", "Response Cookies".blue());
			for c in self.cookies.iter() {
				println!("    {}: {}", c.name.yellow(), c.value.bold());
			}
		}

		// Print the cookie_store
		if !self.client_cookies.is_empty() {
			println!(" {}:", "Client Cookies".blue());
			for c in self.client_cookies.iter() {
				println!("    {}: {}", c.name.yellow(), c.value.bold());
			}
		}

		if body {
			// Print the body (json pretty print if json type)
			println!("{}:", "Response Body".blue());
			match &self.body {
				Body::Json(val) => println!("{}", to_string_pretty(val)?.to_colored_json_auto()?),
				Body::Text(val) => println!("    {}", val.color(status_color)),
				_ => (),
			}
		}

		println!("\n");
		Ok(())
	}

	#[cfg(not(feature = "color-output"))]
	async fn inner_print(&self, body: bool) -> Result<()> {
		println!();
		println!("=== Response for {} {}", self.request_method, &self.request_url);

		println!(
			"=> {:<15}: {} {}",
			"Status",
			self.status.as_str(),
			self.status.canonical_reason().unwrap_or_default()
		);

		// Print the response headers.
		println!("=> {:<15}:", "Headers");

		for (n, v) in self.header_map.iter() {
			println!("   {}: {}", n, v.to_str().unwrap_or_default());
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
				Body::Text(val) => println!("{}", val),
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

	// region:    --- Status Code
	/// Return the Response status code
	pub fn status(&self) -> StatusCode {
		self.status
	}
	// endregion: --- Status Code

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
			_ => Err(Error::Static("No json body")),
		}
	}

	pub fn text_body(&self) -> Result<String> {
		match &self.body {
			Body::Text(val) => Ok(val.clone()),
			_ => Err(Error::Static("No text body")),
		}
	}

	pub fn json_value<T>(&self, pointer: &str) -> Result<T>
	where
		T: DeserializeOwned,
	{
		let Body::Json(body) = &self.body else {
			return Err(Error::Static("No json body"));
		};

		let value = body.pointer(pointer).ok_or_else(|| Error::NoJsonValueFound {
			json_pointer: pointer.to_string(),
		})?;

		Ok(serde_json::from_value::<T>(value.clone())?)
	}

	pub fn json_body_as<T>(&self) -> Result<T>
	where
		T: DeserializeOwned,
	{
		self.json_body()
			.and_then(|val| serde_json::from_value::<T>(val).map_err(Error::SerdeJson))
	}
	// endregion: --- Body
}
