//! Command handlers for collateral operations.

#![allow(async_fn_in_trait)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::Collateral;
use crate::domain::CollateralType;
use crate::service::CollateralService;
use fineract_command::CommandHandler;
use fineract_command::CommandRequest;
use fineract_core::ids::CollateralId;
use fineract_core::ids::LoanId;
use fineract_core::ids::TenantId;
use fineract_security::AuthorizedContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCollateralCommand {
    pub tenant_id: TenantId,
    pub loan_id: LoanId,
    pub collateral_type: String,
    pub description: Option<String>,
    pub value: String,
    pub owned_by: Option<String>,
}

impl CommandRequest for CreateCollateralCommand {
    type Response = CollateralId;
    const COMMAND_TYPE: &'static str = "CreateCollateral";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCollateralCommand {
    pub id: CollateralId,
    pub tenant_id: TenantId,
    pub loan_id: LoanId,
    pub collateral_type: String,
    pub description: Option<String>,
    pub value: String,
    pub owned_by: Option<String>,
    pub active: bool,
}

impl CommandRequest for UpdateCollateralCommand {
    type Response = CollateralId;
    const COMMAND_TYPE: &'static str = "UpdateCollateral";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteCollateralCommand {
    pub id: CollateralId,
    pub tenant_id: TenantId,
}

impl CommandRequest for DeleteCollateralCommand {
    type Response = CollateralId;
    const COMMAND_TYPE: &'static str = "DeleteCollateral";
}

fn parse_collateral_type(s: &str) -> Result<CollateralType, String> {
    match s.to_lowercase().as_str() {
        "property" => Ok(CollateralType::Property),
        "vehicle" => Ok(CollateralType::Vehicle),
        "equipment" => Ok(CollateralType::Equipment),
        "inventory" => Ok(CollateralType::Inventory),
        "receivables" => Ok(CollateralType::Receivables),
        "cash_deposit" => Ok(CollateralType::CashDeposit),
        "guarantor" => Ok(CollateralType::Guarantor),
        "other" => Ok(CollateralType::Other),
        _ => Err(format!("invalid collateral_type: {s}")),
    }
}

pub struct CreateCollateralHandler<S: CollateralService> {
    service: Arc<S>,
}

impl<S: CollateralService> CreateCollateralHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: CollateralService> CommandHandler<CreateCollateralCommand> for CreateCollateralHandler<S> {
    const COMMAND_TYPE: &'static str = "CreateCollateral";

    async fn handle(
        &self,
        request: CreateCollateralCommand,
        context: &AuthorizedContext,
    ) -> Result<CollateralId, fineract_command::CommandError> {
        let collateral_type = parse_collateral_type(&request.collateral_type)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        let collateral = Collateral::new(
            CollateralId::new(1),
            request.tenant_id,
            request.loan_id,
            collateral_type,
            request.description,
            request.value,
            request.owned_by,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        self.service
            .create_collateral(context, collateral)
            .await
            .map(|c| c.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct UpdateCollateralHandler<S: CollateralService> {
    service: Arc<S>,
}

impl<S: CollateralService> UpdateCollateralHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: CollateralService> CommandHandler<UpdateCollateralCommand> for UpdateCollateralHandler<S> {
    const COMMAND_TYPE: &'static str = "UpdateCollateral";

    async fn handle(
        &self,
        request: UpdateCollateralCommand,
        context: &AuthorizedContext,
    ) -> Result<CollateralId, fineract_command::CommandError> {
        let collateral_type = parse_collateral_type(&request.collateral_type)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        let collateral = Collateral::new(
            request.id,
            request.tenant_id,
            request.loan_id,
            collateral_type,
            request.description,
            request.value,
            request.owned_by,
        )
        .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))?;

        let mut collateral = collateral;
        collateral.active = request.active;

        self.service
            .update_collateral(context, collateral)
            .await
            .map(|c| c.id)
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub struct DeleteCollateralHandler<S: CollateralService> {
    service: Arc<S>,
}

impl<S: CollateralService> DeleteCollateralHandler<S> {
    pub fn new(service: Arc<S>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S: CollateralService> CommandHandler<DeleteCollateralCommand> for DeleteCollateralHandler<S> {
    const COMMAND_TYPE: &'static str = "DeleteCollateral";

    async fn handle(
        &self,
        request: DeleteCollateralCommand,
        context: &AuthorizedContext,
    ) -> Result<CollateralId, fineract_command::CommandError> {
        self.service
            .delete_collateral(context, request.id)
            .await
            .map_err(|e| fineract_command::CommandError::HandlerError(e.to_string()))
    }
}

pub fn register_handlers<S: CollateralService + 'static>(
    dispatcher: &mut fineract_command::SynchronousCommandDispatcher,
    service: Arc<S>,
) {
    dispatcher.register_handler(CreateCollateralHandler::new(service.clone()));
    dispatcher.register_handler(UpdateCollateralHandler::new(service.clone()));
    dispatcher.register_handler(DeleteCollateralHandler::new(service));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_types_are_distinct() {
        assert_eq!(CreateCollateralCommand::COMMAND_TYPE, "CreateCollateral");
        assert_eq!(UpdateCollateralCommand::COMMAND_TYPE, "UpdateCollateral");
        assert_eq!(DeleteCollateralCommand::COMMAND_TYPE, "DeleteCollateral");
    }
}
