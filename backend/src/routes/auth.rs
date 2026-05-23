use actix_web::{
    post,
    web, HttpResponse, Responder,
};
use serde::{Serialize, Deserialize};
use sqlx::{PgPool};
use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::{SaltString, rand_core::OsRng, PasswordHash, PasswordVerifier as _};
use jsonwebtoken::{encode, Header, EncodingKey};
use chrono::{Utc, Duration};
use crate::state::AppState;




//=========================DATA TRANSFER OBJECTS======================

// Internal JWT claims structure (used only inside this auth module)
// NOTE: This is separate from middleware Claims struct, even if similar
#[derive(Serialize, Deserialize)]
struct Claims {

    // User ID from database
    id: i32,

    // Username (JWT subject)
    sub: String,

    // Token expiration timestamp (Unix time)
    exp: usize,

    // User role (admin, manager, pos_staff)
    role: String,

    // Full name of user
    fullname: String,

    // Company the user belongs to
    companyname: String,

    // Branch assigned to user
    branchname: String,

    // Company info ID (foreign key reference)
    company_info_id: i32,
}

// Registration request payload
# [derive(Serialize, Deserialize)]
pub struct Register {

    // Username chosen by user
    pub username: String,

    // Plain password (will be hashed)
    pub password: String,

    // User full name
    pub fullname: String,

    // Company name during registration
    pub companyname: String,

    // Branch name during registration
    pub branchname: String,
}

// Login request payload
#[derive(Serialize, Deserialize)]
pub struct Login {

    // Username for authentication
    pub username: String,

    // Password for authentication
    pub password: String,
}

//=====================================================================




//====================SERVICE STRUCT (OOP-LIKE DESIGN)================

// Authentication service holding DB pool + JWT secret
# [derive(Clone)]
pub struct AuthService {

    // PostgreSQL connection pool
    pool: PgPool,

    // Secret key used to sign JWT tokens
    jwt_secret: String,
}

//====================================================================




//=====================ERROR HANDLING ENUM============================

// Custom authentication-related errors
#[derive(Debug)]
pub enum CustomError {

    // Invalid username or password
    InvalidCredentials,

    // Database query failure
    DatabaseError,

    // Unexpected internal error (e.g. hashing, JWT encoding)
    InternalServerError,
}

// Convert errors into readable strings
impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        // Map error variants to messages
        let msg = match self {
            CustomError::InvalidCredentials => "Invalid authentication credentials",
            CustomError::DatabaseError => "Database error",
            CustomError::InternalServerError => "Internal server error",
        };

        // Output formatted message
        write!(f, "{}", msg)
    }
}

//====================================================================




//============SERVICE IMPLEMENTATION (BUSINESS LOGIC)=================

impl AuthService {

    // Constructor for AuthService
    pub fn new(pool: PgPool, jwt_secret: String) -> Self {
        Self{ pool, jwt_secret }
    }

    // ======================= LOGIN ===============================
    pub async fn login(&self, data:Login) -> Result<String, CustomError> {

        // Extract login fields
        let username = &data.username;
        let password = &data.password;

        // Fetch user record from database
        let user_data = sqlx::query!(
            "SELECT password, user_id, role, full_name, users.company_info_id::INT AS company_info_id, company_info.company_name, company_info.branch_name
            FROM users 
            JOIN company_info ON users.company_info_id = company_info.company_info_id
            WHERE username = $1",
            username,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            eprintln!("Database error: {:?}", e);
            CustomError::DatabaseError
        })?
        .ok_or(CustomError::InvalidCredentials)?;

        // Stored hashed password from DB
        let hash_password = user_data.password;

        // Parse stored hash string into Argon2 format
        let parsed_hash = match PasswordHash::new(&hash_password) {

            // Valid hash format
            Ok(hash) => hash,

            // Invalid hash format (corrupted DB data)
            Err(_) => {
                eprintln!("⚠️ Internal error: Invalid hash format for user {:?}", username);
                return Err(CustomError::InternalServerError);
            }
        };

        // Create Argon2 verifier
        let argon2 = Argon2::default();

        // Verify entered password against stored hash
        if argon2.verify_password(password.as_bytes(), &parsed_hash).is_err() {
            return Err(CustomError::InvalidCredentials);
        }

        // ======================= JWT CREATION =======================

        // Token expiration (24 hours from now)
        let expiration = (Utc::now() + Duration::hours(24)).timestamp() as usize;

        // Build JWT claims payload
        let claims = Claims {
            id: user_data.user_id,
            sub: username.to_string(),
            exp: expiration,
            role: user_data.role,
            fullname: user_data.full_name,
            companyname: user_data.company_name,
            company_info_id: user_data.company_info_id,
            branchname: user_data.branch_name,
        };

        // Sign JWT using secret key
        let token_result = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        );

        // Handle JWT creation result
        let token = match token_result {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Failed to create JWT: {}", e);
                return Err(CustomError::InternalServerError);
            }
        };

        // Return JWT token to caller
        Ok(token)
    }

    // ======================= REGISTER ===========================
    pub async fn register(&self, data:Register) -> Result<(), CustomError> {

        // Extract registration data
        let username = &data.username;
        let password = &data.password;
        let role = "admin"; // default role for new users
        let fullname = &data.fullname;
        let companyname = &data.companyname;
        let branchname = &data.branchname;

        // Generate random salt for hashing
        let salt = SaltString::generate(&mut OsRng);

        // Create Argon2 hasher instance
        let argon2 = Argon2::default();

        // Hash password securely
        let password_hash = match argon2.hash_password(password.as_bytes(), &salt) {
            Ok(hash) => hash.to_string(),
            Err(err) => {
                eprintln!("Error hashing password: {:?}", err);
                return Err(CustomError::InternalServerError);
            }
        };

        let company_info_db = sqlx::query!(
            "INSERT INTO company_info (company_name, branch_name) VALUES ($1, $2) RETURNING company_info_id",
            companyname,
            branchname,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| {
            eprintln!("DB error: {:?}", err);
            CustomError::DatabaseError
        })?
        .company_info_id;

        // Insert new user into database
        let users_db = sqlx::query!(
            "INSERT INTO users (username, password, role, full_name, company_info_id) VALUES ($1, $2, $3, $4, $5)",
            username,
            password_hash,
            role,
            fullname,
            company_info_db,
        )
        .execute(&self.pool)
        .await;

        // Handle DB result
        match users_db {
            Ok(_) => Ok(()),
            Err(err) => {
                eprintln!("DB error: {:?}", err);
                Err(CustomError::DatabaseError)
            }
        }
    }

}

//====================================================================




//=========================API ENDPOINTS==============================

// POST /login endpoint
#[post("/login")]
pub async fn login(
    data: Result<web::Json<Login>, actix_web::Error>,
    service: web::Data<AppState>,
) -> impl Responder {

    // Validate incoming JSON
    let data = match data {
        Ok(d) => d.into_inner(),
        Err(_) => return HttpResponse::BadRequest().body("Invalid input"),
    };

    // Call service login logic
    match service.auth.login(data).await {

        // Success → return JWT token
        Ok(token) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Login successful",
            "token": token
        })),

        // Error handling
        Err(e) => match e {
            CustomError::InvalidCredentials => {
                HttpResponse::Unauthorized().body(e.to_string())
            }

            CustomError::DatabaseError => {
                HttpResponse::InternalServerError().body(e.to_string())
            }

            CustomError::InternalServerError => {
                HttpResponse::InternalServerError().body(e.to_string())
            }
        }
    }
}

// POST /register endpoint
#[post("/register")]
pub async fn register(
    data: Result<web::Json<Register>, actix_web::Error>,
    service: web::Data<AppState>,
) -> impl Responder {

    // Validate incoming JSON
    let data = match data {
        Ok(d) => d.into_inner(),
        Err(_) => return HttpResponse::BadRequest().body("Invalid input"),
    };

    // Call service register logic
    match service.auth.register(data).await {

        // Success response
        Ok(_) => HttpResponse::Created().json(serde_json::json!({
            "message": "User registered successfully"
        })),

        // Error handling
        Err(e) => match e {
            CustomError::InvalidCredentials => {
                HttpResponse::Unauthorized().body(e.to_string())
            }

            CustomError::DatabaseError => {
                HttpResponse::InternalServerError().body(e.to_string())
            }

            CustomError::InternalServerError => {
                HttpResponse::InternalServerError().body(e.to_string())
            }
        }
    }
}

//====================================================================
