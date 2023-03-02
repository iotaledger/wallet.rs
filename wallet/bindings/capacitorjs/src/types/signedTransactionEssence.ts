import type { ITransactionPayload } from '@iota/types';
import type { InputSigningData } from './preparedTransactionData';

<<<<<<< HEAD
/** The signed transaction with inputs data  */
=======
/** The signed transaction with inputs data */
>>>>>>> 5d1939575223b8004d642a02018d3e65f2ec4dbf
export interface SignedTransactionEssence {
    transactionPayload: ITransactionPayload;
    inputsData: InputSigningData;
}
