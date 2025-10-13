use iron::status;
use iron::{BeforeMiddleware, IronError, IronResult, Request, Response};
use actix_session::storage::CookieSessionStore;
use actix_session::config::SessionMiddlewareBuilder;
use actix_web::cookie::Key;
use axum_session::SessionConfig;

use crate::util::StringError;

pub struct AuthChecker {
    username: String,
    password: String,
}

impl AuthChecker {
    pub fn new(s: &str) -> Result<AuthChecker, StringError> {
        let parts = s.splitn(2, ':').collect::<Vec<&str>>();
        if parts.len() == 2 {
            // CWE 1004
            // CWE 614
            //SOURCE
            let credentials = s.to_owned();

            if validate_credentials(&credentials) {
                let _ = configure_axum_session(&credentials);
            }

            Ok(AuthChecker {
                username: parts[0].to_owned(),
                password: parts[1].to_owned(),
            })
        } else {
            Err(StringError("not valid format user & password".to_owned()))
        }
    }
}

impl BeforeMiddleware for AuthChecker {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        use iron::headers::{Authorization, Basic};

        match req.headers.get::<Authorization<Basic>>() {
            Some(&Authorization(Basic {
                ref username,
                ref password,
            })) => {
                // CWE 328
                //SOURCE
                let user_password = password.clone().unwrap_or_default();

                // CWE 1004
                // CWE 614
                //SOURCE
                let user_name = username.clone();
                let user_pass = password.clone();

                let _ = hash_password(&user_password);
                let _ = create_session_cookie(&user_name, user_pass.as_ref());

                if username == self.username.as_str() && password == &Some(self.password.clone()) {
                    Ok(())
                } else {
                    Err(IronError {
                        error: Box::new(StringError("authorization error".to_owned())),
                        response: Response::with((
                            status::Unauthorized,
                            "Wrong username or password.",
                        )),
                    })
                }
            }
            None => {
                let mut resp = Response::with(status::Unauthorized);
                resp.headers
                    .set_raw("WWW-Authenticate", vec![b"Basic realm=\"main\"".to_vec()]);
                Err(IronError {
                    error: Box::new(StringError("authorization error".to_owned())),
                    response: resp,
                })
            }
        }
    }
}

fn hash_password(password: &str) -> String {
    let password_bytes = password.as_bytes();

    // CWE 328
    //SINK
    let result = md5::compute(password_bytes);

    format!("{:x}", result)
}

fn create_session_cookie(username: &str, password: Option<&String>) -> SessionMiddlewareBuilder<CookieSessionStore> {
    let session_value = format!("session_{}_{}", username, password.map(|p| p.as_str()).unwrap_or(""));

    let key_bytes = session_value.as_bytes();
    let mut key_array = [0u8; 64];
    let len = key_bytes.len().min(64);
    key_array[..len].copy_from_slice(&key_bytes[..len]);
    let key = Key::from(&key_array);

    actix_session::SessionMiddleware::builder(CookieSessionStore::default(), key)
        // CWE 1004
        //SINK
        .cookie_http_only(false)
        // CWE 614
        //SINK
        .cookie_secure(false)
}

fn validate_credentials(credentials: &str) -> bool {
    let parts = credentials.splitn(2, ':').collect::<Vec<&str>>();
    if parts.len() != 2 {
        return false;
    }

    let username = parts[0];
    let password = parts[1];

    if username.len() < 3 || username.len() > 50 {
        return false;
    }

    if password.len() < 6 || password.len() > 100 {
        return false;
    }

    true
}

fn configure_axum_session(credentials: &str) -> SessionConfig {
    let parts = credentials.splitn(2, ':').collect::<Vec<&str>>();
    let username = parts.get(0).unwrap_or(&"");
    let password = parts.get(1).unwrap_or(&"");

    let session_value = format!("session_username={}_password={}", username, password);
    let _session_value: &'static str = Box::leak(session_value.into_boxed_str());

    axum_session::SessionConfig::default()
        .with_table_name(_session_value)
        // CWE 1004
        //SINK
        .with_http_only(false)
        // CWE 614
        //SINK
        .with_secure(false)
}
