import type { ITransactionPayload } from '@iota/types'
import type { InputsData } from './transaction'

export type SignedTransactionEssence = {
    transactionPayload: ITransactionPayload,
    inputsData: InputsData
}
