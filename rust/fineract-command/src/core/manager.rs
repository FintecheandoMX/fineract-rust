//! Command handler manager — registry of command handlers.
//!
//! Handlers are stored as type-erased `Box<dyn ErasedCommandHandler>` and
//! dispatched by command type name.

use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::core::handler::CommandHandler;
use crate::core::request::CommandRequest;
use crate::errors::CommandError;
use async_trait::async_trait;
use fineract_security::AuthorizedContext;

/// Type-erased command handler — object-safe so it can be stored as a trait
/// object.
#[async_trait]
pub trait ErasedCommandHandler: Send + Sync + Debug {
    async fn handle(
        &self,
        payload: Box<dyn Any + Send + Sync>,
        context: &AuthorizedContext,
    ) -> Result<Box<dyn Any + Send + Sync>, CommandError>;
}

/// Wraps a concrete [`CommandHandler`] so it can be stored as
/// [`ErasedCommandHandler`]. The `R` type parameter constrains the impl.
pub struct HandlerWrapper<R: CommandRequest, H: CommandHandler<R>> {
    pub handler: H,
    pub _marker: PhantomData<R>,
}

impl<R: CommandRequest, H: CommandHandler<R>> Debug for HandlerWrapper<R, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HandlerWrapper")
            .field("command_type", &R::COMMAND_TYPE)
            .finish()
    }
}

#[async_trait]
impl<R: CommandRequest + 'static, H: CommandHandler<R> + 'static> ErasedCommandHandler
    for HandlerWrapper<R, H>
{
    async fn handle(
        &self,
        payload: Box<dyn Any + Send + Sync>,
        context: &AuthorizedContext,
    ) -> Result<Box<dyn Any + Send + Sync>, CommandError> {
        let req = *payload
            .downcast::<R>()
            .expect("command payload type mismatch");
        let response = self.handler.handle(req, context).await?;
        Ok(Box::new(response) as Box<dyn Any + Send + Sync>)
    }
}

/// Manages registration and lookup of command handlers by command type.
pub trait CommandHandlerManager: Send + Sync {
    /// Register a handler for a command type.
    fn register_handler<R: CommandRequest + 'static, H: CommandHandler<R> + 'static>(
        &mut self,
        handler: H,
    );

    /// Look up a handler by command type name.
    fn find_handler(
        &self,
        command_type: &str,
    ) -> Option<Arc<dyn ErasedCommandHandler + Send + Sync>>;

    /// Returns true if a handler is registered for the command type.
    fn has_handler(&self, command_type: &str) -> bool;
}

/// Default in-memory handler manager.
pub struct DefaultCommandHandlerManager {
    handlers: HashMap<String, Arc<dyn ErasedCommandHandler + Send + Sync>>,
}

impl DefaultCommandHandlerManager {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }
}

impl Default for DefaultCommandHandlerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandHandlerManager for DefaultCommandHandlerManager {
    fn register_handler<R: CommandRequest + 'static, H: CommandHandler<R> + 'static>(
        &mut self,
        handler: H,
    ) {
        let wrapper = HandlerWrapper {
            handler,
            _marker: PhantomData,
        };
        self.handlers
            .insert(R::COMMAND_TYPE.to_string(), Arc::new(wrapper));
    }

    fn find_handler(
        &self,
        command_type: &str,
    ) -> Option<Arc<dyn ErasedCommandHandler + Send + Sync>> {
        self.handlers.get(command_type).cloned()
    }

    fn has_handler(&self, command_type: &str) -> bool {
        self.handlers.contains_key(command_type)
    }
}
