use std::time::{Duration, SystemTime};

#[derive(Debug)]
pub struct Cookie {
	pub name: String,
	pub value: String,
	pub http_only: bool,
	pub secure: bool,
	pub same_site_lax: bool,
	pub same_site_strict: bool,
	pub path: Option<String>,
	pub max_age: Option<Duration>,
	pub expires: Option<SystemTime>,
}

pub fn from_tower_cookie_deref(val: &cookie::Cookie) -> Cookie {
	let max_age: Option<Duration> = val
		.max_age()
		.map(|d| d.try_into().expect("time::Duration into std::time::Duration"));
	let expires = match val.expires() {
		Some(cookie::Expiration::DateTime(offset)) => Some(SystemTime::from(offset)),
		None | Some(cookie::Expiration::Session) => None,
	};

	Cookie {
		name: val.name().to_string(),
		value: val.value().to_string(),
		http_only: val.http_only().unwrap_or(false),
		secure: val.secure().unwrap_or(false),
		same_site_lax: matches!(val.same_site(), Some(cookie::SameSite::Lax)),
		same_site_strict: matches!(val.same_site(), Some(cookie::SameSite::Strict)),
		path: val.path().map(String::from),
		max_age,
		expires,
	}
}

impl From<&cookie::Cookie<'_>> for Cookie {
	fn from(val: &cookie::Cookie) -> Self {
		let max_age: Option<Duration> = val
			.max_age()
			.map(|d| d.try_into().expect("time::Duration into std::time::Duration"));
		let expires = match val.expires() {
			Some(cookie::Expiration::DateTime(offset)) => Some(SystemTime::from(offset)),
			None | Some(cookie::Expiration::Session) => None,
		};

		Cookie {
			name: val.name().to_string(),
			value: val.value().to_string(),
			http_only: val.http_only().unwrap_or(false),
			secure: val.secure().unwrap_or(false),
			same_site_lax: matches!(val.same_site(), Some(cookie::SameSite::Lax)),
			same_site_strict: matches!(val.same_site(), Some(cookie::SameSite::Strict)),
			path: val.path().map(String::from),
			max_age,
			expires,
		}
	}
}

impl From<reqwest::cookie::Cookie<'_>> for Cookie {
	fn from(val: reqwest::cookie::Cookie<'_>) -> Self {
		Cookie {
			name: val.name().to_string(),
			value: val.value().to_string(),
			http_only: val.http_only(),
			secure: val.secure(),
			same_site_lax: val.same_site_lax(),
			same_site_strict: val.same_site_strict(),
			path: val.path().map(String::from),
			max_age: val.max_age(),
			expires: val.expires(),
		}
	}
}

impl From<&reqwest::cookie::Cookie<'_>> for Cookie {
	fn from(val: &reqwest::cookie::Cookie) -> Self {
		Cookie {
			name: val.name().to_string(),
			value: val.value().to_string(),
			http_only: val.http_only(),
			secure: val.secure(),
			same_site_lax: val.same_site_lax(),
			same_site_strict: val.same_site_strict(),
			path: val.path().map(String::from),
			max_age: val.max_age(),
			expires: val.expires(),
		}
	}
}
