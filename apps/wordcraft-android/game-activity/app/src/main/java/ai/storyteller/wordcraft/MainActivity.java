package ai.storyteller.wordcraft;

import android.content.Intent;
import android.net.Uri;
import android.os.Build;
import android.os.Bundle;
import android.provider.Settings;
import android.provider.MediaStore;
import android.util.Log;
import android.webkit.MimeTypeMap;
import android.view.WindowManager;

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
    private static final int REQUEST_PICK_IMAGE = 4103;

    private boolean photoPickerResultReady = false;
    private String photoPickerResult = null;

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
     * Open Android's native Photo Picker on Android 13+, with the system document picker
     * as a compatibility fallback on older versions. No photo-library permission is needed.
     */
    public void pickImage() {
        runOnUiThread(() -> {
            synchronized (this) {
                photoPickerResultReady = false;
                photoPickerResult = null;
            }

            Intent intent;
            if (Build.VERSION.SDK_INT >= 33) {
                intent = new Intent(MediaStore.ACTION_PICK_IMAGES);
            } else {
                intent = new Intent(Intent.ACTION_OPEN_DOCUMENT);
                intent.addCategory(Intent.CATEGORY_OPENABLE);
                intent.setType("image/*");
            }
            intent.addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION);
            try {
                startActivityForResult(intent, REQUEST_PICK_IMAGE);
            } catch (RuntimeException error) {
                Log.e("WordCraft", "Could not launch Android Photo Picker", error);
                synchronized (this) {
                    photoPickerResult = "";
                    photoPickerResultReady = true;
                }
            }
        });
    }

    @Override
    protected void onActivityResult(int requestCode, int resultCode, Intent data) {
        super.onActivityResult(requestCode, resultCode, data);
        if (requestCode != REQUEST_PICK_IMAGE) {
            return;
        }

        String path = null;
        if (resultCode == RESULT_OK && data != null) {
            Uri uri = data.getData();
            if (uri != null) {
                path = copyPickedImageToCache(uri);
            }
        }
        synchronized (this) {
            photoPickerResult = path;
            photoPickerResultReady = true;
        }
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
        String extension = "img";
        String mimeType = getContentResolver().getType(uri);
        if (mimeType != null) {
            String candidate = MimeTypeMap.getSingleton().getExtensionFromMimeType(mimeType);
            if (candidate != null && !candidate.isEmpty()) {
                extension = candidate;
            }
        }

        File target = null;
        try {
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
        } catch (IOException error) {
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
    }

    @Override
    protected void onResume() {
        super.onResume();
        hideSystemBars();
    }
}
