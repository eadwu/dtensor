use std::fmt::Display;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::topograph::{GraphDependencies, GraphView, UniqueIdentifier};

mod shader;
pub use shader::*;

mod generators;
pub use generators::*;

static ID_GENERATOR: AtomicU64 = AtomicU64::new(0);

pub type ShaderIRID = u64;

pub trait ShaderIRBuilder {
    fn build_shader_ir(&self) -> ShaderIR;
}

#[derive(Clone, Copy, Debug)]
pub enum ShaderIROp {
    MagicIndex,
    ReduceBegin,
    ReduceEnd,
    ReduceMagic,
    Const,
    Load,
    Store,
    Evaluate,
}

#[derive(Clone, Copy, Debug)]
pub enum ShaderIRType {
    F32,
    I32,
}

#[derive(Clone, Copy, Debug)]
pub enum ShaderIREvaluation {
    F32(f32),
    I32(i32),
    IDENTITY,
    EXP2,
    LOG2,
    CAST,
    SIN,
    SQRT,
    ABS,
    FLOOR,
    CEIL,
    ADD,
    SUB,
    MULTIPLY,
    DIVIDE,
    MAX,
    MOD,
    EQUAL,
    LESSTHAN,
}

impl ShaderIREvaluation {
    pub fn n_dependencies(&self) -> usize {
        match self {
            ShaderIREvaluation::F32(_) => 0,
            ShaderIREvaluation::I32(_) => 0,
            ShaderIREvaluation::IDENTITY => 1,
            ShaderIREvaluation::EXP2 => 1,
            ShaderIREvaluation::LOG2 => 1,
            ShaderIREvaluation::CAST => 1,
            ShaderIREvaluation::SIN => 1,
            ShaderIREvaluation::SQRT => 1,
            ShaderIREvaluation::ABS => 1,
            ShaderIREvaluation::FLOOR => 1,
            ShaderIREvaluation::CEIL => 1,
            ShaderIREvaluation::ADD => 2,
            ShaderIREvaluation::SUB => 2,
            ShaderIREvaluation::MULTIPLY => 2,
            ShaderIREvaluation::DIVIDE => 2,
            ShaderIREvaluation::MAX => 2,
            ShaderIREvaluation::MOD => 2,
            ShaderIREvaluation::EQUAL => 2,
            ShaderIREvaluation::LESSTHAN => 2,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ShaderIR(Arc<ShaderIRInternals>);

#[derive(Clone, Debug)]
pub struct ShaderIRInternals {
    id: ShaderIRID,
    op: ShaderIROp,
    datatype: ShaderIRType,
    inputs: Vec<ShaderIR>,
    evaltype: Option<ShaderIREvaluation>,
}

impl ShaderIR {
    pub fn new(
        op: ShaderIROp,
        datatype: ShaderIRType,
        inputs: &[ShaderIR],
        evaltype: Option<ShaderIREvaluation>,
    ) -> ShaderIR {
        let id = ID_GENERATOR.fetch_add(1, Ordering::Relaxed);
        let inputs = inputs.to_vec();
        ShaderIR(Arc::new(ShaderIRInternals {
            id,
            op,
            datatype,
            inputs,
            evaltype,
        }))
    }

    pub fn id(&self) -> ShaderIRID {
        self.0.id
    }

    pub fn op(&self) -> ShaderIROp {
        self.0.op
    }

    pub fn datatype(&self) -> ShaderIRType {
        self.0.datatype
    }

    pub fn inputs(&self) -> &[ShaderIR] {
        &self.0.inputs[..]
    }

    pub fn evaltype(&self) -> Option<ShaderIREvaluation> {
        self.0.evaltype
    }

    pub fn literal(&self) -> String {
        self.linearize()
            .into_iter()
            .map(|ir| format!("{}", ir))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl GraphDependencies for ShaderIR {
    type Dependency = ShaderIR;
    fn dependencies(&self) -> Vec<Self::Dependency> {
        self.inputs()
            .iter()
            .map(|ir| ir.clone())
            .collect::<Vec<_>>()
    }
}

impl UniqueIdentifier for ShaderIR {
    type Id = u64;
    fn id(&self) -> Self::Id {
        self.id()
    }
}

impl Display for ShaderIR {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inputs = format!(
            "[{}]",
            self.inputs()
                .iter()
                .map(|ir| ir.id().to_string())
                .collect::<Vec<_>>()
                .join(",")
        );

        let evaltype = self
            .0
            .evaltype
            .map(|op| op.to_string())
            .unwrap_or("".to_string());

        write!(
            f,
            "{:<8} {:<16} {:<4} {:<16} {:<}",
            self.id(),
            self.0.op.to_string(),
            self.datatype().to_string(),
            inputs,
            evaltype
        )
    }
}

impl Display for ShaderIROp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for ShaderIRType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for ShaderIREvaluation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
