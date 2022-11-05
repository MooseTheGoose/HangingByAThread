package com.binaryquackers.hbat;

import android.opengl.GLSurfaceView;
import android.os.Bundle;
import android.view.View;
import android.app.Activity;

import java.io.File;

public class MainActivity extends Activity {
    GLSurfaceView glView;
    private native void bridgeOnCreate();

    static {
        System.loadLibrary("hbatandroid");
    }

    @Override
    protected void onCreate(Bundle savedInstance) {
        super.onCreate(savedInstance);

        File dir = getFilesDir();
        bridgeOnCreate();
        glView = new MainGLSurfaceView(this);
        setContentView(glView);
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