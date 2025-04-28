use axum::{body::Body, response::Response, extract::Request};
use futures_util::future::BoxFuture;
use tower::{Service, Layer};
use std::task::{Context, Poll};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::models::AuthLayer;
use crate::services::jwt_service::*;