//! Project-level lib for rs pieces (`pi_project_example`).
//!
//! Repeated logic across pieces graduates here; pieces reference
//! `pi_project_example::...` and the executor injects the path dependency at
//! materialization time (same mechanism as `pi_rust_lib`). Promotion into the
//! bundled `pi_rust_lib` stays a deliberate curation act.

/// Percent-encode every byte outside the unreserved set (RFC 3986):
/// alphanumeric plus `-._~`. Everything else becomes `%XX` - the encoding the
/// marathon pieces need for URL payloads and cookie values.
pub fn percent_encode(s: &str) -> String {
	let mut out = String::new();
	for b in s.bytes() {
		if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
			out.push(b as char);
		} else {
			out.push_str(&format!("%{b:02X}"));
		}
	}
	out
}

/// Encode pairs as an `application/x-www-form-urlencoded` body: reserved
/// bytes percent-encoded and spaces sent as `+`, pairs joined with `&`.
pub fn form_encode(pairs: &[(String, String)]) -> String {
	pairs
		.iter()
		.map(|(k, v)| format!("{}={}", form_component(k), form_component(v)))
		.collect::<Vec<_>>()
		.join("&")
}

fn form_component(s: &str) -> String {
	let mut out = String::new();
	for b in s.bytes() {
		match b {
			b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
			b' ' => out.push('+'),
			_ => out.push_str(&format!("%{b:02X}")),
		}
	}
	out
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn percent_encodes_reserved() {
		assert_eq!(percent_encode("a b/c~"), "a%20b%2Fc~");
	}

	#[test]
	fn form_encodes_spaces_as_plus() {
		assert_eq!(form_encode(&[("a b".into(), "c/d".into())]), "a+b=c%2Fd");
	}
}
