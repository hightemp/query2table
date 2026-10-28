package com.hightemp.query2table

import android.os.Bundle
import android.system.Os
import android.view.View
import androidx.activity.enableEdgeToEdge
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    exportPaths()
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
    // Android draws apps under the status and navigation bars; keep the page clear of them,
    // of display cutouts and of the on-screen keyboard.
    val content = findViewById<View>(android.R.id.content)
    ViewCompat.setOnApplyWindowInsetsListener(content) { view, insets ->
      val bars = insets.getInsets(
        WindowInsetsCompat.Type.systemBars() or
          WindowInsetsCompat.Type.displayCutout() or
          WindowInsetsCompat.Type.ime()
      )
      view.setPadding(bars.left, bars.top, bars.right, bars.bottom)
      WindowInsetsCompat.CONSUMED
    }
  }

  /** Folders the Rust side reads for the bundled video tools, see src-tauri/src/components/bundled.rs. */
  private fun exportPaths() {
    Os.setenv("Q2T_NATIVE_LIB_DIR", applicationInfo.nativeLibraryDir, true)
    Os.setenv("Q2T_PACKAGES_DIR", java.io.File(noBackupFilesDir, "packages").path, true)
    // The default /data/local/tmp is not writable for apps.
    Os.setenv("TMPDIR", cacheDir.path, true)
  }
}
