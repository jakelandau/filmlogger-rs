package ca.jakelandau.filmlogger.ui

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class UpdateLogicTest {
    @Test
    fun comparesSemanticVersionsNumerically() {
        assertTrue(UpdateLogic.isNewer("0.2.0", "0.1.9"))
        assertTrue(UpdateLogic.isNewer("v1.0.0", "0.9.9"))
        assertFalse(UpdateLogic.isNewer("0.1.0", "0.1.0"))
        assertFalse(UpdateLogic.isNewer("0.1.0", "0.1.1"))
    }

    @Test
    fun selectsTheAndroidApkAsset() {
        val asset = UpdateLogic.selectApkAsset(
            listOf(
                ReleaseAsset("FilmLogger_0.2.0_x64-setup.exe", "exe"),
                ReleaseAsset("FilmLogger_0.2.0_amd64.AppImage", "appimage"),
                ReleaseAsset("filmlogger-v0.2.0-android.apk", "apk"),
            )
        )
        assertEquals("apk", asset?.url)
    }
}
