//! Tax calculation engine for Kipko POS
//! 
//! This module provides a robust tax calculation system that handles
//! item-level taxation, tax exemptions, and complex tax scenarios.

use crate::money::Money;
use crate::models::{OrderItem, MenuItem};
use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use thiserror::Error;

/// Tax calculation errors
#[derive(Debug, Error, PartialEq)]
pub enum TaxError {
    #[error("Invalid tax rate: {0}")]
    InvalidTaxRate(Decimal),
    #[error("Tax exemption not found: {0}")]
    ExemptionNotFound(String),
    #[error("Tax jurisdiction not found: {0}")]
    JurisdictionNotFound(String),
    #[error("Money calculation error: {0}")]
    MoneyError(#[from] crate::money::MoneyError),
}

/// Result type for tax operations
pub type TaxResult<T> = Result<T, TaxError>;

/// Tax jurisdiction (e.g., state, city, special district)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct TaxJurisdiction {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub tax_rate: Decimal, // Percentage rate (e.g., 8.5 for 8.5%)
    pub is_active: bool,
    pub effective_date: DateTime<Utc>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TaxJurisdiction {
    pub fn new(name: String, code: String, tax_rate: Decimal) -> TaxResult<Self> {
        if tax_rate < Decimal::ZERO || tax_rate > Decimal::from_str_exact("100").unwrap() {
            return Err(TaxError::InvalidTaxRate(tax_rate));
        }
        
        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            name,
            code,
            tax_rate,
            is_active: true,
            effective_date: now,
            expiry_date: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn is_effective_at(&self, date: DateTime<Utc>) -> bool {
        date >= self.effective_date && 
        self.expiry_date.map_or(true, |expiry| date < expiry)
    }
}

/// Tax exemption type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(type_name = "text"))]
pub enum TaxExemptionType {
    NonProfit,
    Government,
    Resale,
    Agricultural,
    Manufacturing,
    Other,
}

/// Tax exemption
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct TaxExemption {
    pub id: Uuid,
    pub name: String,
    pub exemption_type: TaxExemptionType,
    pub certificate_number: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TaxExemption {
    pub fn new(name: String, exemption_type: TaxExemptionType, certificate_number: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            exemption_type,
            certificate_number,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Tax calculation result for a single item
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemTaxCalculation {
    pub item_id: Uuid,
    pub item_name: String,
    pub pre_tax_amount: Money,
    pub tax_amount: Money,
    pub total_amount: Money,
    pub tax_rate: Decimal,
    pub is_exempt: bool,
    pub exemption_reason: Option<String>,
}

/// Tax calculation result for an entire order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderTaxCalculation {
    pub items: Vec<ItemTaxCalculation>,
    pub subtotal: Money,
    pub total_tax: Money,
    pub grand_total: Money,
    pub tax_breakdown: Vec<TaxBreakdown>,
}

/// Tax breakdown by jurisdiction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaxBreakdown {
    pub jurisdiction_name: String,
    pub tax_rate: Decimal,
    pub taxable_amount: Money,
    pub tax_amount: Money,
}

/// Tax calculation engine
#[derive(Debug, Clone)]
pub struct TaxEngine {
    jurisdictions: Vec<TaxJurisdiction>,
    exemptions: Vec<TaxExemption>,
}

impl TaxEngine {
    pub fn new() -> Self {
        Self {
            jurisdictions: Vec::new(),
            exemptions: Vec::new(),
        }
    }

    /// Add a tax jurisdiction
    pub fn add_jurisdiction(&mut self, jurisdiction: TaxJurisdiction) {
        self.jurisdictions.push(jurisdiction);
    }

    /// Add a tax exemption
    pub fn add_exemption(&mut self, exemption: TaxExemption) {
        self.exemptions.push(exemption);
    }

    /// Initialize with default US tax jurisdictions
    pub fn initialize_default() -> Self {
        let mut engine = Self::new();
        
        // Add common tax jurisdictions
        engine.add_jurisdiction(
            TaxJurisdiction::new("State Tax".to_string(), "STATE".to_string(), Decimal::from_str_exact("6.5").unwrap()).unwrap()
        );
        engine.add_jurisdiction(
            TaxJurisdiction::new("City Tax".to_string(), "CITY".to_string(), Decimal::from_str_exact("2.0").unwrap()).unwrap()
        );
        engine.add_jurisdiction(
            TaxJurisdiction::new("Special District".to_string(), "SPECIAL".to_string(), Decimal::from_str_exact("0.5").unwrap()).unwrap()
        );
        
        // Add common exemptions
        engine.add_exemption(
            TaxExemption::new("Resale Certificate".to_string(), TaxExemptionType::Resale, Some("RES-12345".to_string()))
        );
        engine.add_exemption(
            TaxExemption::new("Non-Profit Exemption".to_string(), TaxExemptionType::NonProfit, Some("NP-67890".to_string()))
        );
        
        engine
    }

    /// Calculate tax for an order
    pub fn calculate_order_tax(
        &self,
        order_items: &[OrderItem],
        menu_items: &[MenuItem],
        exemption_id: Option<Uuid>,
    ) -> TaxResult<OrderTaxCalculation> {
        let mut item_calculations = Vec::new();
        let mut subtotal = Money::zero(crate::money::currencies::usd());
        let mut total_tax = Money::zero(crate::money::currencies::usd());
        let mut tax_breakdown = Vec::new();

        // Get exemption if applicable
        let exemption = exemption_id.and_then(|id| self.exemptions.iter().find(|e| e.id == id));

        // Calculate tax for each item
        for order_item in order_items {
            if order_item.status == crate::models::OrderItemStatus::Voided {
                continue;
            }

            // Find the corresponding menu item
            let menu_item = menu_items.iter()
                .find(|mi| mi.id == order_item.menu_item_id)
                .ok_or_else(|| TaxError::ExemptionNotFound("Menu item not found".to_string()))?;

            let pre_tax_amount = order_item.subtotal();
            let is_exempt = exemption.is_some() && self.is_item_exempt(menu_item, exemption.unwrap());
            
            let tax_amount = if is_exempt {
                Money::zero(pre_tax_amount.currency().to_string())
            } else {
                self.calculate_item_tax(&pre_tax_amount, menu_item.tax_rate)?
            };

            let total_amount = pre_tax_amount.add(&tax_amount)?;

            item_calculations.push(ItemTaxCalculation {
                item_id: order_item.id,
                item_name: menu_item.name.clone(),
                pre_tax_amount: pre_tax_amount.clone(),
                tax_amount: tax_amount.clone(),
                total_amount,
                tax_rate: menu_item.tax_rate,
                is_exempt,
                exemption_reason: exemption.map(|e| e.name.clone()),
            });

            subtotal = subtotal.add(&pre_tax_amount)?;
            total_tax = total_tax.add(&tax_amount)?;
        }

        // Create tax breakdown by jurisdiction
        for jurisdiction in &self.jurisdictions {
            if jurisdiction.is_active && jurisdiction.is_effective_at(Utc::now()) {
                let taxable_amount = if exemption.is_some() {
                    // For exempt orders, calculate based on non-exempt items
                    item_calculations.iter()
                        .filter(|calc| !calc.is_exempt)
                        .map(|calc| calc.pre_tax_amount.clone())
                        .fold(Money::zero(crate::money::currencies::usd()), |acc, amount| acc.add(&amount).unwrap())
                } else {
                    subtotal.clone()
                };

                let tax_amount = taxable_amount.percentage(jurisdiction.tax_rate)?;
                
                if !tax_amount.is_zero() {
                    tax_breakdown.push(TaxBreakdown {
                        jurisdiction_name: jurisdiction.name.clone(),
                        tax_rate: jurisdiction.tax_rate,
                        taxable_amount,
                        tax_amount,
                    });
                }
            }
        }

        let grand_total = subtotal.add(&total_tax)?;

        Ok(OrderTaxCalculation {
            items: item_calculations,
            subtotal,
            total_tax,
            grand_total,
            tax_breakdown,
        })
    }

    /// Calculate tax for a single item
    pub fn calculate_item_tax(&self, pre_tax_amount: &Money, tax_rate: Decimal) -> TaxResult<Money> {
        if tax_rate < Decimal::ZERO || tax_rate > Decimal::from_str_exact("100").unwrap() {
            return Err(TaxError::InvalidTaxRate(tax_rate));
        }

        pre_tax_amount.percentage(tax_rate).map_err(|_| TaxError::InvalidTaxRate(tax_rate))
    }

    /// Check if an item is exempt based on exemption type
    fn is_item_exempt(&self, menu_item: &MenuItem, exemption: &TaxExemption) -> bool {
        match exemption.exemption_type {
            TaxExemptionType::Resale => true, // All items are exempt for resale
            TaxExemptionType::NonProfit => true, // All items are exempt for non-profits
            TaxExemptionType::Government => true, // All items are exempt for government
            TaxExemptionType::Agricultural => {
                // Only certain items might be exempt for agricultural
                menu_item.name.to_lowercase().contains("seed") ||
                menu_item.name.to_lowercase().contains("feed")
            }
            TaxExemptionType::Manufacturing => {
                // Only certain items might be exempt for manufacturing
                menu_item.name.to_lowercase().contains("raw") ||
                menu_item.name.to_lowercase().contains("ingredient")
            }
            TaxExemptionType::Other => false, // Requires manual review
        }
    }

    /// Get effective tax rate at a specific date
    pub fn get_effective_tax_rate(&self, date: DateTime<Utc>) -> Decimal {
        self.jurisdictions
            .iter()
            .filter(|j| j.is_active && j.is_effective_at(date))
            .map(|j| j.tax_rate)
            .fold(Decimal::ZERO, |acc, rate| acc + rate)
    }

    /// Get all active jurisdictions
    pub fn get_active_jurisdictions(&self) -> Vec<&TaxJurisdiction> {
        self.jurisdictions
            .iter()
            .filter(|j| j.is_active && j.is_effective_at(Utc::now()))
            .collect()
    }

    /// Get all active exemptions
    pub fn get_active_exemptions(&self) -> Vec<&TaxExemption> {
        self.exemptions
            .iter()
            .filter(|e| e.is_active)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{OrderItem, MenuItem};
    use rust_decimal_macros::dec;

    fn create_test_menu_item(name: &str, price: f64, tax_rate: f64) -> MenuItem {
        let category_id = Uuid::new_v4();
        MenuItem::new(
            category_id,
            name.to_string(),
            Money::new(dec!(price), crate::money::currencies::usd()).unwrap(),
            dec!(tax_rate),
        )
    }

    fn create_test_order_item(menu_item_id: Uuid, quantity: i32, price: f64) -> OrderItem {
        let order_id = Uuid::new_v4();
        OrderItem::new(
            order_id,
            menu_item_id,
            quantity,
            Money::new(dec!(price), crate::money::currencies::usd()).unwrap(),
        )
    }

    #[test]
    fn test_tax_engine_initialization() {
        let engine = TaxEngine::initialize_default();
        
        // Should have default jurisdictions
        assert!(!engine.get_active_jurisdictions().is_empty());
        
        // Should have default exemptions
        assert!(!engine.get_active_exemptions().is_empty());
    }

    #[test]
    fn test_simple_tax_calculation() {
        let engine = TaxEngine::initialize_default();
        
        let menu_item = create_test_menu_item("Burger", 10.00, 8.5);
        let order_item = create_test_order_item(menu_item.id, 2, 10.00);
        
        let result = engine.calculate_order_tax(&[order_item], &[menu_item], None).unwrap();
        
        assert_eq!(result.subtotal.amount(), dec!(20.00));
        assert_eq!(result.total_tax.amount(), dec!(1.70)); // 8.5% of $20.00
        assert_eq!(result.grand_total.amount(), dec!(21.70));
    }

    #[test]
    fn test_exempt_tax_calculation() {
        let engine = TaxEngine::initialize_default();
        
        let menu_item = create_test_menu_item("Burger", 10.00, 8.5);
        let order_item = create_test_order_item(menu_item.id, 2, 10.00);
        
        let exemption = TaxExemption::new("Test Exemption".to_string(), TaxExemptionType::NonProfit, None);
        let exemption_id = exemption.id;
        engine.add_exemption(exemption);
        
        let result = engine.calculate_order_tax(&[order_item], &[menu_item], Some(exemption_id)).unwrap();
        
        assert_eq!(result.subtotal.amount(), dec!(20.00));
        assert_eq!(result.total_tax.amount(), dec!(0.00)); // Exempt
        assert_eq!(result.grand_total.amount(), dec!(20.00));
        
        // Check that items are marked as exempt
        assert!(result.items[0].is_exempt);
        assert!(result.items[0].exemption_reason.is_some());
    }

    #[test]
    fn test_voided_items_excluded() {
        let engine = TaxEngine::initialize_default();
        
        let menu_item = create_test_menu_item("Burger", 10.00, 8.5);
        let mut order_item = create_test_order_item(menu_item.id, 2, 10.00);
        order_item.status = crate::models::OrderItemStatus::Voided;
        
        let result = engine.calculate_order_tax(&[order_item], &[menu_item], None).unwrap();
        
        assert_eq!(result.subtotal.amount(), dec!(0.00));
        assert_eq!(result.total_tax.amount(), dec!(0.00));
        assert_eq!(result.grand_total.amount(), dec!(0.00));
    }

    #[test]
    fn test_tax_breakdown() {
        let engine = TaxEngine::initialize_default();
        
        let menu_item = create_test_menu_item("Burger", 10.00, 8.5);
        let order_item = create_test_order_item(menu_item.id, 1, 10.00);
        
        let result = engine.calculate_order_tax(&[order_item], &[menu_item], None).unwrap();
        
        // Should have tax breakdown by jurisdiction
        assert!(!result.tax_breakdown.is_empty());
        
        // Check that total tax matches breakdown sum
        let breakdown_total: Decimal = result.tax_breakdown
            .iter()
            .map(|td| td.tax_amount.amount())
            .fold(Decimal::ZERO, |acc, amount| acc + amount);
        
        assert_eq!(breakdown_total, result.total_tax.amount());
    }

    #[test]
    fn test_invalid_tax_rate() {
        let engine = TaxEngine::new();
        let amount = Money::new(dec!(100.00), crate::money::currencies::usd()).unwrap();
        
        // Test invalid tax rates
        assert!(engine.calculate_item_tax(&amount, dec!(-5.0)).is_err());
        assert!(engine.calculate_item_tax(&amount, dec!(150.0)).is_err());
    }

    #[test]
    fn test_jurisdiction_effectiveness() {
        let now = Utc::now();
        let future = now + chrono::Duration::days(30);
        
        let mut jurisdiction = TaxJurisdiction::new("Future Tax".to_string(), "FUTURE".to_string(), dec!(10.0)).unwrap();
        jurisdiction.effective_date = future;
        
        let engine = TaxEngine::new();
        
        // Should not be effective now
        assert!(!jurisdiction.is_effective_at(now));
        
        // Should be effective in the future
        assert!(jurisdiction.is_effective_at(future));
    }
}
