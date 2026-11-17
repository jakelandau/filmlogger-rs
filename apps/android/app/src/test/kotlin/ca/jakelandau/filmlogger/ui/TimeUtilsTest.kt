package ca.jakelandau.filmlogger.ui

import java.time.ZoneId
import org.junit.Assert.assertEquals
import org.junit.Test

class TimeUtilsTest {
    @Test
    fun displayUsesFullMonthAnd24HourTime() {
        assertEquals(
            "June 1, 2020, 08:00",
            TimeUtils.display("2020-06-01T12:00:00Z", ZoneId.of("America/New_York")),
        )
    }
}
