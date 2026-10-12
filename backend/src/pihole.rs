use std::time::Duration;

use reqwest::StatusCode;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Mutex;

/// Quantos domínios bloqueados entram no "top" do card.
const TOP_BLOCKED_COUNT: u32 = 5;

#[derive(Debug, Error)]
pub enum PiholeError {
    #[error("falha ao falar com o Pi-hole: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Pi-hole recusou o login: {0}")]
    Auth(String),
}

/// O que o card do dashboard mostra. As contagens de queries são das
/// últimas 24h — é a janela que o `/api/stats/summary` do Pi-hole usa.
#[derive(Debug, Serialize, PartialEq)]
pub struct PiholeSummary {
    /// "enabled", "disabled", "failed" ou "unknown", como o Pi-hole devolve.
    pub blocking: String,
    pub queries_total: u64,
    pub queries_blocked: u64,
    pub percent_blocked: f64,
    pub unique_domains: u64,
    pub clients_active: u64,
    pub domains_on_blocklist: u64,
    /// Unix timestamp da última atualização das listas (gravity).
    pub gravity_last_update: Option<i64>,
    pub top_blocked: Vec<TopDomain>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TopDomain {
    pub domain: String,
    pub count: u64,
}

// --- formato da API v6 do Pi-hole: só os campos que usamos ---

#[derive(Deserialize)]
struct AuthResponse {
    session: Session,
}

#[derive(Deserialize)]
struct Session {
    valid: bool,
    sid: Option<String>,
    message: Option<String>,
}

#[derive(Deserialize)]
struct StatsSummary {
    queries: Queries,
    clients: Clients,
    gravity: Gravity,
}

#[derive(Deserialize)]
struct Queries {
    total: u64,
    blocked: u64,
    percent_blocked: f64,
    unique_domains: u64,
}

#[derive(Deserialize)]
struct Clients {
    active: u64,
}

#[derive(Deserialize)]
struct Gravity {
    domains_being_blocked: u64,
    last_update: Option<i64>,
}

#[derive(Deserialize)]
struct TopDomains {
    domains: Vec<TopDomain>,
}

#[derive(Deserialize)]
struct Blocking {
    blocking: String,
}

/// Cliente da API v6 do Pi-hole.
///
/// A v6 autentica por sessão: `POST /api/auth` com a senha devolve um `sid`,
/// que vai no header `X-FTL-SID` das próximas requests. O Pi-hole tem um
/// limite de sessões simultâneas (`webserver.api.max_sessions`, 16 por
/// padrão), então logar a cada poll esgotaria as vagas em ~1min — o `sid` é
/// guardado e reaproveitado, e só refeito quando o Pi-hole responde 401
/// (sessão expirou ou o Pi-hole reiniciou).
pub struct PiholeClient {
    http: reqwest::Client,
    base_url: String,
    password: String,
    /// tokio::Mutex, e não std: o lock fica segurado durante o login (que é
    /// `.await`), justamente pra duas requests simultâneas não abrirem duas
    /// sessões.
    sid: Mutex<Option<String>>,
}

impl PiholeClient {
    /// Lê `PIHOLE_URL` (ex: `http://host.docker.internal:8082`) e
    /// `PIHOLE_PASSWORD`. Sem alguma das duas, o Pi-hole fica desligado.
    pub fn from_env() -> Option<Self> {
        let base_url = non_empty_env("PIHOLE_URL")?;
        let password = non_empty_env("PIHOLE_PASSWORD")?;

        // Sem timeout, um Pi-hole travado seguraria a request do frontend
        // pra sempre (e o usePolling pula os polls enquanto ela não volta).
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .ok()?;

        Some(Self {
            http,
            base_url: base_url.trim_end_matches('/').to_string(),
            password,
            sid: Mutex::new(None),
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Faz login só pra validar URL e senha no boot e logar o resultado.
    pub async fn check(&self) -> Result<(), PiholeError> {
        self.session().await.map(|_| ())
    }

    pub async fn summary(&self) -> Result<PiholeSummary, PiholeError> {
        let top_path = format!("/api/stats/top_domains?blocked=true&count={TOP_BLOCKED_COUNT}");
        // As três chamadas são independentes: em paralelo, a latência é a da
        // mais lenta, não a soma.
        let (stats, top, blocking) = tokio::try_join!(
            self.get::<StatsSummary>("/api/stats/summary"),
            self.get::<TopDomains>(&top_path),
            self.get::<Blocking>("/api/dns/blocking"),
        )?;
        Ok(build_summary(stats, top, blocking))
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, PiholeError> {
        let sid = self.session().await?;
        let mut res = self.send(path, &sid).await?;

        if res.status() == StatusCode::UNAUTHORIZED {
            self.invalidate(&sid).await;
            let sid = self.session().await?;
            res = self.send(path, &sid).await?;
        }

        Ok(res.error_for_status()?.json().await?)
    }

    async fn send(&self, path: &str, sid: &str) -> Result<reqwest::Response, PiholeError> {
        Ok(self
            .http
            .get(format!("{}{path}", self.base_url))
            .header("X-FTL-SID", sid)
            .send()
            .await?)
    }

    /// Devolve o `sid` guardado ou faz login. As requests em paralelo do
    /// `summary` passam todas por aqui: a primeira loga, as outras esperam o
    /// lock e já encontram o `sid` pronto.
    async fn session(&self) -> Result<String, PiholeError> {
        let mut guard = self.sid.lock().await;
        if let Some(sid) = guard.as_ref() {
            return Ok(sid.clone());
        }

        let res: AuthResponse = self
            .http
            .post(format!("{}/api/auth", self.base_url))
            .json(&serde_json::json!({ "password": self.password }))
            .send()
            .await?
            .json()
            .await?;

        let sid = session_sid(res.session)?;
        tracing::info!("nova sessão aberta no Pi-hole");
        *guard = Some(sid.clone());
        Ok(sid)
    }

    /// Descarta o `sid` que levou 401 — mas só se ainda for ele. Se outra
    /// request já relogou nesse meio-tempo, o `sid` novo é mantido.
    async fn invalidate(&self, stale: &str) {
        let mut guard = self.sid.lock().await;
        if guard.as_deref() == Some(stale) {
            *guard = None;
        }
    }
}

fn session_sid(session: Session) -> Result<String, PiholeError> {
    match session {
        Session { valid: true, sid: Some(sid), .. } => Ok(sid),
        // Pi-hole sem senha devolve sessão válida sem sid; o Sentinel exige
        // senha configurada, então isso é tratado como erro de configuração.
        Session { valid: true, sid: None, .. } => Err(PiholeError::Auth(
            "o Pi-hole não tem senha configurada e não devolveu sessão".into(),
        )),
        Session { message, .. } => Err(PiholeError::Auth(
            message.unwrap_or_else(|| "senha incorreta".into()),
        )),
    }
}

fn build_summary(stats: StatsSummary, top: TopDomains, blocking: Blocking) -> PiholeSummary {
    PiholeSummary {
        blocking: blocking.blocking,
        queries_total: stats.queries.total,
        queries_blocked: stats.queries.blocked,
        percent_blocked: stats.queries.percent_blocked,
        unique_domains: stats.queries.unique_domains,
        clients_active: stats.clients.active,
        domains_on_blocklist: stats.gravity.domains_being_blocked,
        // o Pi-hole devolve 0 quando o gravity nunca rodou
        gravity_last_update: stats.gravity.last_update.filter(|&t| t > 0),
        top_blocked: top.domains,
    }
}

fn non_empty_env(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Respostas reais (recortadas) de um Pi-hole v6.4 — se a API mudar de
    // formato, é aqui que quebra primeiro.
    const SUMMARY_JSON: &str = r#"{
        "queries": {"total": 1200, "blocked": 300, "percent_blocked": 25.0,
                    "unique_domains": 140, "forwarded": 700, "cached": 200,
                    "types": {"A": 900}},
        "clients": {"active": 4, "total": 6},
        "gravity": {"domains_being_blocked": 72526, "last_update": 1791700441},
        "took": 0.0001
    }"#;
    const TOP_JSON: &str = r#"{
        "domains": [{"domain": "ads.example.com", "count": 120},
                    {"domain": "tracker.example.net", "count": 80}],
        "total_queries": 1200, "blocked_queries": 300, "took": 0.0001
    }"#;
    const BLOCKING_JSON: &str = r#"{"blocking": "enabled", "timer": null, "took": 0.0001}"#;

    #[test]
    fn monta_resumo_a_partir_das_respostas_da_api() {
        let summary = build_summary(
            serde_json::from_str(SUMMARY_JSON).unwrap(),
            serde_json::from_str(TOP_JSON).unwrap(),
            serde_json::from_str(BLOCKING_JSON).unwrap(),
        );

        assert_eq!(summary.blocking, "enabled");
        assert_eq!(summary.queries_total, 1200);
        assert_eq!(summary.queries_blocked, 300);
        assert_eq!(summary.percent_blocked, 25.0);
        assert_eq!(summary.unique_domains, 140);
        assert_eq!(summary.clients_active, 4);
        assert_eq!(summary.domains_on_blocklist, 72526);
        assert_eq!(summary.gravity_last_update, Some(1791700441));
        assert_eq!(summary.top_blocked.len(), 2);
        assert_eq!(summary.top_blocked[0].domain, "ads.example.com");
    }

    #[test]
    fn gravity_que_nunca_rodou_vira_none() {
        let mut stats: StatsSummary = serde_json::from_str(SUMMARY_JSON).unwrap();
        stats.gravity.last_update = Some(0);
        let summary = build_summary(
            stats,
            serde_json::from_str(TOP_JSON).unwrap(),
            serde_json::from_str(BLOCKING_JSON).unwrap(),
        );
        assert_eq!(summary.gravity_last_update, None);
    }

    #[test]
    fn login_valido_devolve_sid() {
        let res: AuthResponse = serde_json::from_str(
            r#"{"session": {"valid": true, "totp": false, "sid": "abc", "csrf": "x",
                            "validity": 1800, "message": "password correct"}}"#,
        )
        .unwrap();
        assert_eq!(session_sid(res.session).unwrap(), "abc");
    }

    #[test]
    fn senha_errada_vira_erro_de_auth() {
        let res: AuthResponse = serde_json::from_str(
            r#"{"session": {"valid": false, "totp": false, "sid": null,
                            "validity": -1, "message": "password incorrect"}}"#,
        )
        .unwrap();
        let err = session_sid(res.session).unwrap_err();
        assert!(matches!(err, PiholeError::Auth(ref m) if m == "password incorrect"));
    }
}
