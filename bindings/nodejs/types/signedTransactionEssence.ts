import type { ITransactionPayload } from '@iota/types';
import type { InputSigningData } from './preparedTransactionData';

export interface SignedTransactionEssence {
    transactionPayload: ITransactionPayload;
    inputsData: InputSigningData;
}
