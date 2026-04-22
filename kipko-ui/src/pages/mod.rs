//! Pages module for the Kipko POS UI
//!
//! This module contains all the main pages/components of the application.

pub mod floorplan;
pub mod menu;
pub mod orders;
pub mod staff;
pub mod inventory;
pub mod registry;
pub mod customers;
pub mod reservations;
pub mod kitchen;
pub mod reports;
pub mod login;
pub mod discounts;

pub use floorplan::FloorPlan;
pub use menu::Menu;
pub use orders::Orders;
pub use staff::StaffPage;
pub use inventory::InventoryPage;
pub use registry::RegistryPage;
pub use customers::CustomersPage;
pub use reservations::ReservationsPage;
pub use kitchen::KitchenPage;
pub use reports::ReportsPage;
pub use login::LoginPage;
pub use discounts::DiscountsPage;
