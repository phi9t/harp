# Exact 0.159.3 primary excerpts
Source commit: 01fc69f4026735edfdf6789820549727a4867b11. Public source only; no local auth contents.

## codex-rs/model-provider-info/src/lib.rs:77-77
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/model-provider-info/src/lib.rs#L77
```rust
77: pub const CHATGPT_CODEX_BASE_URL: &str = "https://chatgpt.com/backend-api/codex";
```

## codex-rs/model-provider-info/src/lib.rs:420-438
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/model-provider-info/src/lib.rs#L420
```rust
420:     pub fn to_api_provider(&self, auth_mode: Option<AuthMode>) -> CodexResult<ApiProvider> {
421:         let default_base_url = if matches!(
422:             auth_mode,
423:             Some(
424:                 AuthMode::Chatgpt
425:                     | AuthMode::ChatgptAuthTokens
426:                     | AuthMode::Headers
427:                     | AuthMode::AgentIdentity
428:                     | AuthMode::PersonalAccessToken
429:             )
430:         ) {
431:             CHATGPT_CODEX_BASE_URL
432:         } else {
433:             "https://api.openai.com/v1"
434:         };
435:         let base_url = self
436:             .base_url
437:             .clone()
438:             .unwrap_or_else(|| default_base_url.to_string());
```

## codex-rs/model-provider-info/src/lib.rs:520-557
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/model-provider-info/src/lib.rs#L520
```rust
520:         ModelProviderInfo {
521:             name: OPENAI_PROVIDER_NAME.into(),
522:             base_url,
523:             model_catalog_url: None,
524:             env_key: None,
525:             env_key_instructions: None,
526:             experimental_bearer_token: None,
527:             auth: None,
528:             gateway_oauth: None,
529:             aws: None,
530:             wire_api: WireApi::Responses,
531:             query_params: None,
532:             http_headers: Some(
533:                 [("version".to_string(), env!("CARGO_PKG_VERSION").into())]
534:                     .into_iter()
535:                     .collect(),
536:             ),
537:             env_http_headers: Some(
538:                 [
539:                     (
540:                         "OpenAI-Organization".to_string(),
541:                         "OPENAI_ORGANIZATION".to_string(),
542:                     ),
543:                     ("OpenAI-Project".to_string(), "OPENAI_PROJECT".to_string()),
544:                 ]
545:                 .into_iter()
546:                 .collect(),
547:             ),
548:             // Use global defaults for retry/timeout unless overridden in config.toml.
549:             request_max_retries: None,
550:             stream_max_retries: None,
551:             stream_idle_timeout_ms: None,
552:             websocket_connect_timeout_ms: None,
553:             requires_openai_auth: true,
554:             supports_websockets: true,
555:             supports_standalone_web_search: true,
556:             include_internal_metadata: true,
557:         }
```

## codex-rs/model-provider/src/auth.rs:126-168
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/model-provider/src/auth.rs#L126
```rust
126: struct AuthManagerAuthProvider {
127:     auth_manager: Arc<AuthManager>,
128:     // Startup auth is only the account-scoped identity anchor. Request
129:     // headers always come from the current AuthManager snapshot below.
130:     expected_auth: CodexAuth,
131: }
132: 
133: impl AuthManagerAuthProvider {
134:     fn is_expected_auth(&self, auth: &CodexAuth) -> bool {
135:         auth.uses_codex_backend()
136:             && auth.get_account_id() == self.expected_auth.get_account_id()
137:             && auth.get_chatgpt_user_id() == self.expected_auth.get_chatgpt_user_id()
138:             && auth.is_workspace_account() == self.expected_auth.is_workspace_account()
139:     }
140: 
141:     fn current_auth(&self) -> Option<CodexAuth> {
142:         self.auth_manager
143:             .auth_cached()
144:             .filter(|auth| self.is_expected_auth(auth))
145:     }
146: }
147: 
148: impl AuthProvider for AuthManagerAuthProvider {
149:     fn add_auth_headers(&self, headers: &mut HeaderMap) {
150:         let Some(auth) = self.current_auth() else {
151:             return;
152:         };
153:         auth_provider_from_auth(&auth).add_auth_headers(headers);
154:     }
155: 
156:     fn resolve_auth_headers(&self) -> AuthHeadersFuture<'_> {
157:         Box::pin(async move {
158:             let auth = self
159:                 .auth_manager
160:                 .auth()
161:                 .await
162:                 .filter(|auth| self.is_expected_auth(auth))
163:                 .ok_or_else(|| {
164:                     AuthError::Transient("managed authentication is unavailable".to_string())
165:                 })?;
166:             Ok(auth_provider_from_auth(&auth).to_auth_headers())
167:         })
168:     }
```

## codex-rs/model-provider/src/auth.rs:197-207
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/model-provider/src/auth.rs#L197
```rust
197: pub(crate) fn resolve_provider_auth(
198:     auth: Option<&CodexAuth>,
199:     provider: &ModelProviderInfo,
200: ) -> codex_protocol::error::Result<SharedAuthProvider> {
201:     if let Some(auth) = bearer_auth_for_provider(provider)? {
202:         return Ok(Arc::new(auth));
203:     }
204: 
205:     if !provider.requires_openai_auth && provider.auth.is_none() {
206:         return Ok(unauthenticated_auth_provider());
207:     }
```

## codex-rs/model-provider/src/auth.rs:306-339
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/model-provider/src/auth.rs#L306
```rust
306: /// Builds request-header auth for a first-party Codex auth snapshot.
307: pub fn auth_provider_from_auth(auth: &CodexAuth) -> SharedAuthProvider {
308:     match auth {
309:         CodexAuth::AgentIdentity(auth) => {
310:             Arc::new(AgentIdentityAuthProvider { auth: auth.clone() })
311:         }
312:         CodexAuth::Headers(auth) => Arc::new(HeaderAuthProvider { auth: auth.clone() }),
313:         CodexAuth::BedrockApiKey(_) | CodexAuth::BedrockAccessKeys(_) => {
314:             unreachable!("{BEDROCK_API_KEY_UNSUPPORTED_MESSAGE}")
315:         }
316:         CodexAuth::ApiKey(_)
317:         | CodexAuth::Chatgpt(_)
318:         | CodexAuth::ChatgptAuthTokens(_)
319:         | CodexAuth::PersonalAccessToken(_) => Arc::new(BearerAuthProvider {
320:             token: auth.get_token().ok(),
321:             account_id: auth.get_account_id(),
322:             is_fedramp_account: auth.is_fedramp_account(),
323:         }),
324:     }
325: }
326: 
327: /// Builds request-header auth that reads the current managed auth snapshot on
328: /// every request while remaining scoped to the expected auth identity.
329: ///
330: /// Callers with account-scoped state should pass the same snapshot that keyed
331: /// that state so a later account switch cannot reuse it.
332: pub fn auth_provider_from_auth_manager(
333:     auth_manager: Arc<AuthManager>,
334:     expected_auth: &CodexAuth,
335: ) -> SharedAuthProvider {
336:     Arc::new(AuthManagerAuthProvider {
337:         auth_manager,
338:         expected_auth: expected_auth.clone(),
339:     })
```

## codex-rs/model-provider/src/bearer_auth_provider.rs:30-46
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/model-provider/src/bearer_auth_provider.rs#L30
```rust
30: 
31: impl AuthProvider for BearerAuthProvider {
32:     fn add_auth_headers(&self, headers: &mut HeaderMap) {
33:         if let Some(token) = self.token.as_ref()
34:             && let Ok(header) = HeaderValue::from_str(&format!("Bearer {token}"))
35:         {
36:             let _ = headers.insert(http::header::AUTHORIZATION, header);
37:         }
38:         if let Some(account_id) = self.account_id.as_ref()
39:             && let Ok(header) = HeaderValue::from_str(account_id)
40:         {
41:             let _ = headers.insert("ChatGPT-Account-ID", header);
42:         }
43:         if self.is_fedramp_account {
44:             let _ = headers.insert("X-OpenAI-Fedramp", HeaderValue::from_static("true"));
45:         }
46:     }
```

## codex-rs/login/src/auth/manager.rs:203-216
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/login/src/auth/manager.rs#L203
```rust
203: const TOKEN_REFRESH_INTERVAL: i64 = 8;
204: const CHATGPT_ACCESS_TOKEN_REFRESH_WINDOW_MINUTES: i64 = 5;
205: 
206: const REFRESH_TOKEN_EXPIRED_MESSAGE: &str = "Your access token could not be refreshed because your refresh token has expired. Please log out and sign in again.";
207: const REFRESH_TOKEN_REUSED_MESSAGE: &str = "Your access token could not be refreshed because your refresh token was already used. Please log out and sign in again.";
208: const REFRESH_TOKEN_INVALIDATED_MESSAGE: &str = "Your access token could not be refreshed because your refresh token was revoked. Please log out and sign in again.";
209: const REFRESH_TOKEN_UNKNOWN_MESSAGE: &str =
210:     "Your access token could not be refreshed. Please log out and sign in again.";
211: const REFRESH_TOKEN_ACCOUNT_MISMATCH_MESSAGE: &str = "Your access token could not be refreshed because you have since logged out or signed in to another account. Please sign in again.";
212: const REFRESH_TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
213: pub(super) const REVOKE_TOKEN_URL: &str = "https://auth.openai.com/oauth/revoke";
214: pub const REFRESH_TOKEN_URL_OVERRIDE_ENV_VAR: &str = "CODEX_REFRESH_TOKEN_URL_OVERRIDE";
215: pub const REVOKE_TOKEN_URL_OVERRIDE_ENV_VAR: &str = "CODEX_REVOKE_TOKEN_URL_OVERRIDE";
216: pub const CLIENT_ID_OVERRIDE_ENV_VAR: &str = "CODEX_APP_SERVER_LOGIN_CLIENT_ID";
```

## codex-rs/login/src/auth/manager.rs:580-625
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/login/src/auth/manager.rs#L580
```rust
580:     pub fn get_token_data(&self) -> Result<TokenData, std::io::Error> {
581:         let auth_dot_json: Option<AuthDotJson> = self.get_current_auth_json();
582:         match auth_dot_json {
583:             Some(AuthDotJson {
584:                 tokens: Some(tokens),
585:                 last_refresh: Some(_),
586:                 ..
587:             }) => Ok(tokens),
588:             _ => Err(std::io::Error::other("Token data is not available.")),
589:         }
590:     }
591: 
592:     /// Returns the token string used for bearer authentication.
593:     pub fn get_token(&self) -> Result<String, std::io::Error> {
594:         match self {
595:             Self::ApiKey(auth) => Ok(auth.api_key.clone()),
596:             Self::Chatgpt(_) | Self::ChatgptAuthTokens(_) => {
597:                 let access_token = self.get_token_data()?.access_token;
598:                 Ok(access_token)
599:             }
600:             Self::AgentIdentity(_) => Err(std::io::Error::other(
601:                 "agent identity auth does not expose a bearer token",
602:             )),
603:             Self::Headers(_) => Err(std::io::Error::other(
604:                 "header auth does not expose a bearer token",
605:             )),
606:             Self::PersonalAccessToken(auth) => Ok(auth.access_token().to_string()),
607:             Self::BedrockApiKey(_) | Self::BedrockAccessKeys(_) => Err(std::io::Error::other(
608:                 "Bedrock API key auth does not expose a Codex bearer token",
609:             )),
610:         }
611:     }
612: 
613:     /// Returns `None` if Codex backend auth does not expose an account id.
614:     pub fn get_account_id(&self) -> Option<String> {
615:         match self {
616:             Self::Headers(headers) => headers
617:                 .headers()
618:                 .get("chatgpt-account-id")
619:                 .and_then(|value| value.to_str().ok())
620:                 .filter(|account_id| !account_id.is_empty() && account_id.trim() == *account_id)
621:                 .map(ToOwned::to_owned),
622:             Self::AgentIdentity(auth) => Some(auth.account_id().to_string()),
623:             Self::PersonalAccessToken(auth) => Some(auth.account_id().to_string()),
624:             _ => self.get_current_token_data().and_then(|t| t.account_id),
625:         }
```

## codex-rs/login/src/auth/manager.rs:1599-1729
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/login/src/auth/manager.rs#L1599
```rust
1599: fn persist_tokens(
1600:     storage: &Arc<dyn AuthStorageBackend>,
1601:     id_token: Option<String>,
1602:     access_token: Option<String>,
1603:     refresh_token: Option<String>,
1604: ) -> std::io::Result<AuthDotJson> {
1605:     let mut auth_dot_json = storage
1606:         .load()?
1607:         .ok_or(std::io::Error::other("Token data is not available."))?;
1608: 
1609:     let tokens = auth_dot_json.tokens.get_or_insert_with(TokenData::default);
1610:     if let Some(id_token) = id_token {
1611:         tokens.id_token = parse_chatgpt_jwt_claims(&id_token).map_err(std::io::Error::other)?;
1612:     }
1613:     if let Some(access_token) = access_token {
1614:         tokens.access_token = access_token;
1615:     }
1616:     if let Some(refresh_token) = refresh_token {
1617:         tokens.refresh_token = refresh_token;
1618:     }
1619:     auth_dot_json.last_refresh = Some(Utc::now());
1620:     storage.save(&auth_dot_json)?;
1621:     Ok(auth_dot_json)
1622: }
1623: 
1624: // Requests refreshed ChatGPT OAuth tokens from the auth service using a refresh token.
1625: // The caller is responsible for persisting any returned tokens.
1626: async fn request_chatgpt_token_refresh(
1627:     refresh_token: String,
1628:     client: &HttpClient,
1629: ) -> Result<RefreshResponse, RefreshTokenError> {
1630:     let client_id = oauth_client_id();
1631:     let endpoint = refresh_token_endpoint();
1632:     let oauth = OAuthClient::new(
1633:         client,
1634:         TokenEndpoint {
1635:             url: &endpoint,
1636:             client_id: &client_id,
1637:             encoding: TokenEncoding::Json,
1638:             timeout: None,
1639:             error_body_limit: ErrorBodyLimit::Unlimited,
1640:         },
1641:     );
1642:     match oauth
1643:         .refresh(RefreshTokenGrant {
1644:             refresh_token: &refresh_token,
1645:             resource: None,
1646:         })
1647:         .await
1648:     {
1649:         Ok(response) => Ok(response),
1650:         Err(OAuthError::Rejected(rejection)) => {
1651:             if let Some(codex_http_client::HttpError::Policy(denied)) = rejection.body_read_error {
1652:                 return Err(denied.into());
1653:             }
1654:             let status = rejection.status;
1655:             let detail = &rejection.detail;
1656:             tracing::error!(%status, ?detail, "Failed to refresh token");
1657:             let code = detail.error_code();
1658:             let is_invalid_grant_bad_request = status == StatusCode::BAD_REQUEST
1659:                 && code.is_some_and(|code| code.eq_ignore_ascii_case("invalid_grant"));
1660:             let failed = classify_refresh_token_failure(code, detail, is_invalid_grant_bad_request);
1661:             if status == StatusCode::UNAUTHORIZED
1662:                 || failed.reason != RefreshTokenFailedReason::Other
1663:                 || is_invalid_grant_bad_request
1664:             {
1665:                 Err(RefreshTokenError::Permanent(failed))
1666:             } else {
1667:                 Err(RefreshTokenError::Transient(std::io::Error::other(
1668:                     format!("Failed to refresh token: {status}: {detail}"),
1669:                 )))
1670:             }
1671:         }
1672:         Err(OAuthError::Transport(error)) => Err(error.into()),
1673:         Err(error @ OAuthError::InvalidResponse) => {
1674:             Err(RefreshTokenError::Transient(std::io::Error::other(error)))
1675:         }
1676:     }
1677: }
1678: 
1679: fn classify_refresh_token_failure(
1680:     code: Option<&str>,
1681:     detail: &TokenErrorDetail,
1682:     is_invalid_grant_bad_request: bool,
1683: ) -> RefreshTokenFailedError {
1684:     let normalized_code = code.map(str::to_ascii_lowercase);
1685:     let reason = match normalized_code.as_deref() {
1686:         Some("refresh_token_expired") => RefreshTokenFailedReason::Expired,
1687:         Some("refresh_token_reused") => RefreshTokenFailedReason::Exhausted,
1688:         Some("refresh_token_invalidated") => RefreshTokenFailedReason::Revoked,
1689:         _ => RefreshTokenFailedReason::Other,
1690:     };
1691: 
1692:     if reason == RefreshTokenFailedReason::Other && !is_invalid_grant_bad_request {
1693:         tracing::warn!(
1694:             backend_detail = ?detail,
1695:             "Encountered unknown response while refreshing token"
1696:         );
1697:     }
1698: 
1699:     let message = match reason {
1700:         RefreshTokenFailedReason::Expired => REFRESH_TOKEN_EXPIRED_MESSAGE.to_string(),
1701:         RefreshTokenFailedReason::Exhausted => REFRESH_TOKEN_REUSED_MESSAGE.to_string(),
1702:         RefreshTokenFailedReason::Revoked => REFRESH_TOKEN_INVALIDATED_MESSAGE.to_string(),
1703:         RefreshTokenFailedReason::Other => REFRESH_TOKEN_UNKNOWN_MESSAGE.to_string(),
1704:     };
1705: 
1706:     RefreshTokenFailedError::new(reason, message)
1707: }
1708: 
1709: #[derive(Deserialize, Clone)]
1710: struct RefreshResponse {
1711:     id_token: Option<String>,
1712:     access_token: Option<String>,
1713:     refresh_token: Option<String>,
1714: }
1715: 
1716: // Shared constant for token refresh (client id used for oauth token refresh flow)
1717: pub const CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
1718: 
1719: pub fn oauth_client_id() -> String {
1720:     std::env::var(CLIENT_ID_OVERRIDE_ENV_VAR)
1721:         .ok()
1722:         .filter(|client_id| !client_id.trim().is_empty())
1723:         .unwrap_or_else(|| CLIENT_ID.to_string())
1724: }
1725: 
1726: fn refresh_token_endpoint() -> String {
1727:     std::env::var(REFRESH_TOKEN_URL_OVERRIDE_ENV_VAR)
1728:         .unwrap_or_else(|_| REFRESH_TOKEN_URL.to_string())
1729: }
```

## codex-rs/login/src/auth/manager.rs:1849-1861
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/login/src/auth/manager.rs#L1849
```rust
1849: // UnauthorizedRecovery is a state machine that handles an attempt to refresh the authentication when requests
1850: // to API fail with 401 status code.
1851: // The client calls next() every time it encounters a 401 error, one time per retry.
1852: // For API key based authentication, we don't do anything and let the error bubble to the user.
1853: //
1854: // For ChatGPT based authentication, we:
1855: // 1. Attempt to reload the auth data from disk. We only reload if the account id matches the one the current process is running as.
1856: // 2. Attempt to refresh the token using OAuth token refresh flow.
1857: // If after both steps the server still responds with 401 we let the error bubble to the user.
1858: //
1859: // For external auth sources, UnauthorizedRecovery retries once by asking the
1860: // configured provider to refresh and caching the returned auth through the same
1861: // path used by other auth sources.
```

## codex-rs/login/src/auth/manager.rs:1984-2027
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/login/src/auth/manager.rs#L1984
```rust
1984:         match self.step {
1985:             UnauthorizedRecoveryStep::Reload => {
1986:                 match self
1987:                     .manager
1988:                     .reload_if_account_id_matches(self.expected_account_id.as_deref())
1989:                     .await
1990:                 {
1991:                     ReloadOutcome::ReloadedChanged => {
1992:                         self.step = UnauthorizedRecoveryStep::RefreshToken;
1993:                         return Ok(UnauthorizedRecoveryStepResult {
1994:                             auth_state_changed: Some(true),
1995:                         });
1996:                     }
1997:                     ReloadOutcome::ReloadedNoChange => {
1998:                         self.step = UnauthorizedRecoveryStep::RefreshToken;
1999:                         return Ok(UnauthorizedRecoveryStepResult {
2000:                             auth_state_changed: Some(false),
2001:                         });
2002:                     }
2003:                     ReloadOutcome::Skipped => {
2004:                         self.step = UnauthorizedRecoveryStep::Done;
2005:                         return Err(RefreshTokenError::Permanent(RefreshTokenFailedError::new(
2006:                             RefreshTokenFailedReason::Other,
2007:                             REFRESH_TOKEN_ACCOUNT_MISMATCH_MESSAGE.to_string(),
2008:                         )));
2009:                     }
2010:                 }
2011:             }
2012:             UnauthorizedRecoveryStep::RefreshToken => {
2013:                 self.manager.refresh_token_from_authority().await?;
2014:                 self.step = UnauthorizedRecoveryStep::Done;
2015:                 return Ok(UnauthorizedRecoveryStepResult {
2016:                     auth_state_changed: Some(true),
2017:                 });
2018:             }
2019:             UnauthorizedRecoveryStep::ExternalRefresh => {
2020:                 self.manager.refresh_token_from_authority().await?;
2021:                 self.step = UnauthorizedRecoveryStep::Done;
2022:                 return Ok(UnauthorizedRecoveryStepResult {
2023:                     auth_state_changed: Some(true),
2024:                 });
2025:             }
2026:             UnauthorizedRecoveryStep::Done => {}
2027:         }
```

## codex-rs/login/src/auth/manager.rs:2390-2406
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/login/src/auth/manager.rs#L2390
```rust
2390:     /// For managed ChatGPT auth that needs a proactive refresh, first performs
2391:     /// a guarded reload and then refreshes only if the on-disk auth is unchanged.
2392:     #[instrument(level = "trace", skip_all)]
2393:     pub async fn auth(&self) -> Option<CodexAuth> {
2394:         if self.has_external_auth() {
2395:             self.reload().await;
2396:             return self.auth_cached();
2397:         }
2398: 
2399:         let auth = self.auth_cached()?;
2400:         if Self::should_refresh_proactively(&auth)
2401:             && let Err(err) = self.refresh_token().await
2402:         {
2403:             tracing::error!("Failed to refresh token: {}", err);
2404:             return Some(auth);
2405:         }
2406:         self.auth_cached()
```

## codex-rs/login/src/auth/manager.rs:2487-2519
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/login/src/auth/manager.rs#L2487
```rust
2487:     async fn reload_if_account_id_matches(
2488:         &self,
2489:         expected_account_id: Option<&str>,
2490:     ) -> ReloadOutcome {
2491:         let expected_account_id = match expected_account_id {
2492:             Some(account_id) => account_id,
2493:             None => {
2494:                 tracing::info!("Skipping auth reload because no account id is available.");
2495:                 return ReloadOutcome::Skipped;
2496:             }
2497:         };
2498: 
2499:         let new_auth = self.load_auth().await;
2500:         let new_account_id = new_auth.as_ref().and_then(CodexAuth::get_account_id);
2501: 
2502:         if new_account_id.as_deref() != Some(expected_account_id) {
2503:             let found_account_id = new_account_id.as_deref().unwrap_or("unknown");
2504:             tracing::info!(
2505:                 "Skipping auth reload due to account id mismatch (expected: {expected_account_id}, found: {found_account_id})"
2506:             );
2507:             return ReloadOutcome::Skipped;
2508:         }
2509: 
2510:         tracing::info!("Reloading auth for account {expected_account_id}");
2511:         let cached_before_reload = self.auth_cached();
2512:         let auth_changed =
2513:             !Self::auths_equal_for_refresh(cached_before_reload.as_ref(), new_auth.as_ref());
2514:         self.set_cached_auth(new_auth);
2515:         if auth_changed {
2516:             ReloadOutcome::ReloadedChanged
2517:         } else {
2518:             ReloadOutcome::ReloadedNoChange
2519:         }
```

## codex-rs/login/src/auth/manager.rs:2844-2939
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/login/src/auth/manager.rs#L2844
```rust
2844:     /// Attempt to refresh the token by first performing a guarded reload from
2845:     /// the active auth source. If the loaded token differs from the cached token,
2846:     /// we can assume that the source already refreshed it. Otherwise, ask the
2847:     /// token authority to refresh.
2848:     pub async fn refresh_token(&self) -> Result<(), RefreshTokenError> {
2849:         let _refresh_guard = self.refresh_lock.acquire().await.map_err(|_| {
2850:             RefreshTokenError::Permanent(RefreshTokenFailedError::new(
2851:                 RefreshTokenFailedReason::Other,
2852:                 REFRESH_TOKEN_UNKNOWN_MESSAGE.to_string(),
2853:             ))
2854:         })?;
2855:         let auth_before_reload = self.auth_cached();
2856:         if auth_before_reload
2857:             .as_ref()
2858:             .is_some_and(|auth| auth.is_api_key_auth() || auth.is_personal_access_token_auth())
2859:         {
2860:             return Ok(());
2861:         }
2862:         let expected_account_id = auth_before_reload
2863:             .as_ref()
2864:             .and_then(CodexAuth::get_account_id);
2865: 
2866:         match self
2867:             .reload_if_account_id_matches(expected_account_id.as_deref())
2868:             .await
2869:         {
2870:             ReloadOutcome::ReloadedChanged => {
2871:                 tracing::info!("Skipping token refresh because auth changed after guarded reload.");
2872:                 Ok(())
2873:             }
2874:             ReloadOutcome::ReloadedNoChange => self.refresh_token_from_authority_impl().await,
2875:             ReloadOutcome::Skipped => {
2876:                 Err(RefreshTokenError::Permanent(RefreshTokenFailedError::new(
2877:                     RefreshTokenFailedReason::Other,
2878:                     REFRESH_TOKEN_ACCOUNT_MISMATCH_MESSAGE.to_string(),
2879:                 )))
2880:             }
2881:         }
2882:     }
2883: 
2884:     /// Attempt to refresh the current auth token from the authority that issued
2885:     /// it and update the shared cache. If the token refresh fails, returns the
2886:     /// error to the caller.
2887:     pub async fn refresh_token_from_authority(&self) -> Result<(), RefreshTokenError> {
2888:         let _refresh_guard = self.refresh_lock.acquire().await.map_err(|_| {
2889:             RefreshTokenError::Permanent(RefreshTokenFailedError::new(
2890:                 RefreshTokenFailedReason::Other,
2891:                 REFRESH_TOKEN_UNKNOWN_MESSAGE.to_string(),
2892:             ))
2893:         })?;
2894:         self.refresh_token_from_authority_impl().await
2895:     }
2896: 
2897:     async fn refresh_token_from_authority_impl(&self) -> Result<(), RefreshTokenError> {
2898:         tracing::info!("Refreshing token");
2899: 
2900:         let attempted_auth = self.auth_cached();
2901:         if let Some(error) = attempted_auth
2902:             .as_ref()
2903:             .and_then(|auth| self.refresh_failure_for_auth(auth))
2904:         {
2905:             return Err(RefreshTokenError::Permanent(error));
2906:         }
2907: 
2908:         let result = if self.has_external_auth() {
2909:             self.refresh_external_auth(ExternalAuthRefreshReason::Unauthorized)
2910:                 .await
2911:         } else {
2912:             match attempted_auth.as_ref() {
2913:                 Some(CodexAuth::Chatgpt(chatgpt_auth)) => {
2914:                     let token_data = chatgpt_auth.current_token_data().ok_or_else(|| {
2915:                         RefreshTokenError::Transient(std::io::Error::other(
2916:                             "Token data is not available.",
2917:                         ))
2918:                     })?;
2919:                     self.refresh_and_persist_chatgpt_token(chatgpt_auth, token_data.refresh_token)
2920:                         .await
2921:                 }
2922:                 Some(
2923:                     CodexAuth::ApiKey(_)
2924:                     | CodexAuth::ChatgptAuthTokens(_)
2925:                     | CodexAuth::Headers(_)
2926:                     | CodexAuth::AgentIdentity(_)
2927:                     | CodexAuth::PersonalAccessToken(_)
2928:                     | CodexAuth::BedrockApiKey(_)
2929:                     | CodexAuth::BedrockAccessKeys(_),
2930:                 )
2931:                 | None => Ok(()),
2932:             }
2933:         };
2934:         if let Some(attempted_auth) = attempted_auth.as_ref()
2935:             && let Err(RefreshTokenError::Permanent(error)) = &result
2936:         {
2937:             self.record_permanent_refresh_failure_if_unchanged(attempted_auth, error);
2938:         }
2939:         result
```

## codex-rs/login/src/auth/manager.rs:3004-3025
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/login/src/auth/manager.rs#L3004
```rust
3004:     fn should_refresh_proactively(auth: &CodexAuth) -> bool {
3005:         let chatgpt_auth = match auth {
3006:             CodexAuth::Chatgpt(chatgpt_auth) => chatgpt_auth,
3007:             _ => return false,
3008:         };
3009: 
3010:         let auth_dot_json = match chatgpt_auth.current_auth_json() {
3011:             Some(auth_dot_json) => auth_dot_json,
3012:             None => return false,
3013:         };
3014:         if let Some(tokens) = auth_dot_json.tokens.as_ref()
3015:             && let Ok(Some(expires_at)) = parse_jwt_expiration(&tokens.access_token)
3016:         {
3017:             return expires_at
3018:                 <= Utc::now()
3019:                     + chrono::Duration::minutes(CHATGPT_ACCESS_TOKEN_REFRESH_WINDOW_MINUTES);
3020:         }
3021:         let last_refresh = match auth_dot_json.last_refresh {
3022:             Some(last_refresh) => last_refresh,
3023:             None => return false,
3024:         };
3025:         last_refresh < Utc::now() - chrono::Duration::days(TOKEN_REFRESH_INTERVAL)
```

## codex-rs/login/src/auth/manager.rs:3092-3109
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/login/src/auth/manager.rs#L3092
```rust
3092:     async fn refresh_and_persist_chatgpt_token(
3093:         &self,
3094:         auth: &ChatgptAuth,
3095:         refresh_token: String,
3096:     ) -> Result<(), RefreshTokenError> {
3097:         let refresh_response = request_chatgpt_token_refresh(refresh_token, auth.client()).await?;
3098: 
3099:         persist_tokens(
3100:             auth.storage(),
3101:             refresh_response.id_token,
3102:             refresh_response.access_token,
3103:             refresh_response.refresh_token,
3104:         )
3105:         .map_err(RefreshTokenError::from)?;
3106:         self.reload().await;
3107: 
3108:         Ok(())
3109:     }
```

## codex-rs/login/src/auth/storage.rs:195-223
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/login/src/auth/storage.rs#L195
```rust
195: impl AuthStorageBackend for FileAuthStorage {
196:     fn load(&self) -> std::io::Result<Option<AuthDotJson>> {
197:         let auth_file = get_auth_file(&self.codex_home);
198:         let auth_dot_json = match self.try_read_auth_json(&auth_file) {
199:             Ok(auth) => auth,
200:             Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
201:             Err(err) => return Err(err),
202:         };
203:         Ok(Some(auth_dot_json))
204:     }
205: 
206:     fn save(&self, auth_dot_json: &AuthDotJson) -> std::io::Result<()> {
207:         let auth_file = get_auth_file(&self.codex_home);
208: 
209:         if let Some(parent) = auth_file.parent() {
210:             std::fs::create_dir_all(parent)?;
211:         }
212:         let json_data = serde_json::to_string_pretty(auth_dot_json)?;
213:         let mut options = OpenOptions::new();
214:         options.truncate(true).write(true).create(true);
215:         #[cfg(unix)]
216:         {
217:             options.mode(0o600);
218:         }
219:         let mut file = options.open(auth_file)?;
220:         file.write_all(json_data.as_bytes())?;
221:         file.flush()?;
222:         Ok(())
223:     }
```

## codex-rs/login/src/server.rs:602-609
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/login/src/server.rs#L602
```rust
602:     build_authorization_url(AuthorizationRequest {
603:         endpoint: &format!("{issuer}/oauth/authorize"),
604:         client_id,
605:         redirect_uri,
606:         scope: Some(
607:             "openid profile email offline_access api.connectors.read api.connectors.invoke",
608:         ),
609:         resource: None,
```

## codex-rs/login/src/server.rs:839-867
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/login/src/server.rs#L839
```rust
839: ) -> io::Result<()> {
840:     // Reuse existing synchronous logic but run it off the async runtime.
841:     let codex_home = codex_home.to_path_buf();
842:     tokio::task::spawn_blocking(move || {
843:         let mut tokens = TokenData {
844:             id_token: parse_chatgpt_jwt_claims(&id_token).map_err(io::Error::other)?,
845:             access_token,
846:             refresh_token,
847:             account_id: None,
848:         };
849:         if let Some(acc) = jwt_auth_claims(&id_token)
850:             .get("chatgpt_account_id")
851:             .and_then(|v| v.as_str())
852:         {
853:             tokens.account_id = Some(acc.to_string());
854:         }
855:         let auth = AuthDotJson {
856:             auth_mode: Some(AuthMode::Chatgpt),
857:             openai_api_key: api_key,
858:             tokens: Some(tokens),
859:             last_refresh: Some(Utc::now()),
860:             agent_identity: None,
861:             personal_access_token: None,
862:             bedrock_api_key: None,
863:             bedrock_access_keys: None,
864:         };
865:         save_auth(
866:             &codex_home,
867:             &auth,
```

## codex-rs/login/src/oauth/client.rs:76-125
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/login/src/oauth/client.rs#L76
```rust
76:     pub(crate) async fn refresh<T: DeserializeOwned>(
77:         &self,
78:         grant: RefreshTokenGrant<'_>,
79:     ) -> Result<T, OAuthError> {
80:         let mut parameters = vec![
81:             ("grant_type", "refresh_token"),
82:             ("client_id", self.endpoint.client_id),
83:             ("refresh_token", grant.refresh_token),
84:         ];
85:         if let Some(resource) = grant.resource {
86:             parameters.push(("resource", resource));
87:         }
88:         self.exchange(&parameters, &[grant.refresh_token]).await
89:     }
90: 
91:     async fn exchange<T: DeserializeOwned>(
92:         &self,
93:         parameters: &[(&str, &str)],
94:         secrets: &[&str],
95:     ) -> Result<T, OAuthError> {
96:         let mut request = self.http_client.post(self.endpoint.url);
97:         if let Some(timeout) = self.endpoint.timeout {
98:             request = request.timeout(timeout);
99:         }
100:         request = match self.endpoint.encoding {
101:             TokenEncoding::Form => {
102:                 let mut form = url::form_urlencoded::Serializer::new(String::new());
103:                 form.extend_pairs(parameters.iter().copied());
104:                 request
105:                     .header("Content-Type", "application/x-www-form-urlencoded")
106:                     .body(form.finish())
107:             }
108:             TokenEncoding::Json => request
109:                 .header("Content-Type", "application/json")
110:                 .json(&parameters.iter().copied().collect::<BTreeMap<_, _>>()),
111:         };
112:         let response = request
113:             .send()
114:             .await
115:             .map_err(|error| OAuthError::Transport(redact_error_url(error)))?;
116:         if !response.status().is_success() {
117:             return Err(OAuthError::Rejected(Box::new(
118:                 TokenRejection::from_response(response, self.endpoint.error_body_limit, secrets)
119:                     .await,
120:             )));
121:         }
122:         response.json().await.map_err(|error| match error {
123:             error @ codex_http_client::HttpError::Policy(_) => OAuthError::Transport(error),
124:             _ => OAuthError::InvalidResponse,
125:         })
```

## codex-rs/model-provider/src/provider.rs:244-283
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/model-provider/src/provider.rs#L244
```rust
244:     /// Resolves routing for Responses HTTP, compaction, and WebSocket handshakes.
245:     #[expect(
246:         clippy::await_holding_invalid_type,
247:         reason = "serialize discovery and the session's first successful routing transition"
248:     )]
249:     fn responses_api_provider<'a>(
250:         &'a self,
251:         routing_context: &'a WorkspaceRoutingContext,
252:     ) -> ModelProviderFuture<'a, codex_protocol::error::Result<ResolvedResponsesProvider>> {
253:         Box::pin(async move {
254:             let mut provider = self.api_provider().await?;
255:             let mut redirect_policy = ClientRedirectPolicy::Default;
256:             if provider_uses_first_party_auth_path(self.info())
257:                 && self.info().supports_codex_backend_routes()
258:                 && let Some(auth) = self.auth().await.filter(CodexAuth::is_chatgpt_auth)
259:                 && let Some(auth_manager) = self.auth_manager()
260:             {
261:                 let mut previously_routed = routing_context.previously_routed.lock().await;
262:                 if let Some(routing) = auth_manager
263:                     .workspace_routing(
264:                         &auth,
265:                         WorkspaceRoutingRequest {
266:                             provider_base_url: provider.base_url.clone(),
267:                             chatgpt_base_url: routing_context.chatgpt_base_url.clone(),
268:                             previously_routed: *previously_routed,
269:                             session: routing_context.session.clone(),
270:                         },
271:                     )
272:                     .await?
273:                 {
274:                     crate::workspace_routing::apply_workspace_routing(&mut provider, routing)?;
275:                     redirect_policy = ClientRedirectPolicy::Reject;
276:                     *previously_routed = true;
277:                 }
278:             }
279:             Ok(ResolvedResponsesProvider {
280:                 provider,
281:                 redirect_policy,
282:             })
283:         })
```

## codex-rs/model-provider/src/workspace_routing.rs:39-85
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/model-provider/src/workspace_routing.rs#L39
```rust
39: /// Changes only the origin of requests to the selected ChatGPT backend.
40: pub(crate) fn apply_workspace_routing(
41:     provider: &mut codex_api::Provider,
42:     routing: WorkspaceRouting,
43: ) -> io::Result<()> {
44:     let base_url = &mut provider.base_url;
45:     let headers = &mut provider.headers;
46:     let parse = |value: &str| Url::parse(value).map_err(io::Error::other);
47:     let mut url = parse(base_url)?;
48:     let backend = parse(&routing.backend_origin)?;
49:     if backend.scheme() != "https"
50:         || backend.host_str().is_none()
51:         || !backend.username().is_empty()
52:         || backend.password().is_some()
53:         || backend.path() != "/"
54:         || backend.query().is_some()
55:         || backend.fragment().is_some()
56:     {
57:         return Err(io::Error::other("invalid workspace backend origin"));
58:     }
59:     let header = match routing.account_routing_override.as_str() {
60:         "NO_CONSTRAINT" => None,
61:         "us" | "us_cr" => Some(
62:             HeaderValue::from_str(&routing.account_routing_override).map_err(io::Error::other)?,
63:         ),
64:         _ => return Err(io::Error::other("invalid workspace routing override")),
65:     };
66:     url.set_scheme(backend.scheme())
67:         .map_err(|()| io::Error::other("invalid backend scheme"))?;
68:     url.set_host(backend.host_str()).map_err(io::Error::other)?;
69:     url.set_port(backend.port())
70:         .map_err(|()| io::Error::other("invalid backend port"))?;
71:     *base_url = url.into();
72:     headers.remove(ACCOUNT_ROUTING_HEADER);
73:     if let Some(header) = header {
74:         headers.insert(ACCOUNT_ROUTING_HEADER, header);
75:     }
76:     Ok(())
77: }
78: 
79: /// API deployment and redirect policy resolved for one Responses request.
80: /// Workspace routes reject redirects even when their routing override supplies no header.
81: #[derive(Debug)]
82: pub struct ResolvedResponsesProvider {
83:     pub provider: codex_api::Provider,
84:     pub redirect_policy: ClientRedirectPolicy,
85: }
```

## codex-rs/app-server/src/request_processors/account_processor/workspace_routing.rs:289-337
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/app-server/src/request_processors/account_processor/workspace_routing.rs#L289
```rust
289:                 let routing = if let Some(cached) = cached {
290:                     cached
291:                 } else {
292:                     let mut recovery = self.auth_manager.unauthorized_recovery();
293:                     let mut discovery_auth = auth.clone();
294:                     let mut discovery_generation = auth_state.generation;
295:                     let response = loop {
296:                         let client = BackendClient::from_auth(
297:                             &config.chatgpt_base_url,
298:                             &discovery_auth,
299:                             config.http_client_factory(),
300:                         );
301:                         match client.get_accounts_check().await {
302:                             Ok(response) => break response,
303:                             Err(error) => {
304:                                 let unauthorized = error.is_unauthorized();
305:                                 if unauthorized
306:                                     && recovery.has_next()
307:                                     && recovery.next().await.is_ok()
308:                                 {
309:                                     let refreshed = *current_auth_changes.borrow();
310:                                     if refreshed.owner_generation != auth_state.owner_generation {
311:                                         return Err(WorkspaceRoutingError::AccountChanged.into());
312:                                     }
313:                                     discovery_generation = refreshed.generation;
314:                                     discovery_auth = self
315:                                         .auth_manager
316:                                         .auth_cached()
317:                                         .ok_or(WorkspaceRoutingError::AccountChanged)?;
318:                                     continue;
319:                                 }
320:                                 return Err(if unauthorized {
321:                                     WorkspaceRoutingError::DiscoveryUnauthorized
322:                                 } else {
323:                                     WorkspaceRoutingError::DiscoveryFailed
324:                                 }
325:                                 .into());
326:                             }
327:                         }
328:                     };
329:                     let mut accounts = response
330:                         .accounts
331:                         .into_iter()
332:                         .filter(|account| account.id == account_id);
333:                     let entry = accounts
334:                         .next()
335:                         .ok_or(WorkspaceRoutingError::MissingWorkspace)?;
336:                     if accounts.next().is_some() {
337:                         return Err(WorkspaceRoutingError::DuplicateWorkspace.into());
```

## codex-rs/app-server/src/request_processors/account_processor/workspace_routing.rs:427-480
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/app-server/src/request_processors/account_processor/workspace_routing.rs#L427
```rust
427: fn resolve_routing(
428:     entry: AccountEntry,
429:     required_chatgpt_base_url: Option<&str>,
430:     effective_base_url: &str,
431: ) -> Result<WorkspaceRouting, AccountReadError> {
432:     let backend = entry
433:         .workspace_backend_origin
434:         .ok_or(WorkspaceRoutingError::MissingBackendOrigin)?;
435:     let account_routing_override = match entry.account_routing_override.as_deref() {
436:         Some(value @ ("NO_CONSTRAINT" | "us" | "us_cr")) => value.to_owned(),
437:         _ => {
438:             return Err(WorkspaceRoutingError::InvalidRoutingOverride.into());
439:         }
440:     };
441:     let required = required_chatgpt_base_url
442:         .map(parse_backend_url)
443:         .transpose()?;
444:     let discovered = if backend == "NO_CONSTRAINT" {
445:         None
446:     } else {
447:         let url = parse_backend_url(&backend)?;
448:         if url.path() != "/" || url.query().is_some() || url.fragment().is_some() {
449:             return Err(WorkspaceRoutingError::BackendIsNotOrigin.into());
450:         }
451:         Some(url)
452:     };
453:     let origin = match (required, discovered) {
454:         (Some(required), Some(discovered)) => {
455:             if required.origin() != discovered.origin() {
456:                 return Err(WorkspaceRoutingError::BackendConflict.into());
457:             }
458:             required.origin()
459:         }
460:         (Some(url), None) | (None, Some(url)) => url.origin(),
461:         (None, None) => parse_backend_url(effective_base_url)?.origin(),
462:     };
463:     Ok(WorkspaceRouting {
464:         chatgpt_account_id: entry.id,
465:         backend_origin: origin.ascii_serialization(),
466:         account_routing_override,
467:     })
468: }
469: 
470: fn parse_backend_url(value: &str) -> Result<Url, AccountReadError> {
471:     let url = Url::parse(value).map_err(|_| WorkspaceRoutingError::InvalidBackendUrl)?;
472:     if url.scheme() != "https"
473:         || url.host_str().is_none()
474:         || !url.username().is_empty()
475:         || url.password().is_some()
476:         || value.trim() != value
477:     {
478:         return Err(WorkspaceRoutingError::InvalidBackendOrigin.into());
479:     }
480:     Ok(url)
```

## codex-rs/backend-client/src/client.rs:398-407
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/backend-client/src/client.rs#L398
```rust
398:     pub async fn get_accounts_check(
399:         &self,
400:     ) -> std::result::Result<AccountsCheckResponse, RequestError> {
401:         let url = match self.path_style {
402:             PathStyle::CodexApi => format!("{}/api/codex/accounts/check", self.base_url),
403:             PathStyle::ChatGptApi => format!("{}/wham/accounts/check", self.base_url),
404:         };
405:         let (body, _) = self.exec_bootstrap_get(&url).await?;
406:         serde_json::from_str(&body)
407:             .map_err(|_| RequestError::Other(anyhow::anyhow!("Invalid accounts response.")))
```

## codex-rs/core/src/client.rs:885-959
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/core/src/client.rs#L885
```rust
885:     pub(crate) fn build_responses_request(
886:         &self,
887:         prompt: &Prompt,
888:         model_info: &ModelInfo,
889:         effort: Option<ReasoningEffortConfig>,
890:         summary: ReasoningSummaryConfig,
891:         service_tier: Option<String>,
892:         responses_metadata: &CodexResponsesMetadata,
893:         include_internal: bool,
894:     ) -> Result<ResponsesApiRequest> {
895:         let mut input = prompt.get_formatted_input_for_request(model_info);
896:         if !self.reasoning_effort_override_enabled(model_info) {
897:             // Unsupported models and disabled overrides must also accept saved history.
898:             // Filter only the request copy; persisted history remains unchanged.
899:             input.retain(|item| !matches!(item, ResponseItem::ConfigurationUpdate { .. }));
900:         }
901:         let is_openai = self.state.provider.info().is_openai();
902:         let (instructions, tools) = if model_info.use_responses_lite {
903:             // These prompt-only items are rebuilt on every request. Hash their visible payloads
904:             // within the thread so retries and resumed sessions preserve their identity.
905:             let prefix_namespace = Uuid::new_v5(
906:                 &Uuid::NAMESPACE_OID,
907:                 self.state.thread_id.to_string().as_bytes(),
908:             );
909:             let tools = if self.state.provider.capabilities().namespace_tools {
910:                 create_tools_json_for_responses_lite(&prompt.tools)?
911:             } else {
912:                 create_tools_json_for_responses_api(&prompt.tools)?
913:             };
914:             let mut prefix = vec![ResponseItem::AdditionalTools {
915:                 id: Some(ResponseItemId::with_suffix(
916:                     "at",
917:                     Uuid::new_v5(&prefix_namespace, &serde_json::to_vec(&tools)?),
918:                 )),
919:                 role: "developer".to_string(),
920:                 tools,
921:             }];
922:             if !prompt.base_instructions.text.is_empty() {
923:                 let mut instructions = ContextualUserFragment::into(BaseInstructionsFragment(
924:                     prompt.base_instructions.text.clone(),
925:                 ));
926:                 instructions.set_id(Some(ResponseItemId::with_suffix(
927:                     "msg",
928:                     Uuid::new_v5(&prefix_namespace, prompt.base_instructions.text.as_bytes()),
929:                 )));
930:                 prefix.push(instructions);
931:             }
932:             input.splice(0..0, prefix);
933:             (String::new(), None)
934:         } else {
935:             (
936:                 prompt.base_instructions.text.clone(),
937:                 Some(create_tools_raw_json_for_responses_api(&prompt.tools)?.into()),
938:             )
939:         };
940:         if !is_openai {
941:             for item in &mut input {
942:                 item.clear_internal_chat_message_metadata_passthrough();
943:                 if let ResponseItem::FunctionCall {
944:                     encrypted_function_args,
945:                     ..
946:                 } = item
947:                 {
948:                     *encrypted_function_args = None;
949:                 }
950:             }
951:         }
952:         let reasoning = self.build_reasoning(model_info, effort, summary);
953:         let stream_options = (self.state.concurrent_reasoning_summaries_enabled
954:             && is_openai
955:             && reasoning.summary.is_some())
956:         .then_some(StreamOptions {
957:             reasoning_summary_delivery: codex_api::ReasoningSummaryDelivery::SequentialCutoff,
958:         });
959:         let include = vec!["reasoning.encrypted_content".to_string()];
```

## codex-rs/core/src/client.rs:989-1007
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/core/src/client.rs#L989
```rust
989:         let request = ResponsesApiRequest {
990:             model: model_info.slug.clone(),
991:             instructions,
992:             input,
993:             tools,
994:             tool_choice: "auto".to_string(),
995:             parallel_tool_calls: prompt.parallel_tool_calls && !model_info.use_responses_lite,
996:             reasoning: Some(reasoning),
997:             store: false,
998:             stream: true,
999:             stream_options,
1000:             include,
1001:             service_tier,
1002:             prompt_cache_key,
1003:             text,
1004:             client_metadata: Some(client_metadata),
1005:             access_programs: None,
1006:         };
1007:         Ok(request)
```

## codex-rs/core/src/client.rs:1350-1378
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/core/src/client.rs#L1350
```rust
1350:         &self,
1351:         responses_metadata: &CodexResponsesMetadata,
1352:         compression: Compression,
1353:         use_responses_lite: bool,
1354:     ) -> ApiResponsesOptions {
1355:         ApiResponsesOptions {
1356:             session_id: Some(self.client.responses_session_id(responses_metadata)),
1357:             thread_id: Some(responses_metadata.thread_id.to_string()),
1358:             session_source: Some(self.client.state.session_source.clone()),
1359:             extra_headers: {
1360:                 let mut headers = build_responses_headers(
1361:                     self.client.state.beta_features_header.as_deref(),
1362:                     Some(&self.turn_state),
1363:                 );
1364:                 add_originator_header(&mut headers, self.client.state.originator.as_str());
1365:                 headers.extend(
1366:                     self.client
1367:                         .build_responses_compatibility_headers(responses_metadata),
1368:                 );
1369:                 if let Some(header_value) = self.client.generate_attestation_header_for().await {
1370:                     headers.insert(X_OAI_ATTESTATION_HEADER, header_value);
1371:                 }
1372:                 add_responses_lite_header(&mut headers, use_responses_lite);
1373:                 headers
1374:             },
1375:             compression,
1376:             turn_state: Some(Arc::clone(&self.turn_state)),
1377:         }
1378:     }
```

## codex-rs/core/src/client.rs:1617-1625
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/core/src/client.rs#L1617
```rust
1617:     fn responses_request_compression(&self, auth: Option<&CodexAuth>) -> Compression {
1618:         if self.client.state.enable_request_compression
1619:             && auth.is_some_and(CodexAuth::uses_codex_backend)
1620:             && self.client.state.provider.info().is_openai()
1621:         {
1622:             Compression::Zstd
1623:         } else {
1624:             Compression::None
1625:         }
```

## codex-rs/core/src/client.rs:2317-2344
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/core/src/client.rs#L2317
```rust
2317: fn build_responses_headers(
2318:     beta_features_header: Option<&str>,
2319:     turn_state: Option<&Arc<OnceLock<String>>>,
2320: ) -> ApiHeaderMap {
2321:     let mut headers = ApiHeaderMap::new();
2322:     if let Some(value) = beta_features_header
2323:         && !value.is_empty()
2324:         && let Ok(header_value) = HeaderValue::from_str(value)
2325:     {
2326:         headers.insert("x-codex-beta-features", header_value);
2327:     }
2328:     if let Some(turn_state) = turn_state
2329:         && let Some(state) = turn_state.get()
2330:         && let Ok(header_value) = HeaderValue::from_str(state)
2331:     {
2332:         headers.insert(X_CODEX_TURN_STATE_HEADER, header_value);
2333:     }
2334:     headers
2335: }
2336: 
2337: fn add_responses_lite_header(headers: &mut ApiHeaderMap, use_responses_lite: bool) {
2338:     if use_responses_lite {
2339:         headers.insert(
2340:             X_OPENAI_INTERNAL_CODEX_RESPONSES_LITE_HEADER,
2341:             HeaderValue::from_static("true"),
2342:         );
2343:     }
2344: }
```

## codex-rs/codex-api/src/endpoint/responses.rs:83-95
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/codex-api/src/endpoint/responses.rs#L83
```rust
83:         let body = EncodedJsonBody::encode(&request)
84:             .map_err(|e| ApiError::Stream(format!("failed to encode responses request: {e}")))?;
85: 
86:         let mut headers = extra_headers;
87:         if let Some(ref thread_id) = thread_id {
88:             insert_header(&mut headers, "x-client-request-id", thread_id);
89:         }
90:         headers.extend(build_session_headers(session_id, thread_id));
91:         if let Some(subagent) = subagent_header(&session_source) {
92:             insert_header(&mut headers, "x-openai-subagent", &subagent);
93:         }
94: 
95:         self.stream_encoded(body, headers, compression, turn_state)
```

## codex-rs/codex-api/src/endpoint/responses.rs:130-155
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/codex-api/src/endpoint/responses.rs#L130
```rust
130:         let request_compression = match compression {
131:             Compression::None => RequestCompression::None,
132:             Compression::Zstd => RequestCompression::Zstd,
133:         };
134: 
135:         let stream_response = self
136:             .session
137:             .stream_encoded_json_with(
138:                 Method::POST,
139:                 "/responses",
140:                 extra_headers,
141:                 Some(body),
142:                 |req| {
143:                     req.headers.insert(
144:                         http::header::ACCEPT,
145:                         HeaderValue::from_static("text/event-stream"),
146:                     );
147:                     req.compression = request_compression;
148:                 },
149:             )
150:             .await?;
151: 
152:         Ok(spawn_response_stream(
153:             stream_response,
154:             self.session.provider().stream_idle_timeout,
155:             self.sse_telemetry.clone(),
```

## codex-rs/codex-api/src/common.rs:278-307
https://github.com/openai/codex/blob/01fc69f4026735edfdf6789820549727a4867b11/codex-rs/codex-api/src/common.rs#L278
```rust
278: #[derive(Debug, Serialize, Clone, PartialEq)]
279: pub struct ResponsesApiRequest {
280:     pub model: String,
281:     #[serde(skip_serializing_if = "String::is_empty")]
282:     pub instructions: String,
283:     pub input: Vec<ResponseItem>,
284:     #[serde(skip_serializing_if = "Option::is_none")]
285:     pub tools: Option<ResponsesApiTools>,
286:     pub tool_choice: String,
287:     pub parallel_tool_calls: bool,
288:     pub reasoning: Option<Reasoning>,
289:     pub store: bool,
290:     pub stream: bool,
291:     #[serde(skip_serializing_if = "Option::is_none")]
292:     pub stream_options: Option<StreamOptions>,
293:     pub include: Vec<String>,
294:     #[serde(skip_serializing_if = "Option::is_none")]
295:     pub service_tier: Option<String>,
296:     #[serde(skip_serializing_if = "Option::is_none")]
297:     pub prompt_cache_key: Option<String>,
298:     #[serde(skip_serializing_if = "Option::is_none")]
299:     pub text: Option<TextControls>,
300:     #[serde(skip_serializing_if = "Option::is_none")]
301:     pub client_metadata: Option<HashMap<String, String>>,
302:     #[serde(skip_serializing_if = "Option::is_none")]
303:     pub access_programs: Option<AccessPrograms>,
304: }
305: 
306: impl<'a> From<&'a ResponsesApiRequest> for ResponseCreateWsRequest<'a> {
307:     fn from(request: &'a ResponsesApiRequest) -> Self {
```
