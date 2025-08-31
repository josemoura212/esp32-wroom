use std::sync::{Arc, Mutex};
use url::form_urlencoded;

pub fn init_routes(
    req: esp_idf_svc::http::server::Request<&mut esp_idf_svc::http::server::EspHttpConnection<'_>>,
    request_count: Arc<Mutex<u32>>,
    last_params: Arc<Mutex<String>>,
) -> Result<(), anyhow::Error> {
    let mut count = request_count.lock().unwrap();
    *count += 1;
    let current_count = *count;
    drop(count);

    let params = req
        .uri()
        .split('?')
        .nth(1)
        .and_then(|q| {
            form_urlencoded::parse(q.as_bytes())
                .filter_map(|(_k, v)| {
                    let s: String = v.into_owned();
                    if s.is_empty() {
                        None
                    } else {
                        Some(s)
                    }
                })
                .next()
        })
        .unwrap_or_else(|| "Nenhum".into());

    {
        let mut last = last_params.lock().unwrap();
        *last = params.clone();
    }

    println!("Request #{}: {}", current_count, params);

    let mut resp = req.into_ok_response()?;
    resp.write(format!("Request #{} - Params: {}", current_count, params).as_bytes())?;
    Ok(())
}
