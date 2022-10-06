package org.iota.types;

public enum CoinType {
    Iota(4218),
    Shimmer(4219);

    private final int coinType;

    CoinType(int coinType) {
        this.coinType = coinType;
    }

    public int getCoinTypeValue() { return coinType; }
}
