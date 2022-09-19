package org.iota.types.outputs;

public class TreasuryOutput extends Output {

    private int type = 2;
    private String amount;

    public TreasuryOutput (String amount) {
        this.amount = amount;
    }

    public int getType() {
        return type;
    }

    public String getAmount() {
        return amount;
    }
}
