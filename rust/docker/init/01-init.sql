# Database initialization script for Fineract Rust migration.
#
# Runs against the `fineract_desarrollo` database on first startup.
# Creates tenant isolation schema and all domain module tables.
#
# Security: tenant `desarrollo` matches the Fineract-Platform-TenantId
# header expected by the provider and integration tests.

-- Tenant database is fineract_desarrollo (created by POSTGRES_DB).
-- All tables live in the public schema; tenant isolation is enforced at
-- the application layer via the tenant_id column + AuthorizedContext checks.

-- sqlx migration tracking table (required by persistence::verify_migrations)
CREATE TABLE IF NOT EXISTS __sqlx_migrations (
    version BIGINT NOT NULL PRIMARY KEY,
    description TEXT NOT NULL,
    installed_on TIMESTAMPTZ NOT NULL DEFAULT now(),
    success BOOLEAN NOT NULL DEFAULT true,
    checksum TEXT NOT NULL,
    execution_time BIGINT NOT NULL
);

-- Offices / branches
CREATE TABLE IF NOT EXISTS offices (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    name VARCHAR(255) NOT NULL,
    external_id VARCHAR(255),
    opening_date DATE,
    timezone VARCHAR(64),
    country VARCHAR(2),
    currency VARCHAR(3) NOT NULL,
    parent_office_id BIGINT,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Loans (core loan product)
CREATE TABLE IF NOT EXISTS loans (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    office_id BIGINT NOT NULL,
    client_id BIGINT NOT NULL,
    loan_officer_id BIGINT,
    loan_type VARCHAR(32) NOT NULL,
    product_id BIGINT,
    principal VARCHAR(64) NOT NULL,
    currency VARCHAR(3) NOT NULL,
    interest_rate VARCHAR(64) NOT NULL,
    term_count INTEGER NOT NULL,
    term_period VARCHAR(16) NOT NULL,
    repayment_count INTEGER NOT NULL DEFAULT 0,
    repayment_every INTEGER NOT NULL DEFAULT 1,
    disbursement_date DATE NOT NULL,
    status VARCHAR(32) NOT NULL,
    external_id VARCHAR(255),
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Loan transactions
CREATE TABLE IF NOT EXISTS loan_transactions (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    loan_id BIGINT NOT NULL,
    transaction_type VARCHAR(32) NOT NULL,
    amount VARCHAR(64) NOT NULL,
    currency VARCHAR(3) NOT NULL,
    transaction_date DATE NOT NULL,
    reference VARCHAR(255),
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT fk_loan_transactions_loan FOREIGN KEY (loan_id) REFERENCES loans(id)
);

-- Loan schedules
CREATE TABLE IF NOT EXISTS loan_schedules (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    loan_id BIGINT NOT NULL,
    installment INTEGER NOT NULL,
    due_date DATE NOT NULL,
    principal_due VARCHAR(64) NOT NULL,
    interest_due VARCHAR(64) NOT NULL,
    total_due VARCHAR(64) NOT NULL,
    principal_paid VARCHAR(64) NOT NULL DEFAULT '0',
    interest_paid VARCHAR(64) NOT NULL DEFAULT '0',
    total_paid VARCHAR(64) NOT NULL DEFAULT '0',
    status VARCHAR(32) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT fk_loan_schedules_loan FOREIGN KEY (loan_id) REFERENCES loans(id)
);

-- Loan collateral
CREATE TABLE IF NOT EXISTS loan_collateral (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    loan_id BIGINT NOT NULL,
    collateral_type VARCHAR(64) NOT NULL,
    description TEXT,
    value VARCHAR(64) NOT NULL,
    currency VARCHAR(3) NOT NULL,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT fk_loan_collateral_loan FOREIGN KEY (loan_id) REFERENCES loans(id)
);

-- Progressive loans
CREATE TABLE IF NOT EXISTS progressive_loans (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    client_id BIGINT NOT NULL,
    total_amount VARCHAR(64) NOT NULL,
    currency VARCHAR(3) NOT NULL,
    interest_rate VARCHAR(64) NOT NULL,
    current_phase VARCHAR(32) NOT NULL,
    disbursed_amount VARCHAR(64) NOT NULL DEFAULT '0',
    remaining_amount VARCHAR(64) NOT NULL DEFAULT '0',
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Working capital loans
CREATE TABLE IF NOT EXISTS working_capital_loans (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    client_id BIGINT NOT NULL,
    credit_limit VARCHAR(64) NOT NULL,
    currency VARCHAR(3) NOT NULL,
    outstanding_balance VARCHAR(64) NOT NULL DEFAULT '0',
    available_balance VARCHAR(64) NOT NULL DEFAULT '0',
    interest_rate VARCHAR(64) NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'Active',
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Loan applications (origination)
CREATE TABLE IF NOT EXISTS loan_applications (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    office_id BIGINT NOT NULL,
    client_id BIGINT NOT NULL,
    requested_amount VARCHAR(64) NOT NULL,
    currency VARCHAR(3) NOT NULL,
    purpose TEXT NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'Submitted',
    submitted_by BIGINT NOT NULL,
    reviewed_by BIGINT,
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    reviewed_at TIMESTAMPTZ,
    rejection_reason TEXT,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Savings accounts
CREATE TABLE IF NOT EXISTS savings_accounts (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    account_no VARCHAR(64) NOT NULL UNIQUE,
    account_type VARCHAR(32) NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'PendingApproval',
    currency VARCHAR(3) NOT NULL,
    balance VARCHAR(64) NOT NULL DEFAULT '0',
    interest_rate VARCHAR(64),
    opened_date DATE NOT NULL,
    closed_date DATE,
    active BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Savings transactions
CREATE TABLE IF NOT EXISTS savings_transactions (
    id BIGINT PRIMARY KEY,
    savings_account_id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    amount VARCHAR(64) NOT NULL,
    transaction_type VARCHAR(32) NOT NULL,
    transaction_date TIMESTAMPTZ NOT NULL DEFAULT now(),
    reference VARCHAR(255),
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT fk_savings_transactions_account FOREIGN KEY (savings_account_id) REFERENCES savings_accounts(id)
);

-- COB / batch jobs
CREATE TABLE IF NOT EXISTS jobs (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    job_type VARCHAR(64) NOT NULL,
    cron_expression VARCHAR(64),
    enabled BOOLEAN NOT NULL DEFAULT true,
    max_retries INTEGER NOT NULL DEFAULT 3,
    retry_delay_seconds INTEGER NOT NULL DEFAULT 60,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS job_executions (
    id BIGINT PRIMARY KEY,
    job_id BIGINT NOT NULL,
    tenant_id BIGINT NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'pending',
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    error_message TEXT,
    retry_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT fk_job_executions_job FOREIGN KEY (job_id) REFERENCES jobs(id)
);

-- COB runs
CREATE TABLE IF NOT EXISTS cob_runs (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    job_id BIGINT NOT NULL,
    run_date DATE NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'started',
    records_processed INTEGER NOT NULL DEFAULT 0,
    records_failed INTEGER NOT NULL DEFAULT 0,
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    CONSTRAINT fk_cob_runs_job FOREIGN KEY (job_id) REFERENCES jobs(id)
);

-- Accounting: GL accounts
CREATE TABLE IF NOT EXISTS gl_accounts (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    account_code VARCHAR(64) NOT NULL UNIQUE,
    account_name VARCHAR(255) NOT NULL,
    account_type VARCHAR(32) NOT NULL,
    currency VARCHAR(3) NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'Active',
    balance VARCHAR(64) NOT NULL DEFAULT '0',
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Accounting: journal entries
CREATE TABLE IF NOT EXISTS journal_entries (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    entry_date DATE NOT NULL,
    description TEXT NOT NULL,
    reference VARCHAR(255),
    status VARCHAR(32) NOT NULL DEFAULT 'draft',
    created_by BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Accounting: journal entry lines
CREATE TABLE IF NOT EXISTS journal_entry_lines (
    id BIGINT PRIMARY KEY,
    journal_entry_id BIGINT NOT NULL,
    account_id BIGINT NOT NULL,
    debit_amount VARCHAR(64) NOT NULL DEFAULT '0',
    credit_amount VARCHAR(64) NOT NULL DEFAULT '0',
    currency VARCHAR(3) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT fk_journal_entry_lines_entry FOREIGN KEY (journal_entry_id) REFERENCES journal_entries(id),
    CONSTRAINT fk_journal_entry_lines_account FOREIGN KEY (account_id) REFERENCES gl_accounts(id)
);

-- Charges
CREATE TABLE IF NOT EXISTS charges (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    amount VARCHAR(64) NOT NULL,
    currency VARCHAR(3) NOT NULL,
    charge_type VARCHAR(32) NOT NULL,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Interest rates
CREATE TABLE IF NOT EXISTS interest_rates (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    name VARCHAR(255) NOT NULL,
    rate VARCHAR(64) NOT NULL,
    rate_type VARCHAR(32) NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Documents
CREATE TABLE IF NOT EXISTS documents (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    entity_type VARCHAR(64) NOT NULL,
    entity_id BIGINT NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    content_type VARCHAR(128) NOT NULL,
    storage_path TEXT NOT NULL,
    uploaded_by BIGINT NOT NULL,
    uploaded_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Tax
CREATE TABLE IF NOT EXISTS tax_rates (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    name VARCHAR(255) NOT NULL,
    rate VARCHAR(64) NOT NULL,
    jurisdiction VARCHAR(128) NOT NULL,
    effective_from DATE NOT NULL,
    effective_to DATE,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Investors
CREATE TABLE IF NOT EXISTS investors (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255),
    phone VARCHAR(64),
    address TEXT,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes for tenant isolation queries (every tenant-scoped query filters by tenant_id)
CREATE INDEX IF NOT EXISTS idx_offices_tenant ON offices(tenant_id);
CREATE INDEX IF NOT EXISTS idx_loans_tenant ON loans(tenant_id);
CREATE INDEX IF NOT EXISTS idx_loan_transactions_tenant ON loan_transactions(tenant_id);
CREATE INDEX IF NOT EXISTS idx_loan_schedules_tenant ON loan_schedules(tenant_id);
CREATE INDEX IF NOT EXISTS idx_loan_collateral_tenant ON loan_collateral(tenant_id);
CREATE INDEX IF NOT EXISTS idx_progressive_loans_tenant ON progressive_loans(tenant_id);
CREATE INDEX IF NOT EXISTS idx_working_capital_loans_tenant ON working_capital_loans(tenant_id);
CREATE INDEX IF NOT EXISTS idx_loan_applications_tenant ON loan_applications(tenant_id);
CREATE INDEX IF NOT EXISTS idx_savings_accounts_tenant ON savings_accounts(tenant_id);
CREATE INDEX IF NOT EXISTS idx_savings_transactions_tenant ON savings_transactions(tenant_id);
CREATE INDEX IF NOT EXISTS idx_jobs_tenant ON jobs(tenant_id);
CREATE INDEX IF NOT EXISTS idx_job_executions_tenant ON job_executions(tenant_id);
CREATE INDEX IF NOT EXISTS idx_cob_runs_tenant ON cob_runs(tenant_id);
CREATE INDEX IF NOT EXISTS idx_gl_accounts_tenant ON gl_accounts(tenant_id);
CREATE INDEX IF NOT EXISTS idx_journal_entries_tenant ON journal_entries(tenant_id);
CREATE INDEX IF NOT EXISTS idx_charges_tenant ON charges(tenant_id);
CREATE INDEX IF NOT EXISTS idx_interest_rates_tenant ON interest_rates(tenant_id);
CREATE INDEX IF NOT EXISTS idx_documents_tenant ON documents(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tax_rates_tenant ON tax_rates(tenant_id);
CREATE INDEX IF NOT EXISTS idx_investors_tenant ON investors(tenant_id);
