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
use async_trait::async_trait;
use sqlx::Row;

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

    /// Get a reference to a saga by name.
    pub fn get(&self, name: &str) -> Option<&Saga> {
        self.sagas.get(name)
    }
}


// ─── Persistent storage trait for sagas (Postgres implementation provided) ───

#[async_trait]
pub trait SagaStorage: Send + Sync {
    async fn create_execution(
        &self,
        transaction_id: &str,
        name: &str,
        ctx: &SagaContext,
    ) -> anyhow::Result<uuid::Uuid>;

    async fn update_state(
        &self,
        id: uuid::Uuid,
        state: &str,
        current_step: Option<&str>,
        error: Option<&str>,
    ) -> anyhow::Result<()>;

    async fn save_step(
        &self,
        exec_id: uuid::Uuid,
        step_name: &str,
        state: &str,
        result: Option<Value>,
        error: Option<&str>,
    ) -> anyhow::Result<()>;

    async fn load_by_transaction_id(
        &self,
        transaction_id: &str,
    ) -> anyhow::Result<Option<(uuid::Uuid, String, SagaContext, String)>>;

    async fn increment_retry(&self, id: uuid::Uuid) -> anyhow::Result<i32>;
}

pub struct PostgresSagaStorage {
    pool: sqlx::PgPool,
}

impl PostgresSagaStorage {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SagaStorage for PostgresSagaStorage {
    async fn create_execution(
        &self,
        transaction_id: &str,
        name: &str,
        ctx: &SagaContext,
    ) -> anyhow::Result<uuid::Uuid> {
        let ctx_json = serde_json::to_value(&ctx.data)?;

        let id: uuid::Uuid = sqlx::query_scalar::<_, uuid::Uuid>(
            r#"
            INSERT INTO workflow_executions (transaction_id, workflow_name, state, context)
            VALUES ($1, $2, 'not_started', $3)
            RETURNING id
            "#,
        )
        .bind(transaction_id)
        .bind(name)
        .bind(ctx_json)
        .fetch_one(&self.pool)
        .await?;

        Ok(id)
    }

    async fn update_state(
        &self,
        id: uuid::Uuid,
        state: &str,
        current_step: Option<&str>,
        error: Option<&str>,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE workflow_executions
            SET state = $1::workflow_state,
                current_step = $2,
                error = $3,
                updated_at = NOW()
            WHERE id = $4
            "#,
        )
        .bind(state)
        .bind(current_step)
        .bind(error)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn save_step(
        &self,
        exec_id: uuid::Uuid,
        step_name: &str,
        state: &str,
        result: Option<Value>,
        error: Option<&str>,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO workflow_steps (execution_id, step_name, state, invoke_result, error, started_at)
            VALUES ($1, $2, $3::step_state, $4, $5, NOW())
            ON CONFLICT (execution_id, step_name)
            DO UPDATE SET
                state = EXCLUDED.state,
                invoke_result = EXCLUDED.invoke_result,
                error = EXCLUDED.error,
                completed_at = CASE WHEN EXCLUDED.state IN ('done', 'failed', 'reverted') THEN NOW() ELSE workflow_steps.completed_at END,
                attempts = workflow_steps.attempts + 1
            "#,
        )
        .bind(exec_id)
        .bind(step_name)
        .bind(state)
        .bind(result)
        .bind(error)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn load_by_transaction_id(
        &self,
        transaction_id: &str,
    ) -> anyhow::Result<Option<(uuid::Uuid, String, SagaContext, String)>> {
        let rec = sqlx::query(
            r#"
            SELECT id, workflow_name, context, state::TEXT as state
            FROM workflow_executions
            WHERE transaction_id = $1
            "#,
        )
        .bind(transaction_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(r) = rec {
            let id: uuid::Uuid = r.try_get("id")?;
            let workflow_name: String = r.try_get("workflow_name")?;
            let ctx_value: serde_json::Value = r.try_get("context")?;
            let state: String = r.try_get("state")?;

            let data: std::collections::HashMap<String, Value> = serde_json::from_value(ctx_value)?;
            let ctx = SagaContext::with_data(data);
            Ok(Some((id, workflow_name, ctx, state)))
        } else {
            Ok(None)
        }
    }

    async fn increment_retry(&self, id: uuid::Uuid) -> anyhow::Result<i32> {
        let retry_count: i32 = sqlx::query_scalar::<_, i32>(
            r#"
            UPDATE workflow_executions
            SET retry_count = retry_count + 1,
                updated_at = NOW()
            WHERE id = $1
            RETURNING retry_count
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        Ok(retry_count)
    }
}

// ─── Persistent saga runner on Saga (idempotent) ─────────────────────────────

impl Saga {
    /// Execute saga with persistent storage and idempotency.
    pub async fn run_persistent(
        &self,
        transaction_id: String,
        initial_ctx: SagaContext,
        storage: &dyn SagaStorage,
    ) -> anyhow::Result<SagaOutcome> {
        // Check idempotency
        if let Some((exec_id, _name, saved_ctx, state)) = storage.load_by_transaction_id(&transaction_id).await? {
            tracing::info!(transaction_id = %transaction_id, execution_id = %exec_id, state = %state, "Workflow already exists (idempotent)");

            if state == "done" {
                return Ok(SagaOutcome::Completed(saved_ctx));
            } else if state == "reverted" || state == "failed" {
                return Ok(SagaOutcome::Compensated {
                    failed_step: "unknown".into(),
                    error: anyhow::anyhow!("Previously failed"),
                    context: saved_ctx,
                });
            }
            // otherwise continue
        }

        // create execution
        let exec_id = storage.create_execution(&transaction_id, &self.name, &initial_ctx).await?;
        storage.update_state(exec_id, "invoking", None, None).await?;

        let mut ctx = initial_ctx;
        let mut completed: Vec<&StepDef> = Vec::new();

        for step in &self.steps {
            storage.update_state(exec_id, "invoking", Some(&step.name), None).await?;
            storage.save_step(exec_id, &step.name, "invoking", None, None).await?;

            tracing::debug!(transaction_id = %transaction_id, saga = %self.name, step = %step.name, "Executing step");

            match (step.action)(ctx.clone()).await {
                Ok((new_ctx, result)) => {
                    ctx = new_ctx;
                    storage.save_step(exec_id, &step.name, "done", result.output.clone(), None).await?;
                    if let Some(output) = result.output {
                        ctx.set(step.name.clone(), output);
                    }
                    completed.push(step);
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    tracing::warn!(transaction_id = %transaction_id, saga = %self.name, step = %step.name, error = %e, "Step failed — running compensations");
                    storage.save_step(exec_id, &step.name, "failed", None, Some(&err_msg)).await?;
                    storage.update_state(exec_id, "compensating", Some(&step.name), Some(&err_msg)).await?;

                    for prev in completed.iter().rev() {
                        if let Some(comp) = &prev.compensation {
                            storage.save_step(exec_id, &prev.name, "compensating", None, None).await?;
                            match (comp)(ctx.clone()).await {
                                Ok(new_ctx) => {
                                    ctx = new_ctx;
                                    storage.save_step(exec_id, &prev.name, "reverted", None, None).await?;
                                }
                                Err(ce) => {
                                    let ce_msg = ce.to_string();
                                    storage.save_step(exec_id, &prev.name, "failed", None, Some(&ce_msg)).await?;
                                    tracing::error!(saga = %self.name, step = %prev.name, error = %ce, "Compensation failed");
                                }
                            }
                        }
                    }

                    storage.update_state(exec_id, "reverted", None, Some(&err_msg)).await?;
                    return Ok(SagaOutcome::Compensated {
                        failed_step: step.name.clone(),
                        error: e,
                        context: ctx,
                    });
                }
            }
        }

        storage.update_state(exec_id, "done", None, None).await?;
        tracing::info!(transaction_id = %transaction_id, saga = %self.name, "Saga completed successfully");
        Ok(SagaOutcome::Completed(ctx))
    }
}
