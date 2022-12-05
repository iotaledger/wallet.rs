// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

package org.iota.types;

import org.iota.types.account_methods.RequestFundsFromFaucet;
import org.iota.types.ids.AliasId;
import org.iota.types.ids.FoundryId;
import org.iota.types.ids.NftId;
import org.iota.types.ids.OutputId;

import java.util.Map;
public class AccountBalance extends AbstractObject {

    /// Total and available amount of the base coin
    private BaseCoinBalance baseCoin;
    /// Current required storage deposit amount
    private RequiredStorageDeposit requiredStorageDeposit;
    /// Native tokens
    private NativeTokensBalance[] nativeTokens;
    /// Nfts
    private NftId[] nfts;
    /// Aliases
    private AliasId[] aliases;
    /// Foundries
    private FoundryId[] foundries;
    /// Outputs with multiple unlock conditions and if they can currently be spent or not. If there is a
    /// [`TimelockUnlockCondition`] or [`ExpirationUnlockCondition`] this can change at any time
    private Map<OutputId, Boolean> potentiallyLockedOutputs;

    public BaseCoinBalance getBaseCoin() {
        return baseCoin;
    }

    public RequiredStorageDeposit getRequiredStorageDeposit() {
        return requiredStorageDeposit;
    }

    public NativeTokensBalance[] getNativeTokens() {
        return nativeTokens;
    }

    public NftId[] getNfts() {
        return nfts;
    }

    public AliasId[] getAliases() {
        return aliases;
    }

    public FoundryId[] getFoundries() {
        return foundries;
    }

    public Map<OutputId, Boolean> getPotentiallyLockedOutputs() {
        return potentiallyLockedOutputs;
    }
}