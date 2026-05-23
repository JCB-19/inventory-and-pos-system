use crate::{AuthService, ProductService, DashboardService, PosService, EmployeeService};



pub struct AppState {
    pub auth: AuthService,
    pub products: ProductService,
    pub dashboards: DashboardService,
    pub pos: PosService,
    pub employees: EmployeeService,
}