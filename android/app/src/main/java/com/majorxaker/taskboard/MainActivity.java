package com.majorxaker.taskboard;

import android.app.Activity;
import android.content.SharedPreferences;
import android.graphics.Color;
import android.os.Bundle;
import android.view.Gravity;
import android.view.ViewGroup;
import android.webkit.JavascriptInterface;
import android.webkit.WebResourceError;
import android.webkit.WebResourceRequest;
import android.webkit.WebSettings;
import android.webkit.WebView;
import android.webkit.WebViewClient;
import android.widget.Button;
import android.widget.EditText;
import android.widget.FrameLayout;
import android.widget.LinearLayout;
import android.widget.TextView;
import android.widget.Toast;

/**
 * Slim TaskBoard client: the server's own web UI inside a WebView.
 *
 * <p>Single configuration item — the server URL. No auth (LAN use), stored in
 * plain {@link SharedPreferences}. If the server cannot be reached, a local
 * error page with Retry / Change-server actions is shown instead of the raw
 * WebView network error.
 */
public class MainActivity extends Activity {

    private static final String PREFS = "taskboard";
    private static final String KEY_URL = "server_url";

    private FrameLayout root;
    private WebView webView;
    private LinearLayout settingsView;
    private EditText urlInput;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        root = new FrameLayout(this);
        setContentView(root);

        webView = new WebView(this);
        webView.setLayoutParams(new FrameLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));

        WebSettings s = webView.getSettings();
        s.setJavaScriptEnabled(true);
        s.setDomStorageEnabled(true);
        s.setCacheMode(WebSettings.LOAD_DEFAULT);

        webView.addJavascriptInterface(new Bridge(), "TaskBoard");

        webView.setWebViewClient(new WebViewClient() {
            @Override
            public boolean shouldOverrideUrlLoading(WebView view, String url) {
                return false; // single-page app — keep everything in this WebView
            }

            @Override
            public void onReceivedError(WebView view, WebResourceRequest request, WebResourceError error) {
                // Ignore sub-frame/resource failures (favicon, aborted XHR, …).
                if (request.isForMainFrame()) showErrorPage();
            }
        });

        root.addView(webView);

        if (getSavedUrl() == null) {
            showSettings();
        } else {
            load();
        }
    }

    @Override
    protected void onDestroy() {
        if (webView != null) {
            root.removeView(webView);
            webView.destroy();
        }
        super.onDestroy();
    }

    @Override
    public void onBackPressed() {
        if (settingsView != null && getSavedUrl() != null) {
            // Settings overlay open, but a server is configured — go back to it.
            dismissSettings();
            return;
        }
        super.onBackPressed();
    }

    // ── Navigation ────────────────────────────────────────────────────────────

    private void load() {
        dismissSettings();
        String url = getSavedUrl();
        if (url == null) {
            showSettings();
        } else {
            webView.loadUrl(url);
        }
    }

    private void showErrorPage() {
        webView.loadUrl("file:///android_asset/error.html");
    }

    // ── Settings (the one and only config: server URL) ────────────────────────

    private void showSettings() {
        if (settingsView != null) return;

        LinearLayout box = new LinearLayout(this);
        box.setOrientation(LinearLayout.VERTICAL);
        box.setGravity(Gravity.CENTER_HORIZONTAL);
        box.setPadding(dp(32), dp(48), dp(32), dp(32));
        LinearLayout.LayoutParams lp = new LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT);
        box.setLayoutParams(lp);

        TextView title = new TextView(this);
        title.setText(R.string.app_name);
        title.setTextSize(28);
        title.setGravity(Gravity.CENTER);
        title.setTextColor(Color.WHITE);
        LinearLayout.LayoutParams tlp = new LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT);
        tlp.bottomMargin = dp(8);
        title.setLayoutParams(tlp);
        box.addView(title);

        TextView hint = new TextView(this);
        hint.setText(R.string.settings_hint);
        hint.setTextSize(14);
        hint.setGravity(Gravity.CENTER);
        hint.setTextColor(0xFF9AA3B2);
        LinearLayout.LayoutParams hlp = new LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT);
        hlp.bottomMargin = dp(24);
        hint.setLayoutParams(hlp);
        box.addView(hint);

        urlInput = new EditText(this);
        urlInput.setHint(R.string.url_hint);
        urlInput.setInputType(0x00000021); // textUri
        urlInput.setTextColor(Color.WHITE);
        urlInput.setHintTextColor(0xFF9AA3B2);
        urlInput.setTextSize(16);
        String saved = getSavedUrl();
        if (saved != null) urlInput.setText(saved);
        LinearLayout.LayoutParams ilp = new LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT);
        ilp.bottomMargin = dp(24);
        urlInput.setLayoutParams(ilp);
        box.addView(urlInput);

        Button save = new Button(this);
        save.setText(R.string.save);
        save.setTextColor(Color.WHITE);
        save.setBackgroundColor(0xFF3B82F6);
        save.setOnClickListener(v -> saveAndLoad());
        box.addView(save);

        settingsView = new LinearLayout(this);
        settingsView.setOrientation(LinearLayout.VERTICAL);
        settingsView.setGravity(Gravity.CENTER_VERTICAL);
        settingsView.setBackgroundColor(0xFF101418); // matches the web app's dark bg
        settingsView.addView(box);
        root.addView(settingsView);
    }

    private void dismissSettings() {
        if (settingsView != null) {
            root.removeView(settingsView);
            settingsView = null;
        }
    }

    private void saveAndLoad() {
        String url = urlInput.getText().toString().trim();
        if (!url.isEmpty() && !url.startsWith("http://") && !url.startsWith("https://")) {
            url = "http://" + url; // LAN use — default to http
        }
        if (!url.startsWith("http://") && !url.startsWith("https://") || url.equals("http://")) {
            Toast.makeText(this, R.string.invalid_url, Toast.LENGTH_SHORT).show();
            return;
        }
        getPreferences().edit().putString(KEY_URL, url).apply();
        load();
    }

    // ── Helpers ────────────────────────────────────────────────────────────────

    private String getSavedUrl() {
        return getPreferences().getString(KEY_URL, null);
    }

    private SharedPreferences getPreferences() {
        return getSharedPreferences(PREFS, MODE_PRIVATE);
    }

    private int dp(int v) {
        return Math.round(v * getResources().getDisplayMetrics().density);
    }

    /** JS bridge used by the local error page. */
    private class Bridge {
        @JavascriptInterface
        public void retry() {
            runOnUiThread(MainActivity.this::load);
        }

        @JavascriptInterface
        public void settings() {
            runOnUiThread(MainActivity.this::showSettings);
        }
    }
}
