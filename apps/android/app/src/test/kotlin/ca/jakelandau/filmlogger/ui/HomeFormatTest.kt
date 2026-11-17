package ca.jakelandau.filmlogger.ui

import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.filmlogger_rs_ffi.Format
import uniffi.filmlogger_rs_ffi.HomeMedium
import uniffi.filmlogger_rs_ffi.Venue

class HomeFormatTest {
    @Test
    fun threeDBluRayForcesTheThreeDFlag() {
        val format = Format.Home(HomeMedium.BluRay3D, threeD = true, hfr = false)
        assertEquals(listOf(FormatFlag.THREE_D), formatFlags(format))
    }

    @Test
    fun uhdBluRayExposesOptionalHfr() {
        val format = Format.Home(HomeMedium.UhdBluRay, threeD = false, hfr = true)
        assertEquals(listOf(FormatFlag.HFR), formatFlags(format))
    }

    @Test
    fun homeFormatRoundTripsThroughTextParts() {
        val format = Format.Home(HomeMedium.UhdBluRay, threeD = false, hfr = true)
        val parts = FormatText.decompose(format, Venue.HOME)
        assertEquals("uhd_bluray", parts.medium)
        assertEquals(true, parts.hfr)
        assertEquals(format, FormatText.build(parts))
    }
}
