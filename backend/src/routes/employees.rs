use actix_web::{
    post,
    web, HttpResponse, Responder, HttpRequest,HttpMessage
};
use crate::middleware::auth_middleware::Claims;
use serde::{Serialize, Deserialize};
use sqlx::{PgPool};
use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::{SaltString, rand_core::OsRng};
use crate::state::AppState;




//================DTOs OR Data Transfer Objects.=======================

// Struct used to receive employee data from the frontend request body
#[derive(Serialize, Deserialize)]
pub struct CreateEmployee {

    // Employee login username
    pub username: String,

    // Plain text password from frontend (will be hashed before saving)
    pub password: String,

    // Role to assign to the employee
    pub role: String,

    // Branch where the employee belongs
    pub branchname: String,

    // Employee full name
    pub fullname: String,
}

//=====================================================================




//====================SERVICE STRUCT (OOP-LIKE DESIGN)======================

// Service struct that contains the PostgreSQL connection pool
#[derive(Clone)]
pub struct EmployeeService {

    // Database connection pool
    pool: PgPool,
}

//====================================================================




//=====================ERROR HANDLING ENUM============================

// Custom application errors
#[derive(Debug)]
pub enum CustomError {

    // User is not authorized
    Unauthorized,

    // Database related error
    DatabaseError,

    // Username already exists
    AlreadyExists,

    // Internal server failure
    InternalServerError,
}

// Converts enum errors into readable text messages
impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        // Match each enum variant to its message
        let msg = match self {
            CustomError::Unauthorized => "Unauthorized",
            CustomError::DatabaseError => "Database error",
            CustomError::AlreadyExists => "Employee with this username already exists",
            CustomError::InternalServerError => "Internal server error",
        };

        // Return the message as a formatted string
        write!(f, "{}", msg)
    }
}

//====================================================================




//============OOP METHODS OR STRUCT IMPLEMENTATION IN RUST============

impl EmployeeService {

    // Constructor function for EmployeeService
    pub fn new(pool: PgPool) -> Self {

        // Create and return EmployeeService instance
        Self{pool}
    }

    // Main business logic for creating an employee
    pub async fn create_employee(&self, data:CreateEmployee, claims: Claims) -> Result<(), CustomError> {

        // Get username reference from request data
        let username = &data.username;

        // Get password reference from request data
        let password = &data.password;

        // Determine what role can be created depending on logged-in user role
        let created_role = match claims.role.as_str() {

            // Admin can create manager or pos_staff
            "admin" =>{

                // Safety check to prevent invalid roles
                if data.role != "manager" && data.role != "pos_staff" {
                    return Err(CustomError::Unauthorized);
                }

                // Use the role sent by frontend
                data.role.clone()
            }

            // Manager can only create pos_staff
            "manager" => "pos_staff".to_string(),

            // Any other role is unauthorized
            _ => return Err(CustomError::Unauthorized),
        };

        // Determine branch assignment based on creator role
        let branchname = match claims.role.as_str() {

            // Admin can choose any branch
            "admin" => data.branchname,

            // Manager can only assign employees to their own branch
            "manager" => claims.branchname,

            // Other roles are unauthorized
            _ => return Err(CustomError::Unauthorized),
        };

        // Get employee full name
        let fullname = &data.fullname;

        // Check if username already exists in database
        let exists: bool = sqlx::query_scalar(
            "SELECT 1 FROM users WHERE username = $1",
        )
        .bind(username)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| CustomError::DatabaseError)?;

        // If username already exists, return conflict error
        if exists {
            return Err(CustomError::AlreadyExists);
        }

        // Generate a random salt for password hashing
        let salt = SaltString::generate(&mut OsRng);

        // Create Argon2 password hasher with default settings
        let argon2 = Argon2::default();

        // Hash the password securely
        let password_hash = match argon2.hash_password(password.as_bytes(), &salt) {

            // Convert hash object into string format
            Ok(hash) => hash.to_string(),

            // Handle hashing failure
            Err(err) => {
                eprintln!("Error hashing password: {:?}", err);
                return Err(CustomError::InternalServerError);
            }
        };

        let company_info_db = sqlx::query!(
            "INSERT INTO company_info (company_name, branch_name) VALUES ($1, $2) RETURNING company_info_id",
            claims.companyname,
            branchname,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| {
            eprintln!("DB error: {:?}", err);
            CustomError::DatabaseError
        })?
        .company_info_id;

        // Insert employee data into database
        let query_result = sqlx::query!(
            "INSERT INTO users (username, password, role, full_name, company_info_id) VALUES ($1, $2, $3, $4, $5)",
            username,
            password_hash,
            created_role,
            fullname,
            company_info_db,
        )
        .execute(&self.pool)
        .await;

        // Handle query result
        match query_result {

            // Insert successful
            Ok(_) => Ok(()),

            // Database insertion failed
            Err(err) => {
                eprintln!("DB error: {:?}", err);
                Err(CustomError::DatabaseError)
            }
        }

    }

}

//====================================================================




//=========================API ENDPOINTS==============================

// POST endpoint for creating employees
#[post("/create_employee")]
pub async fn create_employee(

    // JSON request body from frontend
    data: Result<web::Json<CreateEmployee>, actix_web::Error>,

    // Shared application state
    service: web::Data<AppState>,

    // HTTP request object
    req: HttpRequest,
) -> impl Responder {

    // Read JWT claims added by authentication middleware
    let claims = match req.extensions().get::<Claims>().cloned() {

        // Claims found
        Some(c) => c,

        // No claims means unauthorized request
        None => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    // Check if logged-in user has permission to create employees
    match claims.role.as_str() {

        // Allowed roles
        "admin" | "manager" => {}

        // POS staff is forbidden
        "pos_staff" => return HttpResponse::Forbidden().body("Forbidden"),

        // Any unknown role is unauthorized
        _ => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    // Validate incoming JSON body
    let data = match data {

        // Extract JSON data
        Ok(d) => d.into_inner(),

        // Invalid JSON request
        Err(_) => return HttpResponse::BadRequest().body("Invalid input"),
    };

    // Call service layer to create employee
    match service.employees.create_employee(data, claims).await {

        // Success response
        Ok(_) => HttpResponse::Created().json(serde_json::json!({
            "message": "Employee created successfully"
        })),

        // Error responses
        Err(e) => match e {

            // Unauthorized error
            CustomError::Unauthorized => {
                HttpResponse::Unauthorized().body(e.to_string())
            }

            // Database related error
            CustomError::DatabaseError => {
                HttpResponse::InternalServerError().body(e.to_string())
            }

            // Username already exists
            CustomError::AlreadyExists => {
                HttpResponse::Conflict().body(e.to_string())
            }

            // Internal server error
            CustomError::InternalServerError => {
                HttpResponse::InternalServerError().body(e.to_string())
            }
        }
    }
}

//====================================================================
