"use strict";
/**
 * This example creates a new database and account
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
        const account = yield manager.createAccount({ alias: 'Account # 01 ' });
        // Sync account with the network to fetch the latest information
        yield account.sync();
        console.info('New account created: ', account);
    });
}
run();
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiY3JlYXRlQWNjb3VudC5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uL3NyYy9jcmVhdGVBY2NvdW50LnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiI7QUFBQTs7R0FFRzs7Ozs7Ozs7Ozs7QUFFSCx5Q0FBdUQ7QUFFdkQsU0FBZSxHQUFHOztRQUNkLE1BQU0sT0FBTyxHQUFHLElBQUksdUJBQWMsQ0FBQztZQUMvQixXQUFXLEVBQUUsZUFBZTtZQUM1QixhQUFhLEVBQUU7Z0JBQ1gsS0FBSyxFQUFFO29CQUNIO3dCQUNJLEdBQUcsRUFBRSxxQ0FBcUM7cUJBQzdDO2lCQUNKO2dCQUNELFFBQVEsRUFBRSxJQUFJO2FBQ2pCO1lBQ0QsTUFBTSxFQUFFO2dCQUNKLFFBQVEsRUFBRSxpS0FBaUs7YUFDOUs7U0FDSixDQUFDLENBQUM7UUFFSCxNQUFNLE9BQU8sR0FBWSxNQUFNLE9BQU8sQ0FBQyxhQUFhLENBQUMsRUFBRSxLQUFLLEVBQUUsZUFBZSxFQUFFLENBQUMsQ0FBQTtRQUVoRixnRUFBZ0U7UUFDaEUsTUFBTSxPQUFPLENBQUMsSUFBSSxFQUFFLENBQUM7UUFFckIsT0FBTyxDQUFDLElBQUksQ0FBQyx1QkFBdUIsRUFBRSxPQUFPLENBQUMsQ0FBQztJQUNuRCxDQUFDO0NBQUE7QUFFRCxHQUFHLEVBQUUsQ0FBQyJ9