use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse, body::BoxBody, HttpMessage
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::{
    env,
    future::{Ready, ready},
};



//=========================JWT CLAIMS STRUCT==========================

// Structure of data stored inside the JWT token
// Must match the payload used when encoding the token
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {

    // User ID from database
    pub id: i32,

    // Username (subject of the token)
    pub sub: String,

    // Expiration timestamp (Unix time)
    pub exp: usize,

    // Role of the user (admin, manager, pos_staff)
    pub role: String,

    // Full name of the user
    pub fullname: String,

    // Company name the user belongs to
    pub companyname: String,

    // Branch name assigned to user
    pub branchname: String,

    // Company info ID (foreign key reference)
    pub company_info_id: i32,
}

//====================================================================




//========================MIDDLEWARE STRUCT===========================

// Middleware factory struct (used to create middleware per worker thread)
pub struct AuthMiddleware;

//====================================================================




//======================TRANSFORM IMPLEMENTATION======================

// Convert AuthMiddleware into a middleware service
// S represents the next service in the pipeline (route handler)
impl<S> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    // Called once when middleware is initialized
    fn new_transform(&self, service: S) -> Self::Future {

        // Wrap the next service inside middleware service
        ready(Ok(AuthMiddlewareService { service }))
    }
}

//====================================================================




//======================MIDDLEWARE SERVICE===========================

// Middleware service that wraps actual request handler
pub struct AuthMiddlewareService<S> {
    service: S,
}

//====================================================================




//====================SERVICE IMPLEMENTATION=========================

impl<S> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    // Delegate readiness check to inner service
    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {

        // ─────────────────────────────────────────────────────────────
        // STEP 1: Extract Authorization header from request
        // ─────────────────────────────────────────────────────────────
        let auth_header = match req.headers().get("Authorization") {

            // Convert header value to String
            Some(h) => h.to_str().unwrap_or("").to_owned(),

            // If missing, reject request
            None => {
                return Box::pin(async move {
                    Ok(ServiceResponse::new(
                        req.into_parts().0,
                        HttpResponse::Unauthorized().json(serde_json::json!({"error": "Missing Authorization header"})),
                    ))
                });
            }
        };

        // ─────────────────────────────────────────────────────────────
        // STEP 2: Validate Bearer token format
        // ─────────────────────────────────────────────────────────────
        if !auth_header.starts_with("Bearer ") {
            return Box::pin(async move {
                Ok(ServiceResponse::new(
                    req.into_parts().0,
                    HttpResponse::Unauthorized().json(serde_json::json!({"error": "Invalid token format"})),
                ))
            });
        }

        // Extract raw JWT token (remove "Bearer ")
        let token = auth_header.trim_start_matches("Bearer ").trim().to_owned();

        // ─────────────────────────────────────────────────────────────
        // STEP 3: Load JWT secret from environment
        // ─────────────────────────────────────────────────────────────
        let secret = match env::var("JWT_SECRET") {

            // Secret found
            Ok(val) => val,

            // Missing secret = server misconfiguration
            Err(_) => {
                eprintln!("JWT_SECRET is not set in environment!");

                return Box::pin(async move {
                    Ok(ServiceResponse::new(
                        req.into_parts().0,
                        HttpResponse::InternalServerError().json(serde_json::json!({"error": "Server misconfiguration"}))
                    ))
                });
            }
        };

        // ─────────────────────────────────────────────────────────────
        // STEP 4: Decode and validate JWT token
        // ─────────────────────────────────────────────────────────────

        // Use HS256 algorithm
        let mut validation = Validation::new(Algorithm::HS256);

        // Only require "exp" claim
        validation.set_required_spec_claims(&["exp"]);

        let token_data = match decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret.as_ref()),
            &validation,
        ) {
            // Token valid
            Ok(c) => c,

            // Invalid or expired token
            Err(_) => {
                return Box::pin(async move {
                    Ok(ServiceResponse::new(
                        req.into_parts().0,
                        HttpResponse::Unauthorized().json(serde_json::json!({"error": "Invalid or expired token"})),
                    ))
                });
            }
        };

        // ─────────────────────────────────────────────────────────────
        // STEP 5: Store claims inside request extensions
        // ─────────────────────────────────────────────────────────────

        // These claims can be accessed later in handlers
        req.extensions_mut().insert(token_data.claims);

        // ─────────────────────────────────────────────────────────────
        // STEP 6: Forward request to next service (route handler)
        // ─────────────────────────────────────────────────────────────

        let fut = self.service.call(req);

        Box::pin(async move {

            // Wait for handler response and return it
            let res = fut.await?;
            Ok(res)
        })
    }
}

//====================================================================


