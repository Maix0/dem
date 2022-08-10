use anyhow::anyhow;
use reqwest::{Request, Response};
use reqwest_middleware::*;
use task_local_extensions::Extensions;

static MAXIMUM_NUMBER_OF_RETRIES: u32 = 10;

pub struct DiscordRateLimitMiddleware;

#[async_trait]
impl Middleware for DiscordRateLimitMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        // TODO: Ideally we should create a new instance of the `Extensions` map to pass
        // downstream. This will guard against previous retries poluting `Extensions`.
        // That is, we only return what's populated in the typemap for the last retry attempt
        // and copy those into the the `global` Extensions map.
        self.execute_with_retry(req, next, extensions).await
    }
}


impl DiscordRateLimitMiddleware {
    /// This function will try to execute the request, if it fails
    /// with an error classified as transient it will call itself
    /// to retry the request.
    async fn execute_with_retry<'a>(
        &'a self,
        req: Request,
        next: Next<'a>,
        ext: &'a mut Extensions,
    ) -> Result<Response> {
        let mut n_past_retries = 0;
        loop {
            // Cloning the request object before-the-fact is not ideal..
            // However, if the body of the request is not static, e.g of type `Bytes`,
            // the Clone operation should be of constant complexity and not O(N)
            // since the byte abstraction is a shared pointer over a buffer.
            let duplicate_request = req.try_clone().ok_or_else(|| {
                Error::Middleware(anyhow!(
                    "Request object is not clonable. Are you passing a streaming body?".to_string()
                ))
            })?;

            let result = next.clone().run(duplicate_request, ext).await;

            // We classify the response which will return None if not
            // errors were returned.
            break {
                match result {
                    Ok(r) if n_past_retries < MAXIMUM_NUMBER_OF_RETRIES => {
                        // If the response failed and the error type was transient
                        // we can safely try to retry the request.
                        let retry_decicion = Self::should_retry(&r);
                        if let Retry::After(dur) = retry_decicion {
                            // Sleep the requested amount before we try again.
                            tokio::time::sleep(dur).await;
                            n_past_retries += 1;
                            continue;
                        } else {
                            Ok(r)
                        }
                    }
                    v => v,
                }
            };
        }
    }

    fn should_retry(res: &Response) -> Retry {
        if res.status().is_success() {
            return Retry::No;
        }
        if let Some(Ok(v)) = res
            .headers()
            .get("X-RateLimit-Reset-After")
            .map(|v| v.to_str())
        {
            let dur_f32: f32 = v.parse().unwrap();
            if dur_f32 <= 0.0 {
                return Retry::No;
            }
            return Retry::After(std::time::Duration::from_secs_f32(dur_f32));
        } else {
            Retry::No
        }
    }
}

enum Retry {
    After(std::time::Duration),
    No,
}
