package ai.storyteller.wordcraft;

import android.content.Intent;
import android.net.Uri;
import android.os.Build;
import android.os.Bundle;
import android.provider.Settings;
import android.util.Log;
import android.webkit.MimeTypeMap;
import android.view.WindowManager;

import androidx.activity.result.ActivityResultLauncher;
import androidx.activity.result.PickVisualMediaRequest;
import androidx.activity.result.contract.ActivityResultContracts;
import androidx.core.view.WindowCompat;
import androidx.core.view.WindowInsetsCompat;
import androidx.core.view.WindowInsetsControllerCompat;

import com.google.androidgamesdk.GameActivity;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;

/** AndroidX GameActivity host for WordCraft's Rust/winit GameActivity backend. */
public final class MainActivity extends GameActivity {
    private boolean photoPickerResultReady = false;
    private String photoPickerResult = null;
    private ActivityResultLauncher<PickVisualMediaRequest> photoPickerLauncher;

    static {
        System.loadLibrary("wordcraft");
    }

    private void hideSystemBars() {
        getWindow().getAttributes().layoutInDisplayCutoutMode =
                WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_ALWAYS;
        WindowInsetsControllerCompat controller =
                new WindowInsetsControllerCompat(getWindow(), getWindow().getDecorView());
        controller.hide(WindowInsetsCompat.Type.systemBars());
        controller.setSystemBarsBehavior(
                WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE);
    }

    /**
     * Open AndroidX's recommended Photo Picker contract. It prefers the native system picker,
     * then uses the available system/Google fallback or ACTION_OPEN_DOCUMENT on older devices.
     * The picker grants access only to the item selected; no photo-library permission is needed.
     */
    public void pickImage() {
        runOnUiThread(() -> {
            synchronized (this) {
                photoPickerResultReady = false;
                photoPickerResult = null;
            }
            try {
                photoPickerLauncher.launch(
                        new PickVisualMediaRequest.Builder()
                                .setMediaType(ActivityResultContracts.PickVisualMedia.ImageOnly.INSTANCE)
                                .build()
                );
            } catch (RuntimeException error) {
                Log.e("WordCraft", "Could not launch Android Photo Picker", error);
                completePhotoPickerResult(null);
            }
        });
    }

    private void handlePhotoPickerResult(Uri uri) {
        if (uri == null) {
            completePhotoPickerResult(null);
            return;
        }
        // Cloud-backed or very large images can take time to stream. Never copy them on
        // Android's main thread; WordCraft receives the result when the copy has completed.
        new Thread(() -> completePhotoPickerResult(copyPickedImageToCache(uri)),
                "wordcraft-photo-copy").start();
    }

    private synchronized void completePhotoPickerResult(String path) {
        photoPickerResult = path;
        photoPickerResultReady = true;
    }

    /** Null means no result yet; an empty string means cancelled or copy failed. */
    public synchronized String takePhotoPickerResult() {
        if (!photoPickerResultReady) {
            return null;
        }
        photoPickerResultReady = false;
        String result = photoPickerResult;
        photoPickerResult = null;
        return result == null ? "" : result;
    }

    /** Copy the temporary content-URI grant into app-private cache for the path-based engine. */
    private String copyPickedImageToCache(Uri uri) {
        File target = null;
        try {
            // ContentResolver calls can also throw SecurityException for a revoked or
            // unavailable URI grant. Keep them inside the failure path so Rust is always
            // notified instead of leaving the pending insert dialog waiting forever.
            String extension = "img";
            String mimeType = getContentResolver().getType(uri);
            if (mimeType != null) {
                String candidate = MimeTypeMap.getSingleton().getExtensionFromMimeType(mimeType);
                if (candidate != null && !candidate.isEmpty()) {
                    extension = candidate;
                }
            }

            target = File.createTempFile("wordcraft-image-", "." + extension, getCacheDir());
            try (InputStream input = getContentResolver().openInputStream(uri);
                 FileOutputStream output = new FileOutputStream(target)) {
                if (input == null) {
                    throw new IOException("The selected image could not be opened");
                }
                byte[] buffer = new byte[16 * 1024];
                int count;
                while ((count = input.read(buffer)) != -1) {
                    output.write(buffer, 0, count);
                }
                output.flush();
            }
            return target.getAbsolutePath();
        } catch (IOException | RuntimeException error) {
            if (target != null) {
                //noinspection ResultOfMethodCallIgnored
                target.delete();
            }
            Log.e("WordCraft", "Could not copy selected image into app cache", error);
            return null;
        }
    }

    /** Open the system page for granting this app broad shared-storage access. */
    public void openStorageSettings() {
        Intent intent;
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            intent = new Intent(Settings.ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION);
        } else {
            intent = new Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS);
        }
        intent.setData(Uri.parse("package:" + getPackageName()));
        try {
            startActivity(intent);
        } catch (android.content.ActivityNotFoundException ignored) {
            Intent fallback = new Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS);
            fallback.setData(Uri.parse("package:" + getPackageName()));
            startActivity(fallback);
        }
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        WindowCompat.setDecorFitsSystemWindows(getWindow(), false);
        hideSystemBars();
        super.onCreate(savedInstanceState);
        photoPickerLauncher = registerForActivityResult(
                new ActivityResultContracts.PickVisualMedia(),
                this::handlePhotoPickerResult
        );
    }

    @Override
    protected void onResume() {
        super.onResume();
        hideSystemBars();
    }
}
