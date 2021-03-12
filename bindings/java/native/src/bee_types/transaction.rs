use iota_wallet::{
    message::{
        MessageTransactionPayload as MessageTransactionPayloadRust,
        TransactionRegularEssence as TransactionRegularEssenceRust, 
        TransactionEssence as TransactionEssenceRust,
        TransactionInput as RustWalletInput,
        TransactionOutput as RustWalletOutput,
    },
    address::{
        OutputKind as RustOutputKind,
    },
};

use iota::{
    Payload as RustPayload,
    UnlockBlock as RustUnlockBlock,
};

pub enum InputKind {
    UTXO = 0,
    Treasury = 1,
}

pub enum UnlockBlockKind {
    Reference = 0,
    Ed25519 = 1,
}
        
pub struct MessageTransactionPayload {
    essence: Essence,
    unlock_blocks: Vec<UnlockBlock>,
}

impl MessageTransactionPayload {
    pub fn new_with_rust(payload: &Box<MessageTransactionPayloadRust>) -> MessageTransactionPayload {
        Self {
            essence: Essence {
                essence: payload.essence().to_owned()
            },
            unlock_blocks: payload
                .unlock_blocks()
                .iter()
                .cloned()
                .map(|unlock_block| UnlockBlock {
                    unlock_block: unlock_block
                })
                .collect()
        }
    }

    pub fn essence(&self) -> Essence {
        self.essence.clone()
    }

    pub fn unlock_blocks(&self) -> Vec<UnlockBlock> {
        self.unlock_blocks
            .iter()
            .cloned()
            .collect()
    }
}
#[derive(Clone)]
pub struct Essence {
    essence: TransactionEssenceRust,
}

impl Essence {
    pub fn get_as_regular(&self) -> Option<RegularEssence> {
        if let TransactionEssenceRust::Regular(essence) = &self.essence {
            Some(RegularEssence { 
                essence: essence.clone()
            })
        } else {
            None
        }
    } 
}

#[derive(Clone)]
pub struct RegularEssence {
    essence: TransactionRegularEssenceRust,
}

impl RegularEssence {
    pub fn inputs(&self) -> Vec<TransactionInput> {
        self.essence.inputs().iter()
            .cloned()
            .map(|input| TransactionInput {
                input: input
            })
            .collect()
    }

    /// Gets the transaction outputs.
    pub fn outputs(&self) -> Vec<TransactionOutput> {
        self.essence.outputs().iter()
            .cloned()
            .map(|output| TransactionOutput {
                output: output
            })
            .collect()
    }

    /// Gets the transaction chained payload.
    pub fn payload(&self) -> &Option<RustPayload> {
        self.essence.payload()
    }
}

#[derive(Clone)]
pub struct TransactionInput {
    input: RustWalletInput,
}

impl TransactionInput {
    pub fn kind(&self) -> InputKind {
        match self.input {
            RustWalletInput::UTXO(_) => InputKind::UTXO,
            RustWalletInput::Treasury(_) => InputKind::Treasury,
        }
    }

    pub fn to_string(&self) -> String {
        format!("{:?}", self.input)
    }
}

#[derive(Clone)]
pub struct TransactionOutput {
    output: RustWalletOutput,
}

impl TransactionOutput {
    pub fn kind(&self) -> RustOutputKind {
        match self.output {
            RustWalletOutput::SignatureLockedSingle(_) => RustOutputKind::SignatureLockedSingle,
            RustWalletOutput::SignatureLockedDustAllowance(_) => RustOutputKind::SignatureLockedDustAllowance,
            RustWalletOutput::Treasury(_) => RustOutputKind::Treasury,
        }
    }

    pub fn to_string(&self) -> String {
        format!("{:?}", self.output)
    }
}

#[derive(Clone)]
pub struct UnlockBlock {
    unlock_block: RustUnlockBlock,
}

impl UnlockBlock {
    pub fn kind(&self) -> UnlockBlockKind {
        match self.unlock_block {
            RustUnlockBlock::Signature(_) => UnlockBlockKind::Ed25519,
            RustUnlockBlock::Reference(_) => UnlockBlockKind::Reference,
            _ => panic!("Found unknown unlock block")
        }
    }

    pub fn to_string(&self) -> String {
        format!("{:?}", self.unlock_block)
    }
}

/*
pub struct TransactionPayloadBuilder {
    builder: Rc<RefCell<Option<TransactionPayloadBuilderRust>>>
}

impl Default for TransactionPayloadBuilder {
    fn default() -> Self {
        Self {
            builder: Rc::new(RefCell::new(Option::from(TransactionPayloadBuilderRust::default())))
        }
    }
}

impl TransactionPayloadBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    fn new_with_builder(builder: TransactionPayloadBuilder) -> Self {
        Self {
            builder: Rc::new(RefCell::new(Option::from(builder)))
        }
    }
}
*/