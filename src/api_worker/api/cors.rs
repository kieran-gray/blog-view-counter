use serde_json::json;
use worker::{Request, Response};

pub struct CorsContext {
    allowed_origins: Vec<String>,
    origin: Option<String>,
}

impl CorsContext {
    pub fn new(allowed_origins: Vec<String>, req: &Request) -> Self {
        let origin = req.headers().get("Origin").ok().flatten();
        Self {
            allowed_origins,
            origin,
        }
    }

    #[cfg(test)]
    fn new_for_test(allowed_origins: Vec<String>, origin: Option<String>) -> Self {
        // We can't use Request objects in tests as they require wasm
        Self {
            allowed_origins,
            origin,
        }
    }

    pub fn validate(&self, req: &Request) -> Result<(), Response> {
        if !self.is_allowed() {
            tracing::error!(
                origin = ?self.origin,
                method = %req.method(),
                path = %req.path(),
                allowed_origins = ?self.allowed_origins,
                "CORS check failed"
            );
            let message = json!({"message": "Forbidden"});
            return Err(Response::from_json(&message).unwrap().with_status(403));
        }
        Ok(())
    }

    pub fn add_to_response(&self, mut response: Response) -> Response {
        if let Some(origin_value) = self.origin.clone()
            && (self.allowed_origins.is_empty() || self.allowed_origins.contains(&origin_value))
        {
            let _ = response
                .headers_mut()
                .set("Access-Control-Allow-Origin", &origin_value);
            let _ = response
                .headers_mut()
                .set("Access-Control-Allow-Methods", "GET, POST, OPTIONS");
            let _ = response.headers_mut().set(
                "Access-Control-Allow-Headers",
                "Content-Type, Authorization",
            );
        }
        response
    }

    pub fn preflight_response(&self) -> worker::Result<Response> {
        let response = Response::empty()?;
        let mut response = self.add_to_response(response);

        response
            .headers_mut()
            .set("Access-Control-Max-Age", "86400")?;

        Ok(response)
    }

    fn is_allowed(&self) -> bool {
        if self.allowed_origins.is_empty() {
            return true;
        }
        self.origin
            .as_ref()
            .is_some_and(|o| self.allowed_origins.contains(o))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_allowed_empty_allowed_list() {
        let cors = CorsContext::new_for_test(vec![], Some("http://example.com".to_string()));
        assert!(cors.is_allowed());

        let cors = CorsContext::new_for_test(vec![], Some("http://evil.com".to_string()));
        assert!(cors.is_allowed());

        let cors = CorsContext::new_for_test(vec![], None);
        assert!(cors.is_allowed());
    }

    #[test]
    fn test_is_allowed_with_allowed_list() {
        let allowed = vec![
            "http://localhost:5173".to_string(),
            "https://example.com".to_string(),
        ];

        let cors =
            CorsContext::new_for_test(allowed.clone(), Some("http://localhost:5173".to_string()));
        assert!(cors.is_allowed());

        let cors =
            CorsContext::new_for_test(allowed.clone(), Some("https://example.com".to_string()));
        assert!(cors.is_allowed());

        let cors = CorsContext::new_for_test(allowed.clone(), Some("http://evil.com".to_string()));
        assert!(!cors.is_allowed());

        let cors =
            CorsContext::new_for_test(allowed.clone(), Some("https://different.com".to_string()));
        assert!(!cors.is_allowed());

        let cors = CorsContext::new_for_test(allowed, None);
        assert!(!cors.is_allowed());
    }

    #[test]
    fn test_is_allowed_exact_match() {
        let allowed = vec!["https://example.com".to_string()];

        let cors =
            CorsContext::new_for_test(allowed.clone(), Some("https://example.com".to_string()));
        assert!(cors.is_allowed());

        let cors =
            CorsContext::new_for_test(allowed.clone(), Some("http://example.com".to_string()));
        assert!(!cors.is_allowed());

        let cors = CorsContext::new_for_test(allowed, Some("https://sub.example.com".to_string()));
        assert!(!cors.is_allowed());
    }
}
