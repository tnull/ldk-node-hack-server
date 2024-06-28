extern crate prost_build;

use std::{env, fs, path::Path};

fn main() {
	generate_protos();
}

fn generate_protos() {
	let mut config = prost_build::Config::new();
	config.protoc_arg("--experimental_allow_proto3_optional");
	config.compile_protos(&["src/proto/ldk_server_hack.proto"], &["src/"])
		.expect("protobuf compilation failed");
	println!("sss {}", &env::var("OUT_DIR").unwrap());
	let from_path = Path::new(&env::var("OUT_DIR").unwrap()).join("ldk_server_hack.rs");
	fs::copy(from_path, "src/lib.rs").unwrap();
}
