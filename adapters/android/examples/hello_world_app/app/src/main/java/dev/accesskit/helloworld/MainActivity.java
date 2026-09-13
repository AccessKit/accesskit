package dev.accesskit.helloworld;

import android.app.Activity;
import android.os.Bundle;

public final class MainActivity extends Activity {
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        HelloWorldView view = new HelloWorldView(this);
        setContentView(view);
        view.requestFocus();
    }
}
