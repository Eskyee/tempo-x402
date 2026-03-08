use actix_web::{web, HttpResponse};
use crate::db;
use crate::state::NodeState;

pub(crate) fn is_valid_uuid(s: &str) -> bool {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 5 { return false; }
    let expected_lens = [8, 4, 4, 4, 12];
    parts.iter().zip(expected_lens.iter()).all(|(part, &len)| part.len() == len && part.chars().all(|c| c.is_ascii_hexdigit()))
}

#[allow(dead_code)]
fn is_valid_evm_address(s: &str) -> bool {
    if s.is_empty() { return true; }
    s.len() == 42 && s.starts_with("0x") && s[2..].chars().all(|c| c.is_ascii_hexdigit())
}

#[allow(dead_code)]
fn is_valid_https_url(s: &str) -> bool {
    s.starts_with("https://") && s.len() > 8
}

pub async fn info(state: web::Data<NodeState>) -> HttpResponse {
    let identity_info = state.identity.as_ref().map(|id| {
        serde_json::json!({
            "address": format!("{:#x}", id.address),
            "instance_id": id.instance_id,
            "parent_url": id.parent_url,
            "parent_address": id.parent_address.map(|a| format!("{:#x}", a)),
            "created_at": id.created_at.to_rfc3339(),
        })
    });
    let children = rusqlite::Connection::open(&state.db_path).ok().and_then(|conn| db::query_children_active(&conn).ok()).unwrap_or_default();
    let uptime_secs = (chrono::Utc::now() - state.started_at).num_seconds();
    let clone_available = state.agent.is_some() && state.clone_price.is_some() && (children.len() as u32) < state.clone_max_children;
    HttpResponse::Ok().json(serde_json::json!({
        "identity": identity_info,
        "agent_token_id": state.agent_token_id,
        "children": children,
        "children_count": children.len(),
        "clone_available": clone_available,
        "clone_price": state.clone_price,
        "clone_max_children": state.clone_max_children,
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_seconds": uptime_secs,
    }))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/instance").route("/info", web::get().to(info)));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_valid_uuid() {
        assert!(is_valid_uuid("550e8400-e29b-41d4-a716-446655440000"));
    }
}
