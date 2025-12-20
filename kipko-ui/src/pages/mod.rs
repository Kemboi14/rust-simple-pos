//! Pages module for the Kipko POS UI
//! 
//! This module contains all the main pages/components of the application.

pub mod floorplan;
pub mod orders;
pub mod menu;
pub mod staff;

pub use floorplan::FloorPlan;
pub use orders::Orders;
pub use menu::Menu;
pub use staff::Staff;
