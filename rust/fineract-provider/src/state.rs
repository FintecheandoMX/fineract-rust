//! Shared application state.
//!
//! Holds the command dispatcher, authentication service, authorization
//! service, tenant repositories (for read operations), and the connection
//! string that all handlers share via Axum `State`.

use std::sync::Arc;

use fineract_accounting::SqlxAccountRepository;
use fineract_branch::SqlxBranchRepository;
use fineract_charge::SqlxChargeRepository;
use fineract_client::SqlxClientRepository;
use fineract_code::SqlxCodeRepository;
use fineract_collateral::SqlxCollateralRepository;
use fineract_command::SynchronousCommandDispatcher;
use fineract_currency::SqlxCurrencyRepository;
use fineract_document::SqlxDocumentRepository;
use fineract_guarantor::SqlxGuarantorRepository;
use fineract_holiday::SqlxHolidayRepository;
use fineract_investor::SqlxInvestorRepository;
use fineract_loan::SqlxLoanRepository;
use fineract_loan_origination::SqlxLoanApplicationRepository;
use fineract_loan_product::SqlxLoanProductRepository;
use fineract_note::SqlxNoteRepository;
use fineract_persistence::TenantPoolManager;
use fineract_progressive_loan::SqlxProgressiveLoanRepository;
use fineract_rates::SqlxRateRepository;
use fineract_report::SqlxReportRepository;
use fineract_savings::SqlxSavingsAccountRepository;
use fineract_savings_product::SqlxSavingsProductRepository;
use fineract_security::AuthenticationService;
use fineract_security::AuthorizationService;
use fineract_staff::SqlxStaffRepository;
use fineract_tax::SqlxTaxRepository;
use fineract_working_capital_loan::SqlxWorkingCapitalLoanRepository;

/// Shared state injected into every Axum handler via `State(AppState)`.
#[derive(Clone)]
pub struct AppState {
    pub dispatcher: Arc<SynchronousCommandDispatcher>,
    pub auth_service: Arc<dyn AuthenticationService>,
    pub authz_service: Arc<dyn AuthorizationService>,
    pub pool_manager: Arc<TenantPoolManager>,
    pub connection_string: String,
    pub office_repository: Arc<SqlxBranchRepository>,
    pub loan_repository: Arc<SqlxLoanRepository>,
    pub savings_repository: Arc<SqlxSavingsAccountRepository>,
    pub account_repository: Arc<SqlxAccountRepository>,
    pub client_repository: Arc<SqlxClientRepository>,
    pub charge_repository: Arc<SqlxChargeRepository>,
    pub rate_repository: Arc<SqlxRateRepository>,
    pub tax_repository: Arc<SqlxTaxRepository>,
    pub document_repository: Arc<SqlxDocumentRepository>,
    pub investor_repository: Arc<SqlxInvestorRepository>,
    pub loan_application_repository: Arc<SqlxLoanApplicationRepository>,
    pub progressive_loan_repository: Arc<SqlxProgressiveLoanRepository>,
    pub working_capital_loan_repository: Arc<SqlxWorkingCapitalLoanRepository>,
    pub staff_repository: Arc<SqlxStaffRepository>,
    pub holiday_repository: Arc<SqlxHolidayRepository>,
    pub code_repository: Arc<SqlxCodeRepository>,
    pub currency_repository: Arc<SqlxCurrencyRepository>,
    pub loan_product_repository: Arc<SqlxLoanProductRepository>,
    pub savings_product_repository: Arc<SqlxSavingsProductRepository>,
    pub note_repository: Arc<SqlxNoteRepository>,
    pub collateral_repository: Arc<SqlxCollateralRepository>,
    pub guarantor_repository: Arc<SqlxGuarantorRepository>,
    pub report_repository: Arc<SqlxReportRepository>,
}

impl AppState {
    pub fn new(
        dispatcher: SynchronousCommandDispatcher,
        auth_service: Arc<dyn AuthenticationService>,
        authz_service: Arc<dyn AuthorizationService>,
        pool_manager: Arc<TenantPoolManager>,
        connection_string: String,
    ) -> Self {
        let cs1 = connection_string.clone();
        let cs2 = connection_string.clone();
        let cs3 = connection_string.clone();
        let cs4 = connection_string.clone();
        let cs5 = connection_string.clone();
        let cs6 = connection_string.clone();
        let cs7 = connection_string.clone();
        let cs8 = connection_string.clone();
        let cs9 = connection_string.clone();
        let cs10 = connection_string.clone();
        let cs11 = connection_string.clone();
        let cs12 = connection_string.clone();
        let cs13 = connection_string.clone();
        let cs14 = connection_string.clone();
        let cs15 = connection_string.clone();
        let cs16 = connection_string.clone();
        let cs17 = connection_string.clone();
        let cs18 = connection_string.clone();
        let cs19 = connection_string.clone();
        let cs20 = connection_string.clone();
        let cs21 = connection_string.clone();
        let cs22 = connection_string.clone();
        let cs23 = connection_string.clone();
        let pm1 = pool_manager.clone();
        let pm2 = pool_manager.clone();
        let pm3 = pool_manager.clone();
        let pm4 = pool_manager.clone();
        let pm5 = pool_manager.clone();
        let pm6 = pool_manager.clone();
        let pm7 = pool_manager.clone();
        let pm8 = pool_manager.clone();
        let pm9 = pool_manager.clone();
        let pm10 = pool_manager.clone();
        let pm11 = pool_manager.clone();
        let pm12 = pool_manager.clone();
        let pm13 = pool_manager.clone();
        let pm14 = pool_manager.clone();
        let pm15 = pool_manager.clone();
        let pm16 = pool_manager.clone();
        let pm17 = pool_manager.clone();
        let pm18 = pool_manager.clone();
        let pm19 = pool_manager.clone();
        let pm20 = pool_manager.clone();
        let pm21 = pool_manager.clone();
        let pm22 = pool_manager.clone();
        let pm23 = pool_manager.clone();
        Self {
            dispatcher: Arc::new(dispatcher),
            auth_service,
            authz_service,
            pool_manager: pool_manager.clone(),
            connection_string,
            office_repository: Arc::new(SqlxBranchRepository::new(pm1, cs1)),
            loan_repository: Arc::new(SqlxLoanRepository::new(pm2, cs2)),
            savings_repository: Arc::new(SqlxSavingsAccountRepository::new(pm3, cs3)),
            account_repository: Arc::new(SqlxAccountRepository::new(pm4, cs4)),
            client_repository: Arc::new(SqlxClientRepository::new(pm5, cs5)),
            charge_repository: Arc::new(SqlxChargeRepository::new(pm6, cs6)),
            rate_repository: Arc::new(SqlxRateRepository::new(pm7, cs7)),
            tax_repository: Arc::new(SqlxTaxRepository::new(pm8, cs8)),
            document_repository: Arc::new(SqlxDocumentRepository::new(pm9, cs9)),
            investor_repository: Arc::new(SqlxInvestorRepository::new(pm10, cs10)),
            loan_application_repository: Arc::new(SqlxLoanApplicationRepository::new(pm11, cs11)),
            progressive_loan_repository: Arc::new(SqlxProgressiveLoanRepository::new(pm12, cs12)),
            working_capital_loan_repository: Arc::new(SqlxWorkingCapitalLoanRepository::new(
                pm13, cs13,
            )),
            staff_repository: Arc::new(SqlxStaffRepository::new(pm14, cs14)),
            holiday_repository: Arc::new(SqlxHolidayRepository::new(pm15, cs15)),
            code_repository: Arc::new(SqlxCodeRepository::new(pm16, cs16)),
            currency_repository: Arc::new(SqlxCurrencyRepository::new(pm17, cs17)),
            loan_product_repository: Arc::new(SqlxLoanProductRepository::new(pm18, cs18)),
            savings_product_repository: Arc::new(SqlxSavingsProductRepository::new(pm19, cs19)),
            note_repository: Arc::new(SqlxNoteRepository::new(pm20, cs20)),
            collateral_repository: Arc::new(SqlxCollateralRepository::new(pm21, cs21)),
            guarantor_repository: Arc::new(SqlxGuarantorRepository::new(pm22, cs22)),
            report_repository: Arc::new(SqlxReportRepository::new(pm23, cs23)),
        }
    }
}
