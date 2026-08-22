pub async fn login_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> Result<(CookieJar, Redirect), AppError> {
    let unauth_user = UnauthenticatedUser::new(form.username, form.password);
    let user = unauth_user.authenticate(&repository).await?;

    // Passa a chave secreta configurada no AppState
    let token = user.auth_token(state.jwt_secret.as_bytes())?;

    let cookie = Cookie::build(("token", token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax);

    Ok((jar.add(cookie), Redirect::to("/assets")))
}
