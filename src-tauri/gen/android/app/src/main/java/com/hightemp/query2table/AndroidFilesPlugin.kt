package com.hightemp.query2table

import android.Manifest
import android.app.Activity
import android.content.ActivityNotFoundException
import android.content.ContentValues
import android.content.Intent
import android.content.pm.PackageManager
import android.media.MediaScannerConnection
import android.os.Build
import android.os.Environment
import android.provider.MediaStore
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import androidx.core.content.FileProvider
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.io.File

@InvokeArg
class CreateArgs {
  lateinit var name: String
  lateinit var mimeType: String
  /** "pictures" or "downloads". */
  lateinit var folder: String
}

@InvokeArg
class ViewArgs {
  lateinit var path: String
  lateinit var mimeType: String
}

@InvokeArg
class ScanArgs {
  lateinit var path: String
  lateinit var mimeType: String
}

/**
 * File work Android does not allow through paths.
 * - `create`: a file in the shared Pictures/Query2Table or Download/Query2Table folder, where the
 *   Gallery and Files apps show it. Returns where to write (`uri`: a content address, or a plain
 *   path before Android 10) and the folder and name to show (`path`); Rust writes the bytes.
 * - `view`: opens a file of the app's cache in another app, through the FileProvider.
 */
@TauriPlugin
class AndroidFilesPlugin(private val activity: Activity) : Plugin(activity) {
  private val subfolder = "Query2Table"

  @Command
  fun create(invoke: Invoke) {
    val args = invoke.parseArgs(CreateArgs::class.java)
    try {
      val pictures = args.folder == "pictures"
      if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
        val base = if (pictures) Environment.DIRECTORY_PICTURES else Environment.DIRECTORY_DOWNLOADS
        val relative = "$base/$subfolder"
        val values = ContentValues().apply {
          put(MediaStore.MediaColumns.DISPLAY_NAME, args.name)
          put(MediaStore.MediaColumns.MIME_TYPE, args.mimeType)
          put(MediaStore.MediaColumns.RELATIVE_PATH, relative)
        }
        val collection =
          if (pictures) MediaStore.Images.Media.EXTERNAL_CONTENT_URI else MediaStore.Downloads.EXTERNAL_CONTENT_URI
        val resolver = activity.contentResolver
        val uri = resolver.insert(collection, values) ?: throw IllegalStateException("The system did not create the file")
        // The system renames the file when the name is taken.
        var name = args.name
        resolver.query(uri, arrayOf(MediaStore.MediaColumns.DISPLAY_NAME), null, null, null)?.use { cursor ->
          if (cursor.moveToFirst()) name = cursor.getString(0) ?: name
        }
        invoke.resolve(JSObject().put("uri", uri.toString()).put("path", "$relative/$name"))
      } else {
        // Android 9 and older write shared folders as plain files, with the storage permission.
        val permission = Manifest.permission.WRITE_EXTERNAL_STORAGE
        if (ContextCompat.checkSelfPermission(activity, permission) != PackageManager.PERMISSION_GRANTED) {
          ActivityCompat.requestPermissions(activity, arrayOf(permission), 1)
          invoke.reject("Allow access to storage, then try again.")
          return
        }
        val base = if (pictures) Environment.DIRECTORY_PICTURES else Environment.DIRECTORY_DOWNLOADS
        val dir = File(Environment.getExternalStoragePublicDirectory(base), subfolder)
        dir.mkdirs()
        val file = unusedFile(dir, args.name)
        file.createNewFile()
        invoke.resolve(JSObject().put("uri", file.absolutePath).put("path", "$base/$subfolder/${file.name}"))
      }
    } catch (e: Exception) {
      invoke.reject(e.message ?: "Could not create the file")
    }
  }

  @Command
  fun view(invoke: Invoke) {
    val args = invoke.parseArgs(ViewArgs::class.java)
    try {
      val uri = FileProvider.getUriForFile(activity, "${activity.packageName}.fileprovider", File(args.path))
      val intent = Intent(Intent.ACTION_VIEW).apply {
        setDataAndType(uri, args.mimeType)
        addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
      }
      activity.startActivity(intent)
      invoke.resolve()
    } catch (e: ActivityNotFoundException) {
      invoke.reject("No app on this phone opens ${args.mimeType} files.")
    } catch (e: Exception) {
      invoke.reject(e.message ?: "Could not open the file")
    }
  }

  /** Android 9 and older: makes a file written by path visible in Gallery and Files. */
  @Command
  fun scan(invoke: Invoke) {
    val args = invoke.parseArgs(ScanArgs::class.java)
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.Q) {
      MediaScannerConnection.scanFile(activity, arrayOf(args.path), arrayOf(args.mimeType), null)
    }
    invoke.resolve()
  }

  private fun unusedFile(dir: File, name: String): File {
    val dot = name.lastIndexOf('.')
    val stem = if (dot > 0) name.substring(0, dot) else name
    val ext = if (dot > 0) name.substring(dot) else ""
    var file = File(dir, name)
    var n = 2
    while (file.exists()) {
      file = File(dir, "$stem ($n)$ext")
      n++
    }
    return file
  }
}
