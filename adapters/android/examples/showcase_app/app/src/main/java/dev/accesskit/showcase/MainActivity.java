package dev.accesskit.showcase;

import android.app.Activity;
import android.os.Bundle;

public final class MainActivity extends Activity {
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        ShowcaseView view = new ShowcaseView(this);
        setContentView(view);
        view.requestFocus();
    }
}
