use crate::protocol::{handshake::password_method, AuthMethod, UserKey};
use as_any::AsAny;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::net::TcpStream;

/// This trait is for defining the socks5 authentication method.
///
/// Pre-defined authentication methods can be found in the [`auth`](https://docs.rs/socks5-impl/latest/socks5_impl/server/auth/index.html) module.
///
/// You can create your own authentication method by implementing this trait. Since GAT is not stabled yet,
/// [async_trait](https://docs.rs/async-trait/latest/async_trait/index.html) needs to be used.
///
/// # Example
/// ```rust
/// use async_trait::async_trait;
/// use socks5_impl::protocol::AuthMethod;
/// use socks5_impl::server::AuthExecutor;
/// use tokio::net::TcpStream;
///
/// pub struct MyAuth;
///
/// #[async_trait]
/// impl AuthExecutor for MyAuth {
///     type Output = std::io::Result<usize>;
///
///     fn auth_method(&self) -> AuthMethod {
///         AuthMethod::from(0x80)
///     }
///
///     async fn execute(&self, stream: &mut TcpStream) -> Self::Output {
///         // do something
///         Ok(1145141919810)
///     }
/// }
/// ```
#[async_trait]
pub trait AuthExecutor {
    type Output: AsAny;
    fn auth_method(&self) -> AuthMethod;
    async fn execute(&self, stream: &mut TcpStream) -> Self::Output;
}

pub type AuthAdaptor<O> = Arc<dyn AuthExecutor<Output = O> + Send + Sync>;

/// No authentication as the socks5 handshake method.
#[derive(Debug, Default)]
pub struct NoAuth;

#[async_trait]
impl AuthExecutor for NoAuth {
    type Output = ();
    fn auth_method(&self) -> AuthMethod {
        AuthMethod::NoAuth
    }

    async fn execute(&self, _: &mut TcpStream) -> Self::Output {}
}

/// Username and password as the socks5 handshake method.
pub struct UserKeyAuth {
    user_key: UserKey,
}

impl UserKeyAuth {
    pub fn new(username: &str, password: &str) -> Self {
        let user_key = UserKey::new(username, password);
        Self { user_key }
    }
}

#[async_trait]
impl AuthExecutor for UserKeyAuth {
    type Output = std::io::Result<bool>;

    fn auth_method(&self) -> AuthMethod {
        AuthMethod::UserPass
    }

    async fn execute(&self, stream: &mut TcpStream) -> Self::Output {
        let req = password_method::Request::rebuild_from_stream(stream).await?;

        let is_equal = req.user_key == self.user_key;
        let resp = password_method::Response::new(is_equal);
        resp.write_to_stream(stream).await?;
        if is_equal {
            Ok(true)
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "username or password is incorrect"))
        }
    }
}
