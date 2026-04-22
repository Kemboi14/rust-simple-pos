-- Add inventory, stock management, product images, and registry functionality

-- Add new columns to menu_items table
ALTER TABLE menu_items 
ADD COLUMN image_url TEXT,
ADD COLUMN stock_quantity INTEGER DEFAULT 0 CHECK (stock_quantity >= 0),
ADD COLUMN low_stock_threshold INTEGER DEFAULT 10 CHECK (low_stock_threshold >= 0);

-- Create enum for inventory transaction types
CREATE TYPE inventory_transaction_type AS ENUM ('StockIn', 'StockOut', 'Adjustment', 'Transfer');

-- Create inventory transactions table
CREATE TABLE inventory_transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    menu_item_id UUID NOT NULL REFERENCES menu_items(id),
    transaction_type inventory_transaction_type NOT NULL,
    quantity INTEGER NOT NULL,
    notes TEXT,
    created_by UUID NOT NULL REFERENCES staff(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create registry entries table for tracking system events
CREATE TABLE registry_entries (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    entity_type VARCHAR(255) NOT NULL,
    entity_id UUID NOT NULL,
    action VARCHAR(255) NOT NULL,
    details TEXT,
    created_by UUID NOT NULL REFERENCES staff(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for new tables
CREATE INDEX idx_inventory_transactions_menu_item_id ON inventory_transactions(menu_item_id);
CREATE INDEX idx_inventory_transactions_created_by ON inventory_transactions(created_by);
CREATE INDEX idx_inventory_transactions_created_at ON inventory_transactions(created_at);
CREATE INDEX idx_registry_entries_entity_type ON registry_entries(entity_type);
CREATE INDEX idx_registry_entries_entity_id ON registry_entries(entity_id);
CREATE INDEX idx_registry_entries_created_by ON registry_entries(created_by);
CREATE INDEX idx_registry_entries_created_at ON registry_entries(created_at);

-- Create trigger for inventory_transactions updated_at (if needed in future)
CREATE TRIGGER update_inventory_transactions_updated_at BEFORE UPDATE ON inventory_transactions FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
