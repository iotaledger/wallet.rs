package org.iota.types;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.iota.types.account_method.AccountMethod;

public class AccountBalance implements ReturnJson {


    pub struct AccountBalanceDto {
        /// Total and available amount of the base coin
    #[serde(rename = "baseCoin")]
        pub base_coin: BaseCoinBalanceDto,
        /// Current required storage deposit amount
    #[serde(rename = "requiredStorageDeposit")]
        pub required_storage_deposit: String,
        /// Native tokens
    #[]
        pub native_tokens: Vec<NativeTokensBalanceDto>,
        /// Nfts
        pub nfts: Vec<NftId>,
        /// Aliases
        pub aliases: Vec<AliasId>,
        /// Foundries
        pub foundries: Vec<FoundryId>,
        /// Outputs with multiple unlock conditions and if they can currently be spent or not. If there is a
        /// [`TimelockUnlockCondition`] or [`ExpirationUnlockCondition`] this can change at any time
    #[serde(rename = "potentiallyLockedOutputs")]
        pub potentially_locked_outputs: HashMap<OutputId, bool>,
    }

    private BaseCoinBalance baseCoin;
    private String requiredStorageDeposit;
    private NativeToken[] native_tokens;

    public AccountBalance withAccountAddress(String accountAddress) {
        this.accountAddress = accountAddress;
        return this;
    }

    public AccountBalance withCirculating_supply(String circulatingSupply) {
        this.circulatingSupply = circulatingSupply;
        return this;
    }

    public AccountBalance withMaximumSupply(String maximumSupply) {
        this.maximumSupply = maximumSupply;
        return this;
    }

    public AccountBalance withFoundryMetadata(String foundryMetadata) {
        this.foundryMetadata = foundryMetadata;
        return this;
    }

    @Override
    public JsonElement toJson() {
        JsonObject o = new JsonObject();
        o.addProperty("accountAddress", accountAddress);
        o.addProperty("circulatingSupply", circulatingSupply);
        o.addProperty("maximumSupply", maximumSupply);
        o.addProperty("foundryMetadata", foundryMetadata);

        return o;
    }

}