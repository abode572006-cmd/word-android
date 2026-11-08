package com.abode.wordandroid

import androidx.activity.ComponentActivity
import android.net.Uri
import android.os.Bundle
import android.view.View
import android.webkit.ValueCallback
import android.webkit.WebChromeClient
import android.webkit.WebResourceRequest
import android.webkit.WebSettings
import android.webkit.WebView
import androidx.activity.result.contract.ActivityResultContracts
import androidx.webkit.WebViewAssetLoader
import androidx.webkit.WebViewClientCompat

class MainActivity : ComponentActivity() {
    private var pendingFileCallback: ValueCallback<Array<Uri>>? = null

    private val openDocuments =
        registerForActivityResult(ActivityResultContracts.OpenMultipleDocuments()) { uris ->
            pendingFileCallback?.onReceiveValue(uris.toTypedArray())
            pendingFileCallback = null
        }

    private lateinit var wordCraftWebView: WebView

    private val assetLoader by lazy {
        WebViewAssetLoader.Builder()
            .addPathHandler("/assets/", WebViewAssetLoader.AssetsPathHandler(this))
            .build()
    }

    @Suppress("SetJavaScriptEnabled")
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        wordCraftWebView = WebView(this).apply {
            setLayerType(View.LAYER_TYPE_HARDWARE, null)
            settings.javaScriptEnabled = true
            settings.domStorageEnabled = true
            settings.databaseEnabled = true
            settings.allowFileAccess = false
            settings.allowContentAccess = true
            settings.mixedContentMode = WebSettings.MIXED_CONTENT_NEVER_ALLOW
            settings.javaScriptCanOpenWindowsAutomatically = false
            settings.setSupportMultipleWindows(false)

            webViewClient = object : WebViewClientCompat() {
                override fun shouldInterceptRequest(
                    view: WebView,
                    request: WebResourceRequest
                ) = assetLoader.shouldInterceptRequest(request.url)
            }

            webChromeClient = object : WebChromeClient() {
                override fun onShowFileChooser(
                    webView: WebView,
                    filePathCallback: ValueCallback<Array<Uri>>,
                    fileChooserParams: FileChooserParams
                ): Boolean {
                    pendingFileCallback?.onReceiveValue(null)
                    pendingFileCallback = filePathCallback
                    openDocuments.launch(arrayOf("*/*"))
                    return true
                }
            }
        }

        setContentView(wordCraftWebView)
        wordCraftWebView.loadUrl(
            "https://appassets.androidplatform.net/assets/wordcraft/index.html"
        )
    }

    @Deprecated("Deprecated in Java")
    override fun onBackPressed() {
        if (::wordCraftWebView.isInitialized && wordCraftWebView.canGoBack()) {
            wordCraftWebView.goBack()
        } else {
            super.onBackPressed()
        }
    }

    override fun onDestroy() {
        pendingFileCallback?.onReceiveValue(null)
        pendingFileCallback = null
        if (::wordCraftWebView.isInitialized) {
            wordCraftWebView.destroy()
        }
        super.onDestroy()
    }
}
