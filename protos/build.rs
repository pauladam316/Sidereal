fn main() {
    tonic_prost_build::compile_protos("protos/planetarium.proto").unwrap();
}
