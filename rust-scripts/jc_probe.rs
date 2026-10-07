#!/usr/bin/env rust-script
//! name: jc_probe
//! description: Link-and-parse probe for the renamed jc facade (jc_parse over csv input, jc_parser_names count).
//! version: 1.0.0
//! keywords: jc, parse, probe, csv

fn main() {
	let value = pi_rust_lib::jc_parse("csv", "a,b\n1,2").expect("csv parse");
	let names = pi_rust_lib::jc_parser_names();
	pi_rust_lib::report::success(
		"jc_probe",
		pi_rust_lib::serde_json::json!({ "csv_a": value[0]["a"], "parser_count": names.len() }),
		"jc facade linked and parsing; suite pieces run by name",
	)
	.unwrap();
}
