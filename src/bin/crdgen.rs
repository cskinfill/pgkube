use kube::CustomResourceExt;

fn main() {
    print!("{}", serde_yaml::to_string(&pgkube::Integration::crd()).unwrap())
}