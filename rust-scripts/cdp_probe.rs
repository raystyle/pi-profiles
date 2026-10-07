#!/usr/bin/env rust-script
//! name: cdp_probe
//! description: Compile-link probe proving the vendored cdp crate resolves with no external browse_rs checkout.
//! version: 1.0.0
//! keywords: cdp, vendor, probe

fn main() {
	let _ = cdp::Session::new();
	pi_rust_lib::report::success(
		"cdp_probe",
		pi_rust_lib::serde_json::json!({ "session_type": std::any::type_name::<cdp::Session>() }),
		"vendored cdp crate compiled and linked; no external checkout needed",
	)
	.unwrap();
}
