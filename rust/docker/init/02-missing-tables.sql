-- Staff/Users table
CREATE TABLE IF NOT EXISTS staff (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    office_id BIGINT,
    firstname VARCHAR(255) NOT NULL,
    lastname VARCHAR(255) NOT NULL,
    username VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255),
    mobile_no VARCHAR(64),
    is_loan_officer BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    password_hash TEXT,
    external_id VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Holidays
CREATE TABLE IF NOT EXISTS holidays (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    name VARCHAR(255) NOT NULL,
    holiday_date DATE NOT NULL,
    is_recurring BOOLEAN NOT NULL DEFAULT false,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Code/CodeValue (lookup tables)
CREATE TABLE IF NOT EXISTS code (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS code_value (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    code_id BIGINT NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT fk_code_value_code FOREIGN KEY (code_id) REFERENCES code(id)
);

-- Currency
CREATE TABLE IF NOT EXISTS currency (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    code VARCHAR(3) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    symbol VARCHAR(10),
    decimal_places INTEGER NOT NULL DEFAULT 2,
    in_multiples_of INTEGER NOT NULL DEFAULT 1,
    display_symbol BOOLEAN NOT NULL DEFAULT true,
    display_label BOOLEAN NOT NULL DEFAULT true,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Loan products
CREATE TABLE IF NOT EXISTS loan_products (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    name VARCHAR(255) NOT NULL,
    short_name VARCHAR(100),
    description TEXT,
    currency_code VARCHAR(3) NOT NULL,
    principal_min VARCHAR(64),
    principal_max VARCHAR(64),
    principal_step VARCHAR(64),
    interest_rate_min VARCHAR(64),
    interest_rate_max VARCHAR(64),
    interest_rate_step VARCHAR(64),
    term_min INTEGER,
    term_max INTEGER,
    term_step INTEGER,
    repayment_every INTEGER NOT NULL DEFAULT 1,
    repayment_type VARCHAR(32) NOT NULL DEFAULT 'fixed',
    interest_calculation_period VARCHAR(32) NOT NULL DEFAULT 'daily',
    interest_rate_type VARCHAR(32) NOT NULL DEFAULT 'fixed',
    amortization_type VARCHAR(32) NOT NULL DEFAULT 'equal_installments',
    interest_recalculation_enabled BOOLEAN NOT NULL DEFAULT false,
    grace_period INTEGER NOT NULL DEFAULT 0,
    multi_disbursement_enabled BOOLEAN NOT NULL DEFAULT false,
    outstanding_balance_min VARCHAR(64),
    outstanding_balance_max VARCHAR(64),
    charge_calculation_on VARCHAR(32) NOT NULL DEFAULT 'original_principal',
    status VARCHAR(32) NOT NULL DEFAULT 'active',
    include_in_borrower_cycle BOOLEAN NOT NULL DEFAULT false,
    number_of_repayment_periods_before_prepayment_allowed INTEGER,
    interest_rate_frequency_type VARCHAR(32),
    interest_rate_frequency INTEGER,
    annual_nominal_interest_rate VARCHAR(64),
    penalty_for_early_repayment VARCHAR(64),
    pre_closure_penalty_applicable_on VARCHAR(32),
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Savings products
CREATE TABLE IF NOT EXISTS savings_products (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    name VARCHAR(255) NOT NULL,
    short_name VARCHAR(100),
    description TEXT,
    currency_code VARCHAR(3) NOT NULL,
    account_type VARCHAR(32) NOT NULL DEFAULT 'individual',
    status VARCHAR(32) NOT NULL DEFAULT 'active',
    deposit_type VARCHAR(32) NOT NULL DEFAULT 'compulsory',
    minimum_balance VARCHAR(64),
    lockin_period_frequency_type VARCHAR(32),
    lockin_period_frequency INTEGER,
    minimum_deposit VARCHAR(64),
    maximum_deposit VARCHAR(64),
    deposit_frequency_type VARCHAR(32),
    deposit_frequency INTEGER,
    withdrawal_frequency_type VARCHAR(32),
    withdrawal_frequency INTEGER,
    allow_overdraft BOOLEAN NOT NULL DEFAULT false,
    overdraft_limit VARCHAR(64),
    interest_rate VARCHAR(64),
    interest_calculation_type VARCHAR(32) NOT NULL DEFAULT 'daily',
    interest_posting_period_type VARCHAR(32) NOT NULL DEFAULT 'monthly',
    interest_calculation_days_in_year_type VARCHAR(32) NOT NULL DEFAULT 'actual',
    interest_compounding_period_type VARCHAR(32) NOT NULL DEFAULT 'monthly',
    account_closure_charge VARCHAR(64),
    account_closure_charge_type VARCHAR(32),
    account_closure_charge_amount VARCHAR(64),
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Guarantors
CREATE TABLE IF NOT EXISTS guarantors (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    loan_id BIGINT NOT NULL,
    client_id BIGINT NOT NULL,
    amount VARCHAR(64) NOT NULL,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT fk_guarantors_loan FOREIGN KEY (loan_id) REFERENCES loans(id)
);

-- Standing instructions
CREATE TABLE IF NOT EXISTS standing_instructions (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    client_id BIGINT NOT NULL,
    from_account_type VARCHAR(32) NOT NULL,
    from_account_id BIGINT NOT NULL,
    to_account_type VARCHAR(32) NOT NULL,
    to_account_id BIGINT NOT NULL,
    amount VARCHAR(64) NOT NULL,
    currency_code VARCHAR(3) NOT NULL,
    frequency_type VARCHAR(32) NOT NULL,
    frequency INTEGER NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE,
    status VARCHAR(32) NOT NULL DEFAULT 'active',
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Reports/Datatables (report definitions)
CREATE TABLE IF NOT EXISTS reports (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    report_type VARCHAR(64) NOT NULL,
    sql_query TEXT,
    parameters JSONB,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- System configuration
CREATE TABLE IF NOT EXISTS system_configuration (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    config_key VARCHAR(255) NOT NULL UNIQUE,
    config_value TEXT,
    description TEXT,
    data_type VARCHAR(32) NOT NULL DEFAULT 'string',
    category VARCHAR(64),
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Email/SMS templates
CREATE TABLE IF NOT EXISTS communication_templates (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    name VARCHAR(255) NOT NULL,
    subject VARCHAR(255),
    body TEXT NOT NULL,
    template_type VARCHAR(32) NOT NULL, -- email, sms, push
    locale VARCHAR(10) NOT NULL DEFAULT 'en',
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Client notes
CREATE TABLE IF NOT EXISTS client_notes (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    client_id BIGINT NOT NULL,
    note TEXT NOT NULL,
    created_by BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Client identifiers (external IDs)
CREATE TABLE IF NOT EXISTS client_identifiers (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    client_id BIGINT NOT NULL,
    identifier_type VARCHAR(64) NOT NULL,
    identifier_value VARCHAR(255) NOT NULL,
    description TEXT,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes for new tables
CREATE INDEX IF NOT EXISTS idx_staff_tenant ON staff(tenant_id);
CREATE INDEX IF NOT EXISTS idx_staff_office ON staff(office_id);
CREATE INDEX IF NOT EXISTS idx_staff_username ON staff(username);
CREATE INDEX IF NOT EXISTS idx_holidays_tenant ON holidays(tenant_id);
CREATE INDEX IF NOT EXISTS idx_holidays_date ON holidays(holiday_date);
CREATE INDEX IF NOT EXISTS idx_code_tenant ON code(tenant_id);
CREATE INDEX IF NOT EXISTS idx_code_value_tenant ON code_value(tenant_id);
CREATE INDEX IF NOT EXISTS idx_code_value_code ON code_value(code_id);
CREATE INDEX IF NOT EXISTS idx_currency_tenant ON currency(tenant_id);
CREATE INDEX IF NOT EXISTS idx_currency_code ON currency(code);
CREATE INDEX IF NOT EXISTS idx_loan_products_tenant ON loan_products(tenant_id);
CREATE INDEX IF NOT EXISTS idx_savings_products_tenant ON savings_products(tenant_id);
CREATE INDEX IF NOT EXISTS idx_guarantors_tenant ON guarantors(tenant_id);
CREATE INDEX IF NOT EXISTS idx_guarantors_loan ON guarantors(loan_id);
CREATE INDEX IF NOT EXISTS idx_standing_instructions_tenant ON standing_instructions(tenant_id);
CREATE INDEX IF NOT EXISTS idx_standing_instructions_client ON standing_instructions(client_id);
CREATE INDEX IF NOT EXISTS idx_reports_tenant ON reports(tenant_id);
CREATE INDEX IF NOT EXISTS idx_system_configuration_tenant ON system_configuration(tenant_id);
CREATE INDEX IF NOT EXISTS idx_communication_templates_tenant ON communication_templates(tenant_id);
CREATE INDEX IF NOT EXISTS idx_client_notes_tenant ON client_notes(tenant_id);
CREATE INDEX IF NOT EXISTS idx_client_notes_client ON client_notes(client_id);
CREATE INDEX IF NOT EXISTS idx_client_identifiers_tenant ON client_identifiers(tenant_id);
CREATE INDEX IF NOT EXISTS idx_client_identifiers_client ON client_identifiers(client_id);