package ai.storyteller.wordcraft;

import android.Manifest;
import android.content.pm.PackageManager;
import android.content.Intent;
import android.net.Uri;
import android.provider.Settings;
import android.os.Build;
import android.os.Bundle;
import android.view.WindowManager;

import androidx.core.view.WindowCompat;
import androidx.core.view.WindowInsetsCompat;
import androidx.core.view.WindowInsetsControllerCompat;

import com.google.androidgamesdk.GameActivity;

/** AndroidX GameActivity host for WordCraft's Rust/winit GameActivity backend. */
public final class MainActivity extends GameActivity {
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
     * The in-app picker reads shared image folders by path. Android's scoped-storage
     * model requires the user to grant media access at runtime; declaring the
     * permission in the manifest alone is not sufficient.
     */
    private void requestImageReadAccess() {
        if (Build.VERSION.SDK_INT >= 34) {
            boolean full = checkSelfPermission(Manifest.permission.READ_MEDIA_IMAGES)
                    == PackageManager.PERMISSION_GRANTED;
            boolean selected = checkSelfPermission(Manifest.permission.READ_MEDIA_VISUAL_USER_SELECTED)
                    == PackageManager.PERMISSION_GRANTED;
            if (!full && !selected) {
                requestPermissions(new String[] {
                        Manifest.permission.READ_MEDIA_IMAGES,
                        Manifest.permission.READ_MEDIA_VISUAL_USER_SELECTED
                }, 4102);
            }
        } else if (Build.VERSION.SDK_INT >= 33) {
            if (checkSelfPermission(Manifest.permission.READ_MEDIA_IMAGES)
                    != PackageManager.PERMISSION_GRANTED) {
                requestPermissions(new String[] {
                        Manifest.permission.READ_MEDIA_IMAGES
                }, 4102);
            }
        } else if (Build.VERSION.SDK_INT >= 23) {
            if (checkSelfPermission(Manifest.permission.READ_EXTERNAL_STORAGE)
                    != PackageManager.PERMISSION_GRANTED) {
                requestPermissions(new String[] {
                        Manifest.permission.READ_EXTERNAL_STORAGE
                }, 4102);
            }
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
        requestImageReadAccess();
    }

    @Override
    protected void onResume() {
        super.onResume();
        hideSystemBars();
    }
}
