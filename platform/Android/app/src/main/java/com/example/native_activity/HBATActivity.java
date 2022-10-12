package com.example.native_activity;

import com.google.androidgamesdk.GameActivity;
import androidx.core.view.WindowCompat;
import android.os.Bundle;

public class HBATActivity extends GameActivity {
    static {
        System.loadLibrary("native-activity");
    }
    @Override
    protected void onCreate(Bundle savedInstanceState) {
	WindowCompat.setDecorFitsSystemWindows(getWindow(), false);
        super.onCreate(savedInstanceState); 
    }
    @Override
    protected void onResume() {
        super.onResume();
    }
}
