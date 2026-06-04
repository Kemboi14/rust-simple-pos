-- Enhanced Accounting Schema Migration
-- This migration adds advanced accounting features including:
-- - Account hierarchy with parent-child relationships
-- - Accounting periods for period closing and reporting
-- - Bank reconciliation functionality
-- - Enhanced audit trails
-- - Financial reporting support

-- Add account hierarchy support to accounts table
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS parent_id UUID REFERENCES accounts(id);
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS account_code VARCHAR(20);
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS normal_balance VARCHAR(10) CHECK (normal_balance IN ('Debit', 'Credit'));
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS opening_balance DECIMAL(10,2) DEFAULT 0;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS current_balance DECIMAL(10,2) DEFAULT 0;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS currency VARCHAR(3) DEFAULT 'KES';
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS reconciliation_account BOOLEAN DEFAULT false;

-- Create accounting periods table
CREATE TABLE IF NOT EXISTS accounting_periods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    period_name VARCHAR(50) NOT NULL,
    start_date TIMESTAMP NOT NULL,
    end_date TIMESTAMP NOT NULL,
    is_closed BOOLEAN DEFAULT false,
    closed_at TIMESTAMP,
    closed_by UUID REFERENCES staff(id),
    fiscal_year INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT unique_period UNIQUE (fiscal_year, period_name)
);

-- Create period journal entries table for tracking entries by period
CREATE TABLE IF NOT EXISTS period_journal_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    period_id UUID NOT NULL REFERENCES accounting_periods(id),
    journal_entry_id UUID NOT NULL REFERENCES journal_entries(id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create bank reconciliation table
CREATE TABLE IF NOT EXISTS bank_reconciliations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES accounts(id),
    reconciliation_date TIMESTAMP NOT NULL,
    statement_balance DECIMAL(10,2) NOT NULL,
    book_balance DECIMAL(10,2) NOT NULL,
    difference DECIMAL(10,2) NOT NULL,
    status VARCHAR(20) DEFAULT 'Pending' CHECK (status IN ('Pending', 'Completed', 'Discrepancy')),
    reconciled_by UUID REFERENCES staff(id),
    notes TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create reconciliation items table
CREATE TABLE IF NOT EXISTS reconciliation_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    reconciliation_id UUID NOT NULL REFERENCES bank_reconciliations(id),
    journal_entry_id UUID REFERENCES journal_entries(id),
    item_type VARCHAR(20) NOT NULL CHECK (item_type IN ('Deposit', 'Withdrawal', 'Adjustment', 'Fee')),
    amount DECIMAL(10,2) NOT NULL,
    description TEXT,
    is_cleared BOOLEAN DEFAULT false,
    cleared_date TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create financial reports table for storing generated reports
CREATE TABLE IF NOT EXISTS financial_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    report_type VARCHAR(50) NOT NULL CHECK (report_type IN ('BalanceSheet', 'IncomeStatement', 'CashFlow', 'TrialBalance', 'GeneralLedger')),
    period_id UUID REFERENCES accounting_periods(id),
    report_data JSONB NOT NULL,
    generated_by UUID REFERENCES staff(id),
    generated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    parameters JSONB
);

-- Create audit log table for accounting changes
CREATE TABLE IF NOT EXISTS accounting_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(50) NOT NULL,
    entity_id UUID NOT NULL,
    action_type VARCHAR(20) NOT NULL CHECK (action_type IN ('Create', 'Update', 'Delete', 'Close')),
    old_values JSONB,
    new_values JSONB,
    changed_by UUID REFERENCES staff(id),
    changed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    description TEXT
);

-- Add currency support to transactions
ALTER TABLE transactions ADD COLUMN IF NOT EXISTS currency VARCHAR(3) DEFAULT 'KES';
ALTER TABLE transactions ADD COLUMN IF NOT EXISTS exchange_rate DECIMAL(10,6) DEFAULT 1;
ALTER TABLE transactions ADD COLUMN IF NOT EXISTS period_id UUID REFERENCES accounting_periods(id);

-- Add enhanced tracking to journal entries
ALTER TABLE journal_entries ADD COLUMN IF NOT EXISTS period_id UUID REFERENCES accounting_periods(id);
ALTER TABLE journal_entries ADD COLUMN IF NOT EXISTS reference_type VARCHAR(50); -- 'Order', 'Payment', 'Adjustment', etc.
ALTER TABLE journal_entries ADD COLUMN IF NOT EXISTS is_reconciled BOOLEAN DEFAULT false;
ALTER TABLE journal_entries ADD COLUMN IF NOT EXISTS reconciled_date TIMESTAMP;

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_accounts_parent_id ON accounts(parent_id);
CREATE INDEX IF NOT EXISTS idx_accounts_account_code ON accounts(account_code);
CREATE INDEX IF NOT EXISTS idx_accounts_reconciliation ON accounts(reconciliation_account);
CREATE INDEX IF NOT EXISTS idx_accounting_periods_fiscal_year ON accounting_periods(fiscal_year);
CREATE INDEX IF NOT EXISTS idx_accounting_periods_dates ON accounting_periods(start_date, end_date);
CREATE INDEX IF NOT EXISTS idx_period_journal_entries_period ON period_journal_entries(period_id);
CREATE INDEX IF NOT EXISTS idx_period_journal_entries_entry ON period_journal_entries(journal_entry_id);
CREATE INDEX IF NOT EXISTS idx_bank_reconciliations_account ON bank_reconciliations(account_id);
CREATE INDEX IF NOT EXISTS idx_bank_reconciliations_date ON bank_reconciliations(reconciliation_date);
CREATE INDEX IF NOT EXISTS idx_reconciliation_items_reconciliation ON reconciliation_items(reconciliation_id);
CREATE INDEX IF NOT EXISTS idx_reconciliation_items_entry ON reconciliation_items(journal_entry_id);
CREATE INDEX IF NOT EXISTS idx_financial_reports_type ON financial_reports(report_type);
CREATE INDEX IF NOT EXISTS idx_financial_reports_period ON financial_reports(period_id);
CREATE INDEX IF NOT EXISTS idx_accounting_audit_entity ON accounting_audit_log(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_accounting_audit_changed_by ON accounting_audit_log(changed_by);
CREATE INDEX IF NOT EXISTS idx_journal_entries_period ON journal_entries(period_id);
CREATE INDEX IF NOT EXISTS idx_journal_entries_reference ON journal_entries(reference_type, reference_id);
CREATE INDEX IF NOT EXISTS idx_transactions_period ON transactions(period_id);

-- Create triggers for updated_at columns
CREATE TRIGGER update_accounting_periods_updated_at BEFORE UPDATE ON accounting_periods FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_bank_reconciliations_updated_at BEFORE UPDATE ON bank_reconciliations FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_financial_reports_updated_at BEFORE UPDATE ON financial_reports FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Create function to update account balances
CREATE OR REPLACE FUNCTION update_account_balance()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        -- Update account current balance based on debit/credit
        IF NEW.debit_credit = 'Debit' THEN
            UPDATE accounts 
            SET current_balance = current_balance + NEW.amount
            WHERE id = NEW.account_id;
        ELSE
            UPDATE accounts 
            SET current_balance = current_balance - NEW.amount
            WHERE id = NEW.account_id;
        END IF;
    ELSIF TG_OP = 'UPDATE' THEN
        -- Handle balance updates on journal entry modifications
        IF OLD.debit_credit != NEW.debit_credit OR OLD.amount != NEW.amount THEN
            -- Reverse old entry
            IF OLD.debit_credit = 'Debit' THEN
                UPDATE accounts 
                SET current_balance = current_balance - OLD.amount
                WHERE id = OLD.account_id;
            ELSE
                UPDATE accounts 
                SET current_balance = current_balance + OLD.amount
                WHERE id = OLD.account_id;
            END IF;
            
            -- Apply new entry
            IF NEW.debit_credit = 'Debit' THEN
                UPDATE accounts 
                SET current_balance = current_balance + NEW.amount
                WHERE id = NEW.account_id;
            ELSE
                UPDATE accounts 
                SET current_balance = current_balance - NEW.amount
                WHERE id = NEW.account_id;
            END IF;
        END IF;
    ELSIF TG_OP = 'DELETE' THEN
        -- Reverse the entry on deletion
        IF OLD.debit_credit = 'Debit' THEN
            UPDATE accounts 
            SET current_balance = current_balance - OLD.amount
            WHERE id = OLD.account_id;
        ELSE
            UPDATE accounts 
            SET current_balance = current_balance + OLD.amount
            WHERE id = OLD.account_id;
        END IF;
    END IF;
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Create trigger for automatic balance updates
DROP TRIGGER IF EXISTS journal_entry_balance_trigger ON journal_entries;
CREATE TRIGGER journal_entry_balance_trigger
AFTER INSERT OR UPDATE OR DELETE ON journal_entries
FOR EACH ROW EXECUTE FUNCTION update_account_balance();

-- Create function to audit accounting changes
CREATE OR REPLACE FUNCTION audit_accounting_changes()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO accounting_audit_log (entity_type, entity_id, action_type, new_values, changed_by, description)
        VALUES (TG_TABLE_NAME, NEW.id, 'Create', to_jsonb(NEW), 
                COALESCE(NEW.created_by, NEW.closed_by, NEW.reconciled_by), 
                'Created new ' || TG_TABLE_NAME);
    ELSIF TG_OP = 'UPDATE' THEN
        INSERT INTO accounting_audit_log (entity_type, entity_id, action_type, old_values, new_values, changed_by, description)
        VALUES (TG_TABLE_NAME, NEW.id, 'Update', to_jsonb(OLD), to_jsonb(NEW),
                COALESCE(NEW.updated_by, NEW.closed_by, NEW.reconciled_by),
                'Updated ' || TG_TABLE_NAME);
    ELSIF TG_OP = 'DELETE' THEN
        INSERT INTO accounting_audit_log (entity_type, entity_id, action_type, old_values, changed_by, description)
        VALUES (TG_TABLE_NAME, OLD.id, 'Delete', to_jsonb(OLD), NULL,
                NULL,
                'Deleted ' || TG_TABLE_NAME);
    END IF;
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Create audit triggers for key accounting tables
CREATE TRIGGER audit_accounts AFTER INSERT OR UPDATE OR DELETE ON accounts FOR EACH ROW EXECUTE FUNCTION audit_accounting_changes();
CREATE TRIGGER audit_transactions AFTER INSERT OR UPDATE OR DELETE ON transactions FOR EACH ROW EXECUTE FUNCTION audit_accounting_changes();
CREATE TRIGGER audit_journal_entries AFTER INSERT OR UPDATE OR DELETE ON journal_entries FOR EACH ROW EXECUTE FUNCTION audit_accounting_changes();
CREATE TRIGGER audit_accounting_periods AFTER INSERT OR UPDATE OR DELETE ON accounting_periods FOR EACH ROW EXECUTE FUNCTION audit_accounting_changes();
CREATE TRIGGER audit_bank_reconciliations AFTER INSERT OR UPDATE OR DELETE ON bank_reconciliations FOR EACH ROW EXECUTE FUNCTION audit_accounting_changes();

-- Insert default accounting period structure
INSERT INTO accounting_periods (period_name, start_date, end_date, fiscal_year) VALUES
('January', '2024-01-01 00:00:00', '2024-01-31 23:59:59', 2024),
('February', '2024-02-01 00:00:00', '2024-02-29 23:59:59', 2024),
('March', '2024-03-01 00:00:00', '2024-03-31 23:59:59', 2024),
('April', '2024-04-01 00:00:00', '2024-04-30 23:59:59', 2024),
('May', '2024-05-01 00:00:00', '2024-05-31 23:59:59', 2024),
('June', '2024-06-01 00:00:00', '2024-06-30 23:59:59', 2024),
('July', '2024-07-01 00:00:00', '2024-07-31 23:59:59', 2024),
('August', '2024-08-01 00:00:00', '2024-08-31 23:59:59', 2024),
('September', '2024-09-01 00:00:00', '2024-09-30 23:59:59', 2024),
('October', '2024-10-01 00:00:00', '2024-10-31 23:59:59', 2024),
('November', '2024-11-01 00:00:00', '2024-11-30 23:59:59', 2024),
('December', '2024-12-01 00:00:00', '2024-12-31 23:59:59', 2024)
ON CONFLICT (fiscal_year, period_name) DO NOTHING;

-- Update existing accounts with account codes and proper structure
UPDATE accounts SET 
    account_code = CASE 
        WHEN name = 'Cash' THEN '1000'
        WHEN name = 'Card Receivable' THEN '1100'
        WHEN name = 'Inventory' THEN '1200'
        WHEN name = 'Tax Payable' THEN '2000'
        WHEN name = 'Tips Payable' THEN '2100'
        WHEN name = 'Owner''s Equity' THEN '3000'
        WHEN name = 'Food Revenue' THEN '4000'
        WHEN name = 'Beverage Revenue' THEN '4100'
        WHEN name = 'Tax Revenue' THEN '4200'
        WHEN name = 'Food Cost' THEN '5000'
        WHEN name = 'Beverage Cost' THEN '5100'
        ELSE account_code
    END,
    normal_balance = CASE 
        WHEN account_type IN ('Asset', 'Expense') THEN 'Debit'
        ELSE 'Credit'
    END,
    reconciliation_account = CASE 
        WHEN name IN ('Cash', 'Card Receivable') THEN true
        ELSE false
    END
WHERE account_code IS NULL;