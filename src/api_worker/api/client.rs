use worker::{Headers, Method, ObjectNamespace, Request, RequestInit, Response};

pub struct DurableObjectClient {
    namespace: ObjectNamespace,
}

impl DurableObjectClient {
    pub fn new(namespace: ObjectNamespace) -> Self {
        Self { namespace }
    }

    pub async fn websocket_upgrade(&self, page: &str) -> worker::Result<Response> {
        let stub = self.namespace.get_by_name(page)?;

        let headers = Headers::new();
        headers.append("Upgrade", "websocket")?;

        let mut init = RequestInit::new();
        init.with_method(Method::Get).with_headers(headers);

        let request = Request::new_with_init("https://_.com/websocket", &init)?;

        stub.fetch_with_request(request).await
    }
}
