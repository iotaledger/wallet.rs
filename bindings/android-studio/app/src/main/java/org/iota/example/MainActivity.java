package org.iota.example;

import androidx.appcompat.app.AppCompatActivity;

import android.os.Bundle;

import org.iota.wallet.*;
import org.iota.wallet.local.*;

public class MainActivity extends AppCompatActivity {

    {
        NativeAPI.verifyLink();
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);
    }
}