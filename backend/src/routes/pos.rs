use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder,post, web};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, types::BigDecimal};
use crate::middleware::auth_middleware::Claims;
use crate::state::AppState;




//================DTOs OR DATA TRANSFER OBJECTS=======================

// Represents a single product line inside an order (POS cart item)
#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
pub struct OrderItem {

    // ID of the product being purchased
    pub product_id: i32,

    // Quantity of the product being purchased
    pub quantity: i32,

    // Price per unit at the time of purchase (stored as string from frontend)
    pub price: String,
}

// Represents full order request sent from POS checkout
// NOTE: user_id is NOT included here because it is taken from JWT claims
#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
pub struct CreateOrderRequest {

    // List of items in the order
    pub items: Vec<OrderItem>,

    // Total order amount
    pub total: String,

    // Amount paid by customer
    pub amount_paid: String,

    // Change returned to customer
    pub change: String,
}

//=====================================================================




//====================SERVICE STRUCT (OOP-LIKE STRUCT)=================

# [derive(Clone)]
pub struct PosService {

    // PostgreSQL connection pool
    pool: PgPool,
}

//====================================================================




//=====================ERROR HANDLING ENUM============================

// Custom errors used in POS operations
#[derive(Debug)]
pub enum CustomError {

    // Database operation failed
    DatabaseError,

    // Invalid input or request data
    BadRequest,
}

// Display implementation for readable error messages
impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        // Map enum variants to human-readable messages
        let msg = match self {
            CustomError::DatabaseError => "Database error",
            CustomError::BadRequest => "Missing or Invalid Input",
        };

        // Return formatted string
        write!(f, "{}", msg)
    }
}

//====================================================================




//============SERVICE IMPLEMENTATION (BUSINESS LOGIC)=================

impl PosService {

    // Constructor function for PosService
    pub fn new(pool: PgPool) -> Self {
        Self{pool}
    }

    // Create order transaction (main POS checkout logic)
    pub async fn create_order(&self, order: CreateOrderRequest, claim: Claims) -> Result<(), CustomError> {

        // ─────────────────────────────────────────────────────────────
        // STEP 1: Parse monetary values from String → BigDecimal
        // ─────────────────────────────────────────────────────────────

        // Convert total amount
        let total = match order.total.parse::<BigDecimal>() {
            Ok(p) => p,
            Err(_) => return Err(CustomError::BadRequest),
        };

        // Convert amount paid
        let amount_paid = match order.amount_paid.parse::<BigDecimal>() {
            Ok(p) => p,
            Err(_) => return Err(CustomError::BadRequest),
        };

        // Convert change amount
        let change = match order.change.parse::<BigDecimal>() {
            Ok(p) => p,
            Err(_) => return Err(CustomError::BadRequest),
        };

        // ─────────────────────────────────────────────────────────────
        // STEP 2: Start database transaction (atomic operation)
        // ─────────────────────────────────────────────────────────────
        // Ensures all DB operations succeed or fail together
        let mut tx = match self.pool.begin().await {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Failed to start transaction: {:?}", e);
                return Err(CustomError::DatabaseError);
            }
        };

        // ─────────────────────────────────────────────────────────────
        // STEP 3: Insert order into orders table
        // ─────────────────────────────────────────────────────────────
        let inserted_order = match sqlx::query!(
            "INSERT INTO orders (user_id, total, amount_paid, change, created_at,role, company_info_id)
            VALUES ($1, $2, $3, $4, NOW(), $5, $6)
            RETURNING orders_id",
            claim.id,
            total,
            amount_paid,
            change,
            claim.role,
            claim.company_info_id,
        )
        .fetch_one(&mut *tx)
        .await
        {
            Ok(row) => row,
            Err(e) => {
                eprintln!("Failed to insert order: {:?}", e);
                let _ = tx.rollback().await; // rollback if order insert fails
                return Err(CustomError::DatabaseError);
            }
        };

        // ─────────────────────────────────────────────────────────────
        // STEP 4: Insert each order item + update product stock
        // ─────────────────────────────────────────────────────────────
        for item in &order.items {

            // Convert item price to BigDecimal
            let price = match item.price.parse::<BigDecimal>() {
                Ok(p) => p,
                Err(_) => {
                    let _ = tx.rollback().await;
                    return Err(CustomError::BadRequest);
                }
            };

            // Insert item into order_items table
            let insert = sqlx::query!(
                "INSERT INTO order_items (order_id, product_id, quantity, price)
                VALUES ($1, $2, $3, $4)",
                inserted_order.orders_id,
                item.product_id,
                item.quantity,
                price,
            )
            .execute(&mut *tx)
            .await;

            // Handle insert failure
            if let Err(e) = insert {
                eprintln!("Failed to insert order item: {:?}", e);
                let _ = tx.rollback().await;
                return Err(CustomError::DatabaseError);
            }

            // Reduce product stock based on quantity sold
            let deduct = sqlx::query!(
                "UPDATE products 
                 SET product_stock = product_stock - $1 
                 FROM users
                 JOIN company_info ON users.company_info_id = company_info.company_info_id
                 WHERE products.user_id = users.user_id
                 AND products.product_stock >= $2 
                 AND products.product_id = $3 
                 AND company_info.company_name = $4 
                 AND company_info.branch_name = $5",
                item.quantity,
                item.quantity,
                item.product_id,
                claim.companyname,
                claim.branchname,
            )
            .execute(&mut *tx)
            .await;

            // Handle stock deduction errors
            let deduct = match deduct {
                Ok(d) => d,
                Err(e) => {
                    eprintln!(
                        "Failed to deduct stock for product {}: {:?}",
                        item.product_id,
                        e
                    );
                    let _ = tx.rollback().await;
                    return Err(CustomError::DatabaseError);
                }
            };

            // If no rows updated → insufficient stock or invalid product
            if deduct.rows_affected() == 0 {
                let _ = tx.rollback().await;
                return Err(CustomError::BadRequest);
            }
        }

        // ─────────────────────────────────────────────────────────────
        // STEP 5: Commit transaction (final step)
        // ─────────────────────────────────────────────────────────────
        if let Err(e) = tx.commit().await {
            eprintln!("Failed to commit transaction: {:?}", e);
            return Err(CustomError::DatabaseError);
        }

        Ok(())
    }

}

//====================================================================




//=========================API ENDPOINTS==============================

// POST endpoint for creating an order
#[post("/create_order")]
pub async fn create_order(

    // JSON request body from frontend
    data: Result<web::Json<CreateOrderRequest>, actix_web::Error>,

    // Shared application state
    service: web::Data<AppState>,

    // HTTP request (used to extract JWT claims)
    req: HttpRequest,
) -> impl Responder {

    // Extract JWT claims from middleware
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    // Role-based access control for POS
    match claims.role.as_str() {

        // Allowed roles
        "admin" | "manager" | "pos_staff" => {}

        // Block all others
        _ => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    // Validate request body
    let order = match data {

        // Valid JSON
        Ok(d) => d.into_inner(),

        // Invalid JSON
        Err(_) => return HttpResponse::BadRequest().body("Missing or Invalid Input"),
    };

    // Call service layer to create order
    match service.pos.create_order(order, claims).await {

        // Success response
        Ok(()) => HttpResponse::Ok().body("Order created successfully"),

        // Error handling
        Err(e) => match e {

            // Bad input error
            CustomError::BadRequest => {
                HttpResponse::BadRequest().body(e.to_string())
            }

            // Database error
            CustomError::DatabaseError => {
                HttpResponse::InternalServerError().body(e.to_string())
            }
        }
    }
}

//====================================================================
