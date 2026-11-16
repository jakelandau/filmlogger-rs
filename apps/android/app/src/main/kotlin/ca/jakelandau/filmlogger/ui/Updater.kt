package ca.jakelandau.filmlogger.ui

import android.content.Context
import android.content.Intent
import androidx.core.content.FileProvider
import java.io.File
import java.net.HttpURLConnection
import java.net.URL
import org.json.JSONObject

/** GitHub Releases based updater for the Android app. */
object Updater {
    private const val REPO = "jakelandau/filmlogger-rs"
    private const val LATEST_URL = "https://api.github.com/repos/$REPO/releases/latest"

    data class Result(val message: String, val updateStarted: Boolean = false)

    /**
     * Checks the latest GitHub release, downloads the APK when it is newer,
     * and opens the system install prompt.
     */
    fun checkAndInstall(context: Context): Result {
        val current = currentVersion(context)
        val release = fetchLatestRelease()
        val tag = release.optString("tag_name").removePrefix("v")
        val assetsJson = release.optJSONArray("assets") ?: return Result("No release assets found.")
        val assets = buildList {
            for (index in 0 until assetsJson.length()) {
                val asset = assetsJson.getJSONObject(index)
                add(
                    ReleaseAsset(
                        asset.optString("name"),
                        asset.optString("browser_download_url"),
                    )
                )
            }
        }
        return updateFromRelease(
            tag = tag,
            assets = assets,
            current = current,
            cacheDir = context.cacheDir,
            download = ::download,
            install = { install(context, it) },
        )
    }

    /** Testable release step: select, download, then start the APK installer. */
    internal fun updateFromRelease(
        tag: String,
        assets: List<ReleaseAsset>,
        current: String,
        cacheDir: File,
        download: (String, File) -> Unit,
        install: (File) -> Unit,
    ): Result {
        if (tag.isBlank() || !UpdateLogic.isNewer(tag, current)) {
            return Result("FilmLogger is up to date (v$current).")
        }
        val asset = UpdateLogic.selectApkAsset(assets)
            ?: return Result("The latest release has no Android APK.")

        val apk = File(cacheDir, "filmlogger-update.apk")
        download(asset.url, apk)
        install(apk)
        return Result("Downloaded v$tag. Opening the Android installer…", updateStarted = true)
    }

    private fun currentVersion(context: Context): String =
        runCatching {
            context.packageManager.getPackageInfo(context.packageName, 0).versionName
        }.getOrNull().orEmpty().ifBlank { "0.0.0" }

    private fun fetchLatestRelease(): JSONObject {
        val connection = URL(LATEST_URL).openConnection() as HttpURLConnection
        connection.connectTimeout = 15_000
        connection.readTimeout = 15_000
        connection.setRequestProperty("Accept", "application/vnd.github+json")
        connection.setRequestProperty("User-Agent", "FilmLogger-Android")
        try {
            if (connection.responseCode !in 200..299) {
                error("GitHub returned HTTP ${connection.responseCode}")
            }
            val body = connection.inputStream.bufferedReader().use { it.readText() }
            return JSONObject(body)
        } finally {
            connection.disconnect()
        }
    }

    private fun download(url: String, destination: File) {
        val connection = URL(url).openConnection() as HttpURLConnection
        connection.connectTimeout = 15_000
        connection.readTimeout = 120_000
        connection.setRequestProperty("User-Agent", "FilmLogger-Android")
        connection.instanceFollowRedirects = true
        try {
            if (connection.responseCode !in 200..299) {
                error("Download failed with HTTP ${connection.responseCode}")
            }
            connection.inputStream.use { input ->
                destination.outputStream().use { output -> input.copyTo(output) }
            }
        } finally {
            connection.disconnect()
        }
    }

    private fun install(context: Context, apk: File) {
        val uri = FileProvider.getUriForFile(
            context,
            "${context.packageName}.fileprovider",
            apk,
        )
        context.startActivity(
            Intent(Intent.ACTION_VIEW).apply {
                setDataAndType(uri, "application/vnd.android.package-archive")
                addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            }
        )
    }

    /** Numeric semver comparison; prerelease/build suffixes are ignored. */
    fun isNewer(remote: String, current: String): Boolean =
        UpdateLogic.isNewer(remote, current)
}
