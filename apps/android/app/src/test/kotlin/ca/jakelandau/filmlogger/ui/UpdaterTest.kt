package ca.jakelandau.filmlogger.ui

import java.io.File
import java.nio.file.Files
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class UpdaterTest {
    @Test
    fun newerReleaseDownloadsApkAndStartsInstall() {
        val dir = Files.createTempDirectory("filmlogger-update").toFile()
        var downloadedUrl: String? = null
        var downloadedFile: File? = null
        var installed: File? = null

        val result = Updater.updateFromRelease(
            tag = "0.2.0",
            assets = listOf(
                ReleaseAsset("filmlogger-v0.2.0-android.apk", "https://example.test/app.apk"),
                ReleaseAsset("filmlogger-v0.2.0-windows.exe", "https://example.test/app.exe"),
            ),
            current = "0.1.0",
            cacheDir = dir,
            download = { url, file ->
                downloadedUrl = url
                downloadedFile = file
                file.writeText("apk")
            },
            install = { installed = it },
        )

        assertTrue(result.updateStarted)
        assertEquals("https://example.test/app.apk", downloadedUrl)
        assertEquals(File(dir, "filmlogger-update.apk"), downloadedFile)
        assertEquals(downloadedFile, installed)
    }

    @Test
    fun currentReleaseDoesNotDownload() {
        val dir = Files.createTempDirectory("filmlogger-update").toFile()
        var downloaded = false

        val result = Updater.updateFromRelease(
            tag = "0.1.0",
            assets = listOf(ReleaseAsset("filmlogger-v0.1.0-android.apk", "https://example.test/app.apk")),
            current = "0.1.0",
            cacheDir = dir,
            download = { _, _ -> downloaded = true },
            install = { error("install should not start") },
        )

        assertFalse(result.updateStarted)
        assertFalse(downloaded)
        assertTrue(result.message.contains("up to date"))
    }
}
