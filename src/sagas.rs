//! Lightweight Saga / Workflow engine.
//!
//! A *saga* is an ordered sequence of steps where each step has an optional
//! *compensation* action that is run if a later step fails (rollback).
//!
//! ## Usage
//! ```rust,ignore
//! use medusa_rust::sagas::{Saga, StepResult};
//!
//! let saga = Saga::builder("create_order")
//!     .step(
//!         "reserve_inventory",
//!         |ctx| async move { /* ... */ Ok(StepResult::done()) },
//!         Some(|ctx| async move { /* release inventory */ Ok(()) }),
//!     )
//!     .step(
//!         "charge_payment",
//!         |ctx| async move { /* ... */ Ok(StepResult::done()) },
//!         Some(|ctx| async move { /* refund */ Ok(()) }),
//!     )
//!     .build();
//!
//! let result = saga.run(ctx).await;
//! ```

use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};
use anyhow::Result;
use serde_json::Value;

// ─── Step result ─────────────────────────────────────────────────────────────

/// Value returned by a saga step.
#[derive(Debug, Clone, Default)]
pub struct StepResult {
    /// Arbitrary output data stored in the saga context under the step name.
    pub output: Option<Value>,
}

impl StepResult {
    /// Create a result with no output data.
    pub fn done() -> Self {
        Self { output: None }
    }

    /// Create a result carrying structured output.
    pub fn with_output(value: Value) -> Self {
        Self { output: Some(value) }
    }
}

// ─── Saga context ─────────────────────────────────────────────────────────────

/// Mutable execution context threaded through all steps.
///
/// Steps can store and retrieve data by key so that later steps can access
/// results produced by earlier ones.
#[derive(Debug, Clone, Default)]
pub struct SagaContext {
    data: HashMap<String, Value>,
}

impl SagaContext {
    /// Create an empty context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialise a context with pre-seeded data.
    pub fn with_data(data: HashMap<String, Value>) -> Self {
        Self { data }
    }

    /// Store a value in the context.
    pub fn set(&mut self, key: impl Into<String>, value: Value) {
        self.data.insert(key.into(), value);
    }

    /// Retrieve a value from the context.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    /// Consume and return all stored data.
    pub fn into_data(self) -> HashMap<String, Value> {
        self.data
    }
}

// ─── Step / Compensation function aliases ────────────────────────────────────

type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;
type StepFn = Arc<dyn Fn(SagaContext) -> BoxFuture<Result<(SagaContext, StepResult)>> + Send + Sync>;
type CompFn = Arc<dyn Fn(SagaContext) -> BoxFuture<Result<SagaContext>> + Send + Sync>;

// ─── Step definition ─────────────────────────────────────────────────────────

struct StepDef {
    name: String,
    action: StepFn,
    compensation: Option<CompFn>,
}

// ─── Saga outcome ─────────────────────────────────────────────────────────────

/// Final outcome of running a saga.
#[derive(Debug)]
pub enum SagaOutcome {
    /// All steps completed successfully.
    Completed(SagaContext),
    /// A step failed; compensations were run.
    Compensated {
        /// The step that failed.
        failed_step: String,
        /// The original error.
        error: anyhow::Error,
        /// Context at the time of failure (after compensations).
        context: SagaContext,
    },
}

impl SagaOutcome {
    /// Returns `true` when all steps succeeded.
    pub fn is_completed(&self) -> bool {
        matches!(self, SagaOutcome::Completed(_))
    }

    /// Returns `true` when the saga was rolled back.
    pub fn is_compensated(&self) -> bool {
        matches!(self, SagaOutcome::Compensated { .. })
    }
}

// ─── Saga ─────────────────────────────────────────────────────────────────────

/// An ordered sequence of named steps with optional compensations.
pub struct Saga {
    name: String,
    steps: Vec<StepDef>,
}

impl Saga {
    /// Start building a saga with the given name.
    pub fn builder(name: impl Into<String>) -> SagaBuilder {
        SagaBuilder::new(name)
    }

    /// Return the name of this saga.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Execute the saga.
    ///
    /// Steps are run in order.  If a step returns an error the saga
    /// immediately starts running compensations for all previously-completed
    /// steps in reverse order, then returns [`SagaOutcome::Compensated`].
    pub async fn run(&self, initial_ctx: SagaContext) -> SagaOutcome {
        let mut ctx = initial_ctx;
        let mut completed: Vec<&StepDef> = Vec::new();

        for step in &self.steps {
            tracing::debug!(saga = %self.name, step = %step.name, "Executing step");
            match (step.action)(ctx.clone()).await {
                Ok((new_ctx, result)) => {
                    ctx = new_ctx;
                    if let Some(output) = result.output {
                        ctx.set(step.name.clone(), output);
                    }
                    completed.push(step);
                }
                Err(e) => {
                    tracing::warn!(
                        saga = %self.name,
                        step = %step.name,
                        error = %e,
                        "Step failed — running compensations"
                    );
                    // Run compensations in reverse order.
                    for prev in completed.iter().rev() {
                        if let Some(comp) = &prev.compensation {
                            match (comp)(ctx.clone()).await {
                                Ok(new_ctx) => ctx = new_ctx,
                                Err(ce) => {
                                    tracing::error!(
                                        saga = %self.name,
                                        step = %prev.name,
                                        error = %ce,
                                        "Compensation failed"
                                    );
                                }
                            }
                        }
                    }
                    return SagaOutcome::Compensated {
                        failed_step: step.name.clone(),
                        error: e,
                        context: ctx,
                    };
                }
            }
        }

        SagaOutcome::Completed(ctx)
    }
}

// ─── SagaBuilder ─────────────────────────────────────────────────────────────

/// Fluent builder for constructing a [`Saga`].
pub struct SagaBuilder {
    name: String,
    steps: Vec<StepDef>,
}

impl SagaBuilder {
    fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), steps: Vec::new() }
    }

    /// Add a step with an optional compensation.
    ///
    /// `action` receives the current context and must return the (possibly
    /// mutated) context plus a [`StepResult`].
    ///
    /// `compensation` (if provided) receives the context at the time of
    /// rollback and must return the (possibly mutated) context.
    pub fn step<A, AF, C, CF>(
        mut self,
        name: impl Into<String>,
        action: A,
        compensation: Option<C>,
    ) -> Self
    where
        A: Fn(SagaContext) -> AF + Send + Sync + 'static,
        AF: Future<Output = Result<(SagaContext, StepResult)>> + Send + 'static,
        C: Fn(SagaContext) -> CF + Send + Sync + 'static,
        CF: Future<Output = Result<SagaContext>> + Send + 'static,
    {
        let action_arc: StepFn = Arc::new(move |ctx| Box::pin(action(ctx)));
        let comp_arc: Option<CompFn> = compensation
            .map(|c| -> CompFn { Arc::new(move |ctx| Box::pin(c(ctx))) });

        self.steps.push(StepDef {
            name: name.into(),
            action: action_arc,
            compensation: comp_arc,
        });
        self
    }

    /// Finalise and return the [`Saga`].
    pub fn build(self) -> Saga {
        Saga { name: self.name, steps: self.steps }
    }
}

// ─── WorkflowEngine ───────────────────────────────────────────────────────────

/// Registry that holds named saga definitions and executes them by name.
///
/// Register all sagas at startup, then call `engine.execute("saga_name", ctx)`
/// from any handler.
#[derive(Default)]
pub struct WorkflowEngine {
    sagas: HashMap<String, Saga>,
}

impl WorkflowEngine {
    /// Create an empty engine.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a saga under its own name.
    pub fn register(&mut self, saga: Saga) {
        self.sagas.insert(saga.name().to_owned(), saga);
    }

    /// Execute a previously-registered saga.
    pub async fn execute(&self, name: &str, ctx: SagaContext) -> Result<SagaOutcome> {
        let saga = self.sagas.get(name)
            .ok_or_else(|| anyhow::anyhow!("No saga registered with name '{}'", name))?;
        Ok(saga.run(ctx).await)
    }

    /// List all registered saga names.
    pub fn list(&self) -> Vec<&str> {
        self.sagas.keys().map(|s| s.as_str()).collect()
    }
}
