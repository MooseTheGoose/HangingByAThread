package com.binaryquackers.hbat;

import android.opengl.GLSurfaceView;
import android.os.Bundle;
import android.view.View;
import android.app.Activity;

import java.io.File;

public class MainActivity extends Activity {
    private native boolean bridgeOnCreate();

    static {
        System.loadLibrary("hbatandroid");
    }

    static void onPanic(boolean status) {
        if (!status) {
            // Rust code will log into a file
            // or devcon why it crashed. For now,
            // just throw a RuntimeException.
            throw new RuntimeException(
            "Rust code panicked! Crashing immediately!");
        }
    }

    @Override
    protected void onCreate(Bundle savedInstance) {
        super.onCreate(savedInstance);
        onPanic(bridgeOnCreate());
    }

    @Override
    public void onWindowFocusChanged(boolean hasFocus) {
        super.onWindowFocusChanged(hasFocus);

        if (hasFocus) {
            hideSystemUi();
        }
    }

    private void hideSystemUi() {
        View decorView = getWindow().getDecorView();
        decorView.setSystemUiVisibility(
                View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY
                        | View.SYSTEM_UI_FLAG_LAYOUT_STABLE
                        | View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION
                        | View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN
                        | View.SYSTEM_UI_FLAG_HIDE_NAVIGATION
                        | View.SYSTEM_UI_FLAG_FULLSCREEN
        );
    }
}