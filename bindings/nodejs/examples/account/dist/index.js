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
        console.info('New account created', account);
        const addresses = yield account.listAddresses();
        console.info(addresses);
    });
}
run();
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiaW5kZXguanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi9zcmMvaW5kZXgudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IjtBQUFBOztHQUVHOzs7Ozs7Ozs7OztBQUVILHlDQUFnRTtBQUVoRSxTQUFlLEdBQUc7O1FBQ2QsTUFBTSxPQUFPLEdBQUcsSUFBSSx1QkFBYyxDQUFDO1lBQy9CLFdBQVcsRUFBRSxlQUFlO1lBQzVCLGFBQWEsRUFBRTtnQkFDWCxLQUFLLEVBQUU7b0JBQ0g7d0JBQ0ksR0FBRyxFQUFFLHFDQUFxQztxQkFDN0M7aUJBQ0o7Z0JBQ0QsUUFBUSxFQUFFLElBQUk7YUFDakI7WUFDRCxNQUFNLEVBQUU7Z0JBQ0osUUFBUSxFQUFFLGlLQUFpSzthQUM5SztTQUNKLENBQUMsQ0FBQztRQUVILE1BQU0sT0FBTyxHQUFZLE1BQU0sT0FBTyxDQUFDLGFBQWEsQ0FBQyxFQUFFLEtBQUssRUFBRSxlQUFlLEVBQUUsQ0FBQyxDQUFBO1FBQ2hGLE9BQU8sQ0FBQyxJQUFJLENBQUMscUJBQXFCLEVBQUUsT0FBTyxDQUFDLENBQUM7UUFFN0MsTUFBTSxTQUFTLEdBQWMsTUFBTSxPQUFPLENBQUMsYUFBYSxFQUFFLENBQUM7UUFDM0QsT0FBTyxDQUFDLElBQUksQ0FBQyxTQUFTLENBQUMsQ0FBQztJQUM1QixDQUFDO0NBQUE7QUFFRCxHQUFHLEVBQUUsQ0FBQyJ9