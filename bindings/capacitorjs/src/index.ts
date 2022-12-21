import { registerPlugin } from '@capacitor/core'

import { IotaWalletMobileTypes } from './definitions'

const IotaWalletMobile = registerPlugin<IotaWalletMobileTypes>('IotaWalletMobile')

// import * as WalletApi from './lib/index.js'

// IotaWalletMobile.WalletApi = WalletApi

export * from './definitions'
export { IotaWalletMobile }
