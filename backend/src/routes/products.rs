use actix_web::{
    HttpMessage, HttpRequest, HttpResponse, Responder,web, get, post, patch, delete,
};
use serde::{Serialize, Deserialize};
use sqlx::{PgPool, types::BigDecimal,QueryBuilder, Postgres, FromRow};
use serde_json::json;
use crate::middleware::auth_middleware::Claims;
use crate::state::AppState;




//=====================DATA TRANSFER OBJECTS=========================

// Used when updating a product — all fields are optional for partial updates
#[derive(Serialize, Deserialize)]
pub struct ProductUpdate {

    // Product ID to update
    pub id: i32,

    // Optional new product name
    pub name: Option<String>,

    // Optional SKU code
    pub sku: Option<String>,

    // Optional category
    pub category: Option<String>,

    // Optional price
    pub price: Option<String>,

    // Optional stock quantity
    pub stock: Option<i32>,

    // Optional branch assignment
    pub branch_name: Option<String>,
}

// Represents a product row returned from DB
#[derive(Serialize, Deserialize, FromRow)]
pub struct RowProduct {

    // Product ID
    pub id: i32,

    // Product name
    pub name: String,

    // SKU code
    pub sku: String,

    // Product category
    pub category: String,

    // Price stored as string (for precision handling)
    pub price: String,

    // Available stock quantity
    pub stock: i32,

    // Branch where product belongs
    //pub branch_name: String,
}

// Payload used when adding a new product
#[derive(Serialize, Deserialize)]
pub struct AddProduct {

    // Product name
    pub name: String,

    // SKU code
    pub sku: String,

    // Category
    pub category: String,

    // Price as string (converted to BigDecimal later)
    pub price: String,

    // Initial stock quantity
    pub stock: i32,

    // Branch name where product is assigned
    //pub branch_name: String,
}




//=====================ERROR HANDLING ENUM============================

// Custom error types for product operations
#[derive(Debug)]
pub enum CustomError {

    // User is not allowed to perform action
    Unauthorized,

    // Invalid request data
    BadRequest,

    // Database operation failed
    DatabaseError,
}

// Convert enum into readable string messages
impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        // Map errors to human-readable messages
        let msg = match self {
            CustomError::Unauthorized => "Unauthorized",
            CustomError::BadRequest => "Missing or Invalid Input",
            CustomError::DatabaseError => "Database error",
        };

        write!(f, "{}", msg)
    }
}

//====================================================================




//====================SERVICE STRUCT (OOP-LIKE DESIGN)==============

// ProductService handles all product-related business logic
# [derive(Clone)]
pub struct ProductService {

    // Database connection pool
    pool: PgPool,
}

//====================================================================




//============SERVICE IMPLEMENTATION (BUSINESS LOGIC)================

impl ProductService {

    // Constructor for ProductService
    pub fn new(pool: PgPool) -> Self {
        Self{pool}
    }

    // Apply role-based filters for product queries
    fn apply_product_filters(
        query: &mut QueryBuilder<Postgres>,
        claim: &Claims,
    ) -> Result<(), CustomError> {

        match claim.role.as_str() {

            // Admin can only see their company data
            "admin" => {
                query.push(" WHERE company_info.company_name = ");
                query.push_bind(claim.companyname.clone());
            }

            // Manager is restricted to company + branch
            "manager" => {
                query.push(" WHERE company_info.company_name = ");
                query.push_bind(claim.companyname.clone());

                query.push(" AND company_info.branch_name = ");
                query.push_bind(claim.branchname.clone());
            }

            // Other roles not allowed
            _ => {
                return Err(CustomError::Unauthorized);
            }
        }

        Ok(())
    }

    // Add a new product into database
    pub async fn add_product(
        &self,
        data: AddProduct,
        claims: Claims
    ) -> Result<serde_json::Value, CustomError> {

        // Determine allowed branch based on role
        let company_info = match claims.role.as_str() {

            // Admin must verify branch exists
            "admin" => {
                let exists = sqlx::query!(
                    "SELECT company_info_id 
                    FROM company_info 
                    WHERE company_name = $1 AND branch_name = $2
                    LIMIT 1",
                    claims.companyname,
                    claims.branchname
                )
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| {
                    eprintln!("DB error: {:?}", e);
                    CustomError::DatabaseError
                })?;

                match exists {
                    Some(r) => r.company_info_id,
                    None => return Err(CustomError::Unauthorized),
                }
            }

            // Manager uses their own branch
            "manager" => {
                if claims.company_info_id <= 0 {
                    return Err(CustomError::Unauthorized);
                }
                claims.company_info_id
            }

            // Others not allowed
            _ => {
                return Err(CustomError::Unauthorized);
            }
        };

        
        // Convert price string → BigDecimal for DB precision
        let price = match data.price.parse::<BigDecimal>() {
            Ok(p) => p,
            Err(_) => return Err(CustomError::BadRequest),
        };

        // Insert product into database
        sqlx::query!(
            "INSERT INTO products 
            (product_name, product_sku, product_category, product_price, product_stock, user_id, company_info_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7)",
            data.name,
            data.sku,
            data.category,
            price,
            data.stock,
            claims.id,
            company_info,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            eprintln!("DB error: {:?}", e);
            CustomError::DatabaseError
        })?;

        // Return success response
        let response = json!({
            "message": "Product added successfully",
        });

        Ok(response)
    }

    // Get all products based on role filtering
    pub async fn view_product(
        &self,
        claims: Claims
    ) -> Result<Vec<RowProduct>, CustomError> {

        let mut rows = sqlx::QueryBuilder::<sqlx::Postgres>::new(
        "SELECT 
            product_id AS id,
            product_name AS name,
            product_sku AS sku,
            product_category AS category,
            product_price::text AS price,
            product_stock AS stock
            FROM products
            JOIN company_info ON products.company_info_id = company_info.company_info_id"
        );

        // Apply role-based filters
        Self::apply_product_filters(&mut rows, &claims)?;

        // Execute query
        let products: Vec<RowProduct> = rows
            .build_query_as::<RowProduct>()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                eprintln!("Failed to fetch products: {:?}", e);
                CustomError::DatabaseError
        })?;

        Ok(products)
    }

    // Update product dynamically (only provided fields are updated)
    pub async fn update_product(
        &self,
        data: ProductUpdate,
        claims: Claims,
        product_id: i32
    ) -> Result<serde_json::Value, CustomError> {

        // Convert price string → BigDecimal for DB precision
        let price = match data.price {
            Some(p) => Some(p.parse::<BigDecimal>().map_err(|_| CustomError::BadRequest)?),
            None => None,
        };

        /*
        if let Some(branch_name) =data.branch_name {

            // Admin can only assign to existing branches
            if claims.role == "admin" {
                sqlx::query!(
                    "UPDATE company_info
                    SET branch_name = $1
                    WHERE company_info_id = $2",
                    branch_name,
                    claims.company_info_id,
                )
                .execute(&self.pool)
                .await
                .map_err(|e| {
                    eprintln!("DB error: {:?}", e);
                    CustomError::DatabaseError
                })?;
            }

            // Manager can only assign to their own branch
            else if claims.role == "manager" {
                if branch_name != claims.branchname {
                    return Err(CustomError::Unauthorized);
                }
            }
        }
        */

        // Start building UPDATE query
        let mut query_builder = sqlx::QueryBuilder::new("UPDATE products SET ");
        let mut has_set = false;

        // Add fields dynamically only if provided
        if let Some(name) = data.name {
            if has_set { query_builder.push(", "); }
            query_builder.push("product_name = ").push_bind(name);
            has_set = true;
        }

        if let Some(price) = price {
            if has_set { query_builder.push(", "); }
            query_builder.push("product_price = ").push_bind(price);
            has_set = true;
        }

        if let Some(category) = data.category {
            if has_set { query_builder.push(", "); }
            query_builder.push("product_category = ").push_bind(category);
            has_set = true;
        }

        if let Some(sku) = data.sku {
            if has_set { query_builder.push(", "); }
            query_builder.push("product_sku = ").push_bind(sku);
            has_set = true;
        }

        if let Some(stock) = data.stock {
            if has_set { query_builder.push(", "); }
            query_builder.push("product_stock = ").push_bind(stock);
            has_set = true;
        }

        // If nothing to update → error
        if !has_set {
            return Err(CustomError::BadRequest);
        }

        query_builder.push(" FROM company_info");

        // Apply role filters
        Self::apply_product_filters(&mut query_builder, &claims)?;

        // Join for role-based filtering
        query_builder.push(" AND company_info.company_info_id = products.company_info_id");

        // Add product ID condition
        query_builder
            .push(" AND product_id = ")
            .push_bind(product_id);

        // Execute query
        let query = query_builder.build();

        let executed_query = query.execute(&self.pool)
            .await
            .map_err(|e| {
                eprintln!("DB error: {:?}", e);
                CustomError::DatabaseError
            })?;

        // If nothing updated → invalid request
        if executed_query.rows_affected() == 0 {
            return Err(CustomError::BadRequest);
        }

        Ok(serde_json::json!({
            "message": "Product updated successfully"
        }))
    }

    // Delete product with role-based restriction
    pub async fn delete_product(
        &self,
        claims: Claims,
        product_id: i32
    ) -> Result<(), CustomError> {

        // Build DELETE query dynamically
        let mut result = sqlx::QueryBuilder::<sqlx::Postgres>::new(
            "DELETE FROM products",
        );

        // Apply role filters
        Self::apply_product_filters(&mut result, &claims)?;

        // Add product condition
        result
            .push(" AND product_id = ")
            .push_bind(product_id);

        let query = result.build();

        // Execute deletion
        let executed_query = query.execute(&self.pool)
        .await
        .map_err(|e| {
            eprintln!("DB error: {:?}", e);
            CustomError::DatabaseError
        })?;

        // If no rows affected → unauthorized or not found
        if executed_query.rows_affected() == 0 {
            return Err(CustomError::Unauthorized);
        }

        Ok(())
    }
}

//====================================================================




//=========================API ENDPOINTS==============================

// Add product endpoint
#[post("/add_product")]
pub async fn add_product(
    data: Result<web::Json<AddProduct>, actix_web::Error>,
    service: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {

    // Extract JWT claims from middleware
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    // Role-based access control
    match claims.role.as_str() {
        "admin" | "manager" => {}
        "pos_staff" => return HttpResponse::Forbidden().body("Forbidden"),
        _ => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    // Validate JSON input
    let product = match data {
        Ok(d) => d.into_inner(),
        Err(_) => return HttpResponse::BadRequest().body("Missing or Invalid Input"),
    };

    // Call service layer
    match service.products.add_product(product, claims).await {
        Ok(m) => HttpResponse::Ok().json(m),
        Err(e) => match e {
            CustomError::Unauthorized => {
                HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()}))
            }
            CustomError::BadRequest => {
                HttpResponse::BadRequest().json(serde_json::json!({"error": e.to_string()}))
            }
            CustomError::DatabaseError => {
                HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
            }
        }
    }
}

// View products endpoint
#[get("/view_product")]
pub async fn view_product(
    service: web::Data<AppState>, 
    req: HttpRequest
) -> impl Responder {

    // Extract JWT claims
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    // Role check
    match claims.role.as_str() {
        "admin" | "manager" => {}
        "pos_staff" => return HttpResponse::Forbidden().body("Forbidden"),
        _ => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    // Fetch products
    match service.products.view_product(claims).await {
        Ok(products) => HttpResponse::Ok().json(products),
        Err(e) => match e {
            CustomError::Unauthorized => HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()})),
            CustomError::BadRequest => HttpResponse::BadRequest().json(serde_json::json!({"error": e.to_string()})),
            CustomError::DatabaseError => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
        }
    }
}

// Update product endpoint
#[patch("/update_product/{id}")]
pub async fn update_product(
    data: Result<web::Json<ProductUpdate>, actix_web::Error>,
    service: web::Data<AppState>, 
    product_id : web::Path<i32>,
    req: HttpRequest
) -> impl Responder {

    // Extract claims
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    // Role validation
    match claims.role.as_str() {
        "admin" | "manager" => {}
        "pos_staff" => return HttpResponse::Forbidden().body("Forbidden"),
        _ => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    // Validate input
    let product = match data {
        Ok(p) => p.into_inner(),
        Err(_) => return HttpResponse::BadRequest().body("Missing or Invalid Input"),
    };

    let product_id = product_id.into_inner();

    // Update product
    match service.products.update_product(product, claims, product_id).await {
        Ok(m) => HttpResponse::Ok().json(m),
        Err(e) => match e {
            CustomError::Unauthorized => HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()})),
            CustomError::BadRequest => HttpResponse::BadRequest().json(serde_json::json!({"error": e.to_string()})),
            CustomError::DatabaseError => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
        }
    }
}

// Delete product endpoint
#[delete("/delete_product/{id}")]
pub async fn delete_product(
    product_id : web::Path<i32>,
    service: web::Data<AppState>,
    req: HttpRequest
) -> impl Responder {

    // Extract claims
    let claims = match req.extensions().get::<Claims>().cloned() {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    // Role validation
    match claims.role.as_str() {
        "admin" | "manager" => {}
        "pos_staff" => return HttpResponse::Forbidden().body("Forbidden"),
        _ => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    let product_id = product_id.into_inner();

    // Delete product
    match service.products.delete_product(claims, product_id).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Product deleted successfully"
        })),

        Err(e) => match e {
            CustomError::Unauthorized => HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()})),
            CustomError::BadRequest => HttpResponse::BadRequest().json(serde_json::json!({"error": e.to_string()})),
            CustomError::DatabaseError => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
        }
    }
}

//====================================================================
