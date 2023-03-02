// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { registerPlugin } from '@capacitor/core'

import { IotaWalletMobileTypes } from './definitions'

const IotaWalletMobile = registerPlugin<IotaWalletMobileTypes>('IotaWalletMobile')

export * from './definitions'

export { IotaWalletMobile }
