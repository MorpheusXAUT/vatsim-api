use crate::client::{CachePolicy, StatusResponse, VatsimClient};
use crate::error::ClientError;
use crate::types::datafeed::DataFeed;

impl VatsimClient {
    /// Fetches the VATSIM datafeed.
    ///
    /// With [`CachePolicy::Cached`], returns a cached copy if one exists and is
    /// still within the configured TTL. With [`CachePolicy::Refresh`], always
    /// fetches from the network and overwrites the cache.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError::Http`] if the network request fails or the server
    /// returns a non-success status code. Returns [`ClientError::NoDataUrls`] if
    /// the status endpoint has no usable mirror URLs.
    pub async fn datafeed(&self, policy: CachePolicy) -> Result<DataFeed, ClientError> {
        if policy == CachePolicy::Cached {
            let cache = self.inner.datafeed_cache.read().await;
            if let Some((ref feed, fetched_at)) = *cache {
                if fetched_at.elapsed().as_secs() < self.inner.config.cache_ttl_secs {
                    return Ok(feed.clone());
                }
            }
        }

        let url = self.resolve_datafeed_url().await?;
        let feed: DataFeed = self
            .inner
            .http
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        *self.inner.datafeed_cache.write().await = Some((feed.clone(), std::time::Instant::now()));
        Ok(feed)
    }

    async fn resolve_datafeed_url(&self) -> Result<String, ClientError> {
        if let Some(ref url) = self.inner.config.datafeed_url_override {
            return Ok(url.clone());
        }

        {
            let urls = self.inner.datafeed_urls.read().await;
            if let Some(url) = urls.first() {
                return Ok(url.clone());
            }
        }

        let status: StatusResponse = self
            .inner
            .http
            .get(&self.inner.config.status_url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        *self.inner.datafeed_urls.write().await = status.data.v3.clone();
        status
            .data
            .v3
            .into_iter()
            .next()
            .ok_or(ClientError::NoDataUrls)
    }
}
