pub fn reset_password_email(reset_link: &str) -> String {
    include_str!("../templates/email/reset_password_template.html")
        .replace("{{reset_link}}", reset_link)
}
