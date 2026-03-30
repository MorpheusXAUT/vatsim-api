use crate::client::VatsimClient;
use crate::error::ClientError;
use crate::types::CertificateId;
use crate::types::slurper::UserConnection;

const SLURPER_URL: &str = "https://slurper.vatsim.net/users/info";

impl VatsimClient {
    /// Fetches the active connections for a user from the
    /// [VATSIM slurper API](https://vatsim.dev/api/slurper-api/get-user-info).
    ///
    /// Returns one [`UserConnection`] per active session the user has on the
    /// network (pilot or ATC). Returns an empty `Vec` if the user is offline.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError::Http`] if the network request fails or the server
    /// returns a non-success status code. Returns [`ClientError::Parse`] if a
    /// CSV line cannot be parsed.
    pub async fn user_connections(
        &self,
        cid: CertificateId,
    ) -> Result<Vec<UserConnection>, ClientError> {
        let url = self
            .inner
            .config
            .slurper_url_override
            .as_deref()
            .unwrap_or(SLURPER_URL);

        let body: String = self
            .inner
            .http
            .get(url)
            .query(&[("cid", cid.to_string())])
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        body.lines()
            .enumerate()
            .filter(|(_, line)| !line.trim().is_empty())
            .map(|(i, line)| UserConnection::parse_line(line, i).map_err(ClientError::from))
            .collect()
    }
}
