use serde_derive::Serialize;

#[derive(Serialize)]
pub struct Settings {
	pub ingress: IngressSettings,
}

#[derive(Serialize)]
pub struct IngressSettings {
	pub user_codes: Vec<u64>,
}
