use std::net::SocketAddr;
use std::sync::Arc;

use cctf_rs::libs::{
    api::{self, AppState, RateLimiter},
    repos::pg::PgStore,
    services::{
        email::{HttpCatcher, HttpCatcherConfig},
        AuthService, ConfigService, OAuthService, ScoreboardService, SolveService,
    }
}
