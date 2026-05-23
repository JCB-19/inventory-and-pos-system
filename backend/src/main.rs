use actix_web::{web, App, HttpServer};
use sqlx::{postgres::PgPoolOptions};
use std::env;
use dotenvy::dotenv;
use actix_cors::Cors;

// Import project modules
mod routes;
mod middleware;

//=========================DASHBOARD IMPORTS==========================

// Import dashboard endpoint and service
use crate::routes::dashboard::{
    dashboard, 
    DashboardService
};

//===================================================================




//============================POS IMPORTS============================

// Import POS endpoint and service
use crate::routes::pos::{
    create_order,
    PosService
};

// Old unused import kept as reference
//use crate::routes::pos::{create_order};

//===================================================================




//=========================PRODUCT IMPORTS===========================

// Import product endpoints and service
use crate::routes::products::{
    add_product,
    view_product,
    update_product,
    delete_product,
    ProductService,
};

//===================================================================




//===========================AUTH IMPORTS============================

// Import authentication service and endpoints
use crate::routes::auth::{
    AuthService, login, register,
};

//===================================================================




//========================EMPLOYEE IMPORTS===========================

// Import employee service and endpoint
use crate::routes::employees::{
    EmployeeService,
    create_employee
};

//===================================================================




//=======================MIDDLEWARE IMPORTS==========================

// Import JWT authentication middleware
use crate::middleware::auth_middleware::{
    AuthMiddleware,
};

//===================================================================




//===========================APP STATE===============================

// Import global application state
mod state;
use state::AppState;

//===================================================================




//===========================MAIN FUNCTION===========================

// Actix async runtime entry point
#[actix_web::main]
async fn main() -> std::io::Result<()> {

    // Load environment variables from .env file
    dotenv().ok();

    // Read database connection string from environment variables
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");


    //======================DATABASE CONNECTION======================

    // Create PostgreSQL connection pool
    let create_pool = PgPoolOptions::new()

        // Maximum number of active database connections
        .max_connections(5)

        // Connect to PostgreSQL database
        .connect(&database_url)
        .await

        // Crash program if connection fails
        .expect("Failed to connect to Postgres");

    // Print success message in terminal
    println!("✅ Connected to database successfully!");

    //================================================================


    // Load JWT secret key from environment variables
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET missing");


    //==========================SERVICES==============================

    // Create authentication service
    let auth = AuthService::new(create_pool.clone(), jwt_secret);

    // Create product service
    let products = ProductService::new(create_pool.clone());

    // Create dashboard service
    let dashboards = DashboardService::new(create_pool.clone());

    // Create POS service
    let pos = PosService::new(create_pool.clone());

    // Create employee service
    let employees = EmployeeService::new(create_pool.clone());

    //================================================================


    //=========================APP STATE==============================

    // Store all services inside global AppState
    let state = AppState {
        auth,
        products,
        dashboards,
        pos,
        employees
    };

    // Wrap AppState inside Actix shared smart pointer container
    let state = web::Data::new(state);

    //================================================================


    //==========================SERVER SETUP==========================

    // Define server IP address and port
    let server_address = ("127.0.0.1", 8080);

    // Create and configure Actix HTTP server
    HttpServer::new(move ||{

        // Create new Actix application instance
        App::new()

        //===========================CORS=============================

        // Configure Cross-Origin Resource Sharing
        .wrap(
            Cors::default()

                // Allow requests from frontend running on localhost:3000
                .allowed_origin("http://localhost:3000")

                // Allow all HTTP methods (GET, POST, PUT, DELETE, etc.)
                .allow_any_method()

                // Allow all request headers
                .allow_any_header()
        )

        //===========================================================


        // Share AppState with all routes
        .app_data(state.clone())

            //=======================PRODUCT ROUTES===================

            .service(
                web::scope("/product")

                    // Protect routes with authentication middleware
                    .wrap(AuthMiddleware)

                    // Add product endpoint
                    .service(add_product)

                    // View product endpoint
                    .service(view_product)

                    // Update product endpoint
                    .service(update_product)

                    // Delete product endpoint
                    .service(delete_product)
            )

            //=======================================================


            //=========================POS ROUTES====================

            .service(
                web::scope("/pos")

                    // Protect routes with authentication middleware
                    .wrap(AuthMiddleware)

                    // Create order endpoint
                    .service(create_order)
            )

            //=======================================================


            //=====================DASHBOARD ROUTES==================

            .service(
                web::scope("/main")

                    // Protect routes with authentication middleware
                    .wrap(AuthMiddleware)

                    // Dashboard endpoint
                    .service(dashboard)
            )

            //=======================================================


            //=========================AUTH ROUTES===================

            .service(
                web::scope("/auth")

                    // Login endpoint
                    .service(login)

                    // Register endpoint
                    .service(register)
            )

            //=======================================================


            //=======================EMPLOYEE ROUTES=================

            .service(
                web::scope("/employees")

                    // Protect routes with authentication middleware
                    .wrap(AuthMiddleware)

                        // Create employee endpoint
                        .service(create_employee)

                        // Future endpoint for getting employees
                        //.service(get_employee)
            )

            //=======================================================
    })

    // Bind server to IP address and port
    .bind(server_address)

    // Panic if server fails to bind
    .unwrap_or_else(|_| panic!("Failed to bind server at {:?}", server_address))

    // Start server
    .run()

    // Keep server running asynchronously
    .await

}

//===================================================================
