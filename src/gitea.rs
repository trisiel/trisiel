use serde::{Deserialize, Serialize};

/// A user.
/// https://try.gitea.io/api/swagger#model-User
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub avatar_url: String,
    pub created: String,
    pub email: String,
    pub full_name: String,
    pub id: i64,
    pub is_admin: bool,
    pub language: String,
    pub last_login: String,
    pub login: String,
}

pub fn user(token: String) -> std::io::Result<User> {
    let resp =
        ureq::get("https://tulpa.dev/api/v1/user")
        .set("Authorization", &format!("bearer {}", token))
        .call();
    if !resp.ok() {
        todo!("error here");
    }
    let user: User = resp.into_json_deserialize()?;
    Ok(user)
}
