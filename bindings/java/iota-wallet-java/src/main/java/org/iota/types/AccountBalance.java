package org.iota.types;

import org.iota.types.ids.AliasId;
import org.iota.types.ids.FoundryId;
import org.iota.types.ids.NftId;
import org.iota.types.ids.OutputId;

import java.util.Map;

public class AccountBalance extends AbstractObject {

    /// Total and available amount of the base coin
    private BaseCoinBalance baseCoin;
    /// Current required storage deposit amount
    private String requiredStorageDeposit;
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

}