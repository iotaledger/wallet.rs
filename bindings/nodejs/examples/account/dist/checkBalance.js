"use strict";
/**
 * This example demonstrates how to check account balance
 */
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
Object.defineProperty(exports, "__esModule", { value: true });
const wallet_1 = require("@iota/wallet");
function run() {
    return __awaiter(this, void 0, void 0, function* () {
        const manager = new wallet_1.AccountManager({
            storagePath: './__storage__',
            clientOptions: {
                nodes: [
                    {
                        url: 'https://api.alphanet.iotaledger.net'
                    }
                ],
                localPow: true,
            },
            signer: {
                Mnemonic: 'flight about meadow begin pigeon assault cricket when curve regular degree board river garlic pride salmon online course congress cup tiny south slender carpet'
            }
        });
        const account = yield manager.getAccount(0);
        // Sync account with the network before getting the balance
        yield account.sync();
        const balance = yield account.balance();
        console.info('Account balance: ', balance);
    });
}
run();
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiY2hlY2tCYWxhbmNlLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vc3JjL2NoZWNrQmFsYW5jZS50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiO0FBQUE7O0dBRUc7Ozs7Ozs7Ozs7O0FBRUgseUNBQWdGO0FBRWhGLFNBQWUsR0FBRzs7UUFDZCxNQUFNLE9BQU8sR0FBRyxJQUFJLHVCQUFjLENBQUM7WUFDL0IsV0FBVyxFQUFFLGVBQWU7WUFDNUIsYUFBYSxFQUFFO2dCQUNYLEtBQUssRUFBRTtvQkFDSDt3QkFDSSxHQUFHLEVBQUUscUNBQXFDO3FCQUM3QztpQkFDSjtnQkFDRCxRQUFRLEVBQUUsSUFBSTthQUNqQjtZQUNELE1BQU0sRUFBRTtnQkFDSixRQUFRLEVBQUUsaUtBQWlLO2FBQzlLO1NBQ0osQ0FBQyxDQUFDO1FBRUgsTUFBTSxPQUFPLEdBQVksTUFBTSxPQUFPLENBQUMsVUFBVSxDQUFDLENBQUMsQ0FBQyxDQUFDO1FBRXJELDJEQUEyRDtRQUMzRCxNQUFNLE9BQU8sQ0FBQyxJQUFJLEVBQUUsQ0FBQztRQUVyQixNQUFNLE9BQU8sR0FBbUIsTUFBTSxPQUFPLENBQUMsT0FBTyxFQUFFLENBQUM7UUFFeEQsT0FBTyxDQUFDLElBQUksQ0FBQyxtQkFBbUIsRUFBRSxPQUFPLENBQUMsQ0FBQztJQUMvQyxDQUFDO0NBQUE7QUFFRCxHQUFHLEVBQUUsQ0FBQyJ9