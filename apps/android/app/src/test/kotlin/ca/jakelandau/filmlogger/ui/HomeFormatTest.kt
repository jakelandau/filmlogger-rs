package ca.jakelandau.filmlogger.ui

import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.filmlogger_rs_ffi.Format
import uniffi.filmlogger_rs_ffi.HomeMedium

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
}
