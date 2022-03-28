use vm_core::{hasher::Digest, CLK_COL_IDX, FMP_COL_IDX, MIN_STACK_DEPTH, STACK_TRACE_OFFSET};
use winter_air::{
    Air, AirContext, Assertion, EvaluationFrame, ProofOptions as WinterProofOptions, TraceInfo,
    TransitionConstraintDegree,
};
use winter_utils::{ByteWriter, Serializable};

mod options;

// EXPORTS
// ================================================================================================

pub use options::ProofOptions;
pub use vm_core::{utils::ToElements, Felt, FieldElement, StarkField};
pub use winter_air::{FieldExtension, HashFunction};

// PROCESSOR AIR
// ================================================================================================

/// TODO: add docs
pub struct ProcessorAir {
    context: AirContext<Felt>,
    stack_inputs: Vec<Felt>,
    stack_outputs: Vec<Felt>,
}

impl Air for ProcessorAir {
    type BaseField = Felt;
    type PublicInputs = PublicInputs;

    fn new(trace_info: TraceInfo, pub_inputs: PublicInputs, options: WinterProofOptions) -> Self {
        let degrees = vec![
            TransitionConstraintDegree::new(1), // clk' = clk + 1
        ];

        Self {
            context: AirContext::new(trace_info, degrees, options),
            stack_inputs: pub_inputs.stack_inputs,
            stack_outputs: pub_inputs.stack_outputs,
        }
    }

    #[allow(clippy::vec_init_then_push)]
    fn get_assertions(&self) -> Vec<Assertion<Felt>> {
        let mut result = Vec::new();

        // --- set assertions for the first step --------------------------------------------------

        // first value of clk is 0
        result.push(Assertion::single(CLK_COL_IDX, 0, Felt::ZERO));

        // first value of fmp is 2^30
        result.push(Assertion::single(FMP_COL_IDX, 0, Felt::new(2u64.pow(30))));

        // stack columns at the first step should be set to stack inputs
        for (i, &value) in self.stack_inputs.iter().enumerate() {
            result.push(Assertion::single(STACK_TRACE_OFFSET + i, 0, value));
        }

        // --- set assertions for the last step ---------------------------------------------------
        let last_step = self.trace_length() - 1;

        // stack columns at the last step should be set to stack outputs
        for (i, &value) in self.stack_outputs.iter().enumerate() {
            result.push(Assertion::single(STACK_TRACE_OFFSET + i, last_step, value));
        }

        result
    }

    fn evaluate_transition<E: FieldElement<BaseField = Felt>>(
        &self,
        frame: &EvaluationFrame<E>,
        _periodic_values: &[E],
        result: &mut [E],
    ) {
        let current = frame.current();
        let next = frame.next();

        // clk' = clk + 1
        result[0] = next[CLK_COL_IDX] - (current[CLK_COL_IDX] + E::ONE)
    }

    fn context(&self) -> &AirContext<Felt> {
        &self.context
    }
}

// PUBLIC INPUTS
// ================================================================================================

#[derive(Debug)]
pub struct PublicInputs {
    program_hash: Digest,
    stack_inputs: Vec<Felt>,
    stack_outputs: Vec<Felt>,
}

impl PublicInputs {
    pub fn new(program_hash: Digest, stack_inputs: Vec<Felt>, stack_outputs: Vec<Felt>) -> Self {
        assert!(
            stack_inputs.len() <= MIN_STACK_DEPTH,
            "too many stack inputs"
        );
        assert!(
            stack_outputs.len() <= MIN_STACK_DEPTH,
            "too many stack outputs"
        );

        Self {
            program_hash,
            stack_inputs,
            stack_outputs,
        }
    }
}

impl Serializable for PublicInputs {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write(self.program_hash.as_elements());
        target.write(self.stack_inputs.as_slice());
        target.write(self.stack_outputs.as_slice());
    }
}
