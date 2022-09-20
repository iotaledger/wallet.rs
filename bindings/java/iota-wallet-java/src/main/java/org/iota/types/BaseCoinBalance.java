package org.iota.types;

public class BaseCoinBalance {
    /// Total amount
    private String total;
    /// Balance that can currently be spent
    private String available;

    public String getTotal() {
        return total;
    }

    public String getAvailable() {
        return available;
    }
}